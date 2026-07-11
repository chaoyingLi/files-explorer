import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { FileEntry, DiskInfo, SpecialDirs } from "@/types";
import * as tauri from "@/utils/tauri";
import { normalizePath } from "@/utils/platform";
import { useTabStore } from "@/stores/tabStore";
import { useNavigationStore } from "@/stores/navigationStore";
import { useSelectionStore } from "@/stores/selectionStore";
import { useViewStore } from "@/stores/viewStore";
import { useDeleteStore } from "@/stores/deleteStore";

// Navigation token to cancel stale async operations
let navigateSeq = 0;

// Shared directory-first case-insensitive sort
export function sortDirFirst(files: FileEntry[]): FileEntry[] {
  return [...files].sort((a, b) => {
    if (a.is_dir && !b.is_dir) return -1;
    if (!a.is_dir && b.is_dir) return 1;
    return a.name.localeCompare(b.name, undefined, { sensitivity: "base" });
  });
}

export const useFileStore = defineStore("file", () => {
  // ── Core state ──
  const currentPath = ref("");
  const files = ref<FileEntry[]>([]);
  const drives = ref<DiskInfo[]>([]);
  const specialDirs = ref<SpecialDirs | null>(null);
  const isSearching = ref(false);
  const loading = ref(false);
  const error = ref("");

  // ── Undo state ──
  const canUndo = ref(false);
  const undoDescription = ref("");

  // ── Recent items ──
  interface RecentItem {
    path: string;
    name: string;
    isDir: boolean;
    ext: string;
    time: number;
  }
  const recentItems = ref<RecentItem[]>(loadRecentItems());

  // ── Lazy store refs (avoid circular init at module level) ──
  let _tabStore: ReturnType<typeof useTabStore> | null = null;
  function getTabStore() {
    if (!_tabStore) _tabStore = useTabStore();
    return _tabStore;
  }

  // ── Computed ──
  const currentDirectoryName = computed(() => {
    if (!currentPath.value) return "";
    const parts = currentPath.value.replace(/\\/g, "/").split("/");
    return parts[parts.length - 1] || currentPath.value;
  });

  const pathSegments = computed(() => {
    if (!currentPath.value) return [];
    const parts = currentPath.value
      .replace(/\\/g, "/")
      .split("/")
      .filter(Boolean);
    const segments: { name: string; path: string }[] = [];
    let accumulated = "";

    if (currentPath.value.match(/^[A-Za-z]:/)) {
      accumulated = parts[0] + "/";
      segments.push({ name: parts[0], path: accumulated });
      parts.shift();
    } else {
      segments.push({ name: "/", path: "/" });
    }

    for (const part of parts) {
      if (
        accumulated &&
        !accumulated.endsWith("/") &&
        !accumulated.endsWith("\\")
      ) {
        accumulated += "/";
      }
      accumulated += part;
      segments.push({ name: part, path: accumulated });
    }

    return segments;
  });

  // ── Persistence bridge (sync with tabs) ──
  function syncToTab() {
    const tab = getTabStore().getFocusedTab();
    if (!tab || tab.isSearch) return;
    const sel = useSelectionStore();
    const view = useViewStore();
    tab.path = currentPath.value;
    tab.files = [...files.value];
    tab.selectedFiles =
      sel.selectedFiles.size > 0 ? [...sel.selectedFiles] : [];
    tab.treeExpanded = view.treeExpanded.size > 0 ? [...view.treeExpanded] : [];
  }

  function loadFromTab() {
    const tab = getTabStore().getFocusedTab();
    if (!tab) return;
    const sel = useSelectionStore();
    const view = useViewStore();
    files.value = [...(tab.files || [])];
    sel.selectedFiles = new Set(tab.selectedFiles || []);
    const raw = tab.path || "";
    currentPath.value = normalizePath(raw);
    if (tab.treeExpanded) view.treeExpanded = new Set(tab.treeExpanded);
  }

  // ── Drive / root loading ──
  async function loadDrives() {
    try {
      drives.value = await tauri.getDrives();
      specialDirs.value = await tauri.getSpecialDirs();
    } catch (e) {
      console.error("loadDrives failed:", e);
    }
    // Listen for preview-window file changes
    setupFileChangeListener();
  }

  // ── Listen for file changes from preview window ──
  let _fileChangeUnlisten: (() => void) | null = null;

  async function setupFileChangeListener() {
    if (_fileChangeUnlisten) return;
    try {
      const { listen } = await import("@tauri-apps/api/event");
      _fileChangeUnlisten = await listen<{ path: string }>(
        "file-changed",
        async (ev) => {
          if (ev.payload.path && currentPath.value) {
            // If the changed directory matches our current path, refresh
            const normCurrent = currentPath.value.replace(/\\/g, "/");
            const normChanged = ev.payload.path.replace(/\\/g, "/");
            if (normCurrent === normChanged) {
              await refresh();
            }
            // Also refresh drives in case of external drive changes
            if (normChanged === "/" || normChanged.match(/^[A-Za-z]:\/?$/)) {
              await loadDrives();
            }
          }
        },
      );
    } catch {
      /* ignore */
    }
  }

  // ── Navigation ──
  let _listUnlisten: (() => void) | null = null;

  async function navigateTo(path: string, addToHistory = true) {
    loading.value = true;
    error.value = "";

    if (isSearching.value) {
      isSearching.value = false;
      cancelCurrentSearch();
    }
    if (_listUnlisten) {
      _listUnlisten();
      _listUnlisten = null;
    }

    // Reset other stores
    const sel = useSelectionStore();
    const view = useViewStore();
    sel.resetCutState();
    sel.clearSelection();
    view.resetTreeState();

    files.value = [];
    currentPath.value = normalizePath(path);

    // Column view: prepare single-column stack for this path
    if (view.viewMode === "column") {
      const tab = getTabStore().getFocusedTab();
      if (tab) {
        tab.columnStack = [
          {
            path,
            name: path.split(/[/\\]/).filter(Boolean).pop() || path,
            files: [],
            selectedIndex: -1,
            loading: true,
          },
        ];
      }
    }

    if (addToHistory) {
      useNavigationStore().pushHistory(path);
    }

    const navId = ++navigateSeq;
    let cleanupListeners: (() => void) | null = null;

    try {
      const { listen } = await import("@tauri-apps/api/event");
      let batchIndex = 0;
      let resolved = false;

      const unlistenProgress = await listen<FileEntry[]>(
        "list-progress",
        (event) => {
          if (resolved || navId !== navigateSeq) return;
          batchIndex++;
          for (const f of event.payload) {
            files.value.push(f);
          }
          if (batchIndex % 3 === 0) {
            files.value = sortDirFirst(files.value);
          }
        },
      );

      let unlistenDone: (() => void) | null = null;
      let unlistenError: (() => void) | null = null;

      const cleanupListenersFn = () => {
        unlistenProgress();
        if (unlistenDone) unlistenDone();
        if (unlistenError) unlistenError();
      };

      cleanupListeners = cleanupListenersFn;
      _listUnlisten = cleanupListenersFn;

      const streamDone = new Promise<void>((resolve, reject) => {
        listen<boolean>("list-done", () => {
          if (resolved || navId !== navigateSeq) return;
          resolved = true;
          cleanupListenersFn();
          files.value = sortDirFirst(files.value);
          loading.value = false;
          // Populate column view if active
          if (view.viewMode === "column") {
            const tab = getTabStore().getFocusedTab();
            if (tab?.columnStack && tab.columnStack.length > 0) {
              tab.columnStack[0].files = files.value;
              tab.columnStack[0].loading = false;
            }
          }
          syncToTab();
          // Record recent
          const parts = path.replace(/\\/g, "/").split("/");
          const name = parts[parts.length - 1] || path;
          addRecentItem(path, name, true, "");
          resolve();
        }).then((u) => {
          unlistenDone = u;
        });
        listen<string>("list-error", (ev) => {
          if (resolved || navId !== navigateSeq) return;
          resolved = true;
          cleanupListenersFn();
          loading.value = false;
          error.value = ev.payload;
          reject(new Error(ev.payload));
        }).then((u) => {
          unlistenError = u;
        });
      });

      await tauri.listDirectoryStreamed(path);
      await streamDone;
    } catch (_e) {
      if (navId !== navigateSeq) return;
      try {
        files.value = await tauri.listDirectory(path);
        loading.value = false;
        if (view.viewMode === "column") {
          const tab = getTabStore().getFocusedTab();
          if (tab?.columnStack && tab.columnStack.length > 0) {
            tab.columnStack[0].files = files.value;
            tab.columnStack[0].loading = false;
          }
        }
        syncToTab();
      } catch (e2) {
        error.value = String(e2);
        loading.value = false;
      }
    } finally {
      if (cleanupListeners && _listUnlisten === cleanupListeners) {
        _listUnlisten = null;
      }
    }
  }

  async function navigateBack() {
    const nav = useNavigationStore();
    if (!nav.canGoBack) return;
    const target = nav.advanceBack();
    if (!target) return;
    if (typeof target === "string") {
      await navigateTo(target, false);
    } else {
      // Column snapshot
      currentPath.value = normalizePath(target.path);
      const tab = getTabStore().getFocusedTab();
      if (tab) {
        tab.columnStack = target.stack.map((c) => ({
          ...c,
          files: [...c.files],
        }));
      }
      files.value = target.stack.length > 0 ? [...target.stack[0].files] : [];
      useSelectionStore().clearSelection();
    }
  }

  async function navigateForward() {
    const nav = useNavigationStore();
    if (!nav.canGoForward) return;
    const target = nav.advanceForward();
    if (!target) return;
    if (typeof target === "string") {
      await navigateTo(target, false);
    } else {
      const snap = target;
      currentPath.value = normalizePath(snap.path);
      const tab = getTabStore().getFocusedTab();
      if (tab) {
        tab.columnStack = snap.stack.map((c) => ({
          ...c,
          files: [...c.files],
        }));
      }
      files.value = snap.stack.length > 0 ? [...snap.stack[0].files] : [];
      useSelectionStore().clearSelection();
    }
  }

  async function navigateUp() {
    const view = useViewStore();
    // Column view: trim rightmost column
    if (view.viewMode === "column") {
      const tab = getTabStore().getFocusedTab();
      if (tab?.columnStack && tab.columnStack.length > 1) {
        tab.columnStack.pop();
        const prev = tab.columnStack[tab.columnStack.length - 1];
        if (prev) prev.selectedIndex = -1;
      }
      return;
    }
    if (currentPath.value) {
      try {
        const parent = await tauri.getParentDirectory(currentPath.value);
        await navigateTo(parent);
      } catch (e) {
        console.error("navigateUp failed:", e);
      }
    }
  }

  async function navigateHome() {
    const sel = useSelectionStore();
    const view = useViewStore();
    currentPath.value = "";
    files.value = [];
    isSearching.value = false;
    sel.clearSelection();
    sel.resetCutState();
    view.resetTreeState();
    loading.value = false;
    error.value = "";
    await loadDrives();
  }

  async function refresh() {
    const view = useViewStore();
    if (view.viewMode === "column") {
      const tab = getTabStore().getFocusedTab();
      if (tab?.columnStack && tab.columnStack.length > 0) {
        const last = tab.columnStack[tab.columnStack.length - 1];
        if (last.path) {
          last.loading = true;
          try {
            last.files = sortDirFirst(await tauri.listDirectory(last.path));
          } catch (e) {
            console.error("column refresh failed:", e);
          }
          last.loading = false;
        }
      }
      return;
    }
    if (currentPath.value) {
      await navigateTo(currentPath.value, false);
    } else {
      await loadDrives();
    }
  }

  async function openDrive(drive: DiskInfo) {
    await navigateTo(drive.name);
  }

  // ── File operations ──
  async function createNewFolder(name: string) {
    const newPath = tauri.joinPath(currentPath.value, name);
    await tauri.createDirectory(newPath);
    await refresh();
  }

  async function createNewFile(name: string) {
    const newPath = tauri.joinPath(currentPath.value, name);
    await tauri.createFile(newPath);
    await refresh();
  }

  async function renameFile(oldPath: string, newName: string) {
    const sep = oldPath.includes("/") ? "/" : "\\";
    const lastSep = oldPath.lastIndexOf(sep);
    const parent = lastSep >= 0 ? oldPath.substring(0, lastSep) : "";
    const newPath = tauri.joinPath(parent, newName);
    await tauri.renameItem(oldPath, newPath);
    await refresh();
  }

  async function openSelectedFile(file: FileEntry) {
    if (file.is_dir) {
      await navigateTo(file.path);
    } else {
      await tauri.openFile(file.path);
    }
  }

  // ── Search ──
  async function cancelCurrentSearch() {
    await tauri.cancelSearch();
    isSearching.value = false;
  }

  // ── Undo ──
  async function performUndo(): Promise<string> {
    try {
      const msg = await tauri.undoLastAction();
      await refresh();
      await checkUndoStatus();
      return msg;
    } catch (e: any) {
      throw e;
    }
  }

  async function checkUndoStatus() {
    try {
      const info = await tauri.getUndoInfo();
      canUndo.value = info !== null;
      if (info) {
        switch (info.kind.type) {
          case "Rename":
            undoDescription.value = `Undo rename: ${info.kind.old_path}`;
            break;
          case "Create":
            undoDescription.value = `Undo create: ${info.kind.path}`;
            break;
          case "Copy":
            undoDescription.value = `Undo copy: ${info.kind.dest}`;
            break;
          case "Delete":
            undoDescription.value = "Recent delete (cannot undo)";
            break;
          default:
            undoDescription.value = "Undo last action";
        }
      } else {
        undoDescription.value = "";
      }
    } catch (e) {
      console.error("checkUndoStatus failed:", e);
      canUndo.value = false;
      undoDescription.value = "";
    }
  }

  // ── Drive polling (detect mount/unmount) ──
  let _driveTimer: ReturnType<typeof setInterval> | null = null;

  async function pollDrives() {
    try {
      const prev = drives.value
        .map((d) => d.mount_point)
        .sort()
        .join(",");
      const newDrives = await tauri.getDrives();
      const curr = newDrives
        .map((d) => d.mount_point)
        .sort()
        .join(",");
      if (prev !== curr) {
        drives.value = newDrives;
        specialDirs.value = await tauri.getSpecialDirs();
      }
    } catch {
      /* ignore */
    }
  }

  function startDrivePolling(intervalMs = 3000) {
    stopDrivePolling();
    _driveTimer = setInterval(pollDrives, intervalMs);
  }

  function stopDrivePolling() {
    if (_driveTimer) {
      clearInterval(_driveTimer);
      _driveTimer = null;
    }
  }

  // ── Recent files/folders ──
  const MAX_RECENT = 50;

  function loadRecentItems(): RecentItem[] {
    try {
      const raw = localStorage.getItem("app-recent-items");
      if (raw) return JSON.parse(raw);
    } catch {}
    return [];
  }

  function saveRecentItems() {
    localStorage.setItem("app-recent-items", JSON.stringify(recentItems.value));
  }

  function addRecentItem(
    path: string,
    name: string,
    isDir: boolean,
    ext: string,
  ) {
    const items = recentItems.value.filter((i) => i.path !== path);
    items.unshift({ path, name, isDir, ext, time: Date.now() });
    if (items.length > MAX_RECENT) items.length = MAX_RECENT;
    recentItems.value = items;
    saveRecentItems();
  }

  function removeRecentItem(path: string) {
    recentItems.value = recentItems.value.filter((i) => i.path !== path);
    saveRecentItems();
  }

  return {
    // State
    currentPath,
    files,
    drives,
    specialDirs,
    isSearching,
    loading,
    error,
    canUndo,
    undoDescription,
    // Computed
    currentDirectoryName,
    pathSegments,
    // Actions
    loadDrives,
    navigateTo,
    navigateBack,
    navigateForward,
    navigateUp,
    navigateHome,
    refresh,
    openDrive,
    createNewFolder,
    createNewFile,
    renameFile,
    openSelectedFile,
    cancelCurrentSearch,
    performUndo,
    checkUndoStatus,
    startDrivePolling,
    stopDrivePolling,
    recentItems,
    addRecentItem,
    removeRecentItem,
    syncToTab,
    loadFromTab,
  };
});
