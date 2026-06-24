import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { FileEntry, DiskInfo, SpecialDirs } from "@/types";
import * as tauri from "@/utils/tauri";
import { useTabStore } from "@/stores/tabStore";

// Navigation token to cancel stale async operations
let navigateSeq = 0;

// Shared directory-first case-insensitive sort (used in 3 places)
function sortDirFirst(files: FileEntry[]): FileEntry[] {
  return [...files].sort((a, b) => {
    if (a.is_dir && !b.is_dir) return -1;
    if (!a.is_dir && b.is_dir) return 1;
    return a.name.localeCompare(b.name, undefined, { sensitivity: "base" });
  });
}

// ── Column state (exported outside store for component imports) ──
export interface ColumnState {
  path: string;
  name: string;
  files: FileEntry[];
  selectedIndex: number;
  loading: boolean;
}

export const useFileStore = defineStore("file", () => {
  // Lazy reference to avoid circular init
  let _tabStore: ReturnType<typeof useTabStore> | null = null;
  function getTabStore() {
    if (!_tabStore) _tabStore = useTabStore();
    return _tabStore;
  }

  function syncToTab() {
    const tab = getTabStore().getFocusedTab();
    if (!tab || tab.isSearch) return;
    tab.path = currentPath.value;
    tab.files = [...files.value];
    tab.selectedFiles =
      selectedFiles.value.size > 0 ? [...selectedFiles.value] : [];
    tab.treeExpanded =
      treeExpanded.value.size > 0 ? [...treeExpanded.value] : [];
  }

  function loadFromTab() {
    const tab = getTabStore().getFocusedTab();
    if (!tab) return;
    // Deep clone to break reference sharing with stored tab data
    files.value = [...(tab.files || [])];
    selectedFiles.value = new Set(tab.selectedFiles || []);
    currentPath.value = tab.path || "";
    if (tab.treeExpanded) treeExpanded.value = new Set(tab.treeExpanded);
  }

  const currentPath = ref("");
  const files = ref<FileEntry[]>([]);
  const drives = ref<DiskInfo[]>([]);
  const selectedFiles = ref<Set<string>>(new Set());
  // Column snapshot stored in history for column view mode
  interface ColumnHistoryEntry {
    path: string;
    stack: ColumnState[];
  }
  const history = ref<(string | ColumnHistoryEntry)[]>([]);
  const historyIndex = ref(-1);
  const isSearching = ref(false);
  const loading = ref(false);
  const error = ref("");
  const viewMode = ref<"details" | "list" | "grid" | "tree" | "column">(
    "details",
  );
  const contextMenuTarget = ref<FileEntry | null>(null);
  const specialDirs = ref<SpecialDirs | null>(null);

  // Cut state: tracks which files are marked for cut (shown as semi-transparent)
  const cutFiles = ref<Set<string>>(new Set());
  const isCutPending = ref(false);

  // Tree view state
  const treeExpanded = ref<Set<string>>(new Set());
  const treeChildrenCache = ref<Map<string, FileEntry[]>>(new Map());

  // Delete confirm dialog state
  const showDeleteConfirm = ref(false);
  const deletePermanently = ref(false);
  const deleteTargetCount = ref(0);
  // Snapshot of paths to delete (captured when dialog opens, immune to selection changes)
  let deleteTargetsSnapshot: string[] = [];

  // Undo state
  const canUndo = ref(false);
  const undoDescription = ref("");

  const canGoBack = computed(() => historyIndex.value > 0);
  const canGoForward = computed(
    () => historyIndex.value < history.value.length - 1,
  );

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
      accumulated = parts[0] + "\\";
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

  async function loadDrives() {
    try {
      drives.value = await tauri.getDrives();
      specialDirs.value = await tauri.getSpecialDirs();
    } catch (e) {
      error.value = String(e);
    }
  }

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
    cutFiles.value = new Set();
    isCutPending.value = false;
    treeExpanded.value = new Set();
    treeChildrenCache.value = new Map();
    files.value = [];
    currentPath.value = path;
    selectedFiles.value.clear();

    // If in column view, prepare single-column stack for this path
    if (viewMode.value === "column") {
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
      // Bug 10 fix: avoid duplicate history entries
      const rawLast =
        historyIndex.value >= 0 && historyIndex.value < history.value.length
          ? history.value[historyIndex.value]
          : undefined;
      const lastPath = typeof rawLast === "string" ? rawLast : rawLast?.path;
      if (path !== lastPath) {
        if (historyIndex.value < history.value.length - 1) {
          history.value = history.value.slice(0, historyIndex.value + 1);
        }
        history.value.push(path);
        historyIndex.value = history.value.length - 1;
        if (history.value.length > 50) {
          history.value = history.value.slice(-50);
          historyIndex.value = history.value.length - 1;
        }
      }
    }

    // Bug 4 fix: assign a navigation token to detect stale callbacks
    const navId = ++navigateSeq;

    let cleanupListeners: (() => void) | null = null;

    try {
      const { listen } = await import("@tauri-apps/api/event");
      let batchIndex = 0;
      let resolved = false;

      // Register ALL listeners BEFORE starting the stream (race condition fix)
      const unlistenProgress = await listen<FileEntry[]>(
        "list-progress",
        (event) => {
          // Bug 4: ignore stale callbacks from older navigation
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

      // Bug 3 fix: properly store all unlisten functions
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
          if (viewMode.value === "column") {
            const tab = getTabStore().getFocusedTab();
            if (tab?.columnStack && tab.columnStack.length > 0) {
              tab.columnStack[0].files = files.value;
              tab.columnStack[0].loading = false;
            }
          }
          syncToTab();
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

      // NOW start the stream
      await tauri.listDirectoryStreamed(path);
      await streamDone;
    } catch (_e) {
      // Fallback to sync (only for the latest navigation)
      if (navId !== navigateSeq) return;
      try {
        files.value = await tauri.listDirectory(path);
        loading.value = false;
        if (viewMode.value === "column") {
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
    if (canGoBack.value) {
      historyIndex.value--;
      historyIndex.value = Math.max(0, historyIndex.value);
      const target = history.value[historyIndex.value];
      if (typeof target === "string") {
        await navigateTo(target, false);
      } else {
        // Column snapshot: restore column stack from state object
        const snap = target as ColumnHistoryEntry;
        currentPath.value = snap.path;
        const tab = getTabStore().getFocusedTab();
        if (tab) {
          tab.columnStack = snap.stack.map((c) => ({
            ...c,
            files: [...c.files],
          }));
        }
        files.value = snap.stack.length > 0 ? [...snap.stack[0].files] : [];
        selectedFiles.value.clear();
      }
    }
  }

  async function navigateForward() {
    if (canGoForward.value) {
      historyIndex.value++;
      const target = history.value[historyIndex.value];
      if (typeof target === "string") {
        await navigateTo(target, false);
      } else {
        const snap = target as ColumnHistoryEntry;
        currentPath.value = snap.path;
        const tab = getTabStore().getFocusedTab();
        if (tab) {
          tab.columnStack = snap.stack.map((c) => ({
            ...c,
            files: [...c.files],
          }));
        }
        files.value = snap.stack.length > 0 ? [...snap.stack[0].files] : [];
        selectedFiles.value.clear();
      }
    }
  }

  async function navigateUp() {
    // In column view: trim the rightmost column on the active tab
    if (viewMode.value === "column") {
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
    currentPath.value = "";
    files.value = [];
    isSearching.value = false;
    selectedFiles.value.clear();
    cutFiles.value = new Set();
    isCutPending.value = false;
    loading.value = false;
    error.value = "";
    await loadDrives();
  }

  async function refresh() {
    // In column view: re-read last column files from active tab
    if (viewMode.value === "column") {
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

  function toggleSelectFile(file: FileEntry) {
    const newSet = new Set(selectedFiles.value);
    if (newSet.has(file.path)) {
      newSet.delete(file.path);
    } else {
      newSet.add(file.path);
    }
    selectedFiles.value = newSet;
  }

  function selectFile(file: FileEntry, multi = false) {
    if (!multi) {
      selectedFiles.value = new Set([file.path]);
    } else {
      toggleSelectFile(file);
    }
  }

  function selectAll() {
    selectedFiles.value = new Set(files.value.map((f) => f.path));
  }

  function clearSelection() {
    selectedFiles.value = new Set();
  }

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

  // Show delete confirmation dialog
  function requestDelete(permanently = false) {
    if (selectedFiles.value.size === 0) return;
    // Bug 8 fix: snapshot selection immediately, before dialog closes
    deleteTargetsSnapshot = [...selectedFiles.value];
    deletePermanently.value = permanently;
    deleteTargetCount.value = deleteTargetsSnapshot.length;
    showDeleteConfirm.value = true;
  }

  // Execute delete after confirmation
  async function confirmDelete(): Promise<{
    success: number;
    failed: number;
    message: string;
  }> {
    showDeleteConfirm.value = false;
    // Bug 8 fix: use snapshot rather than live selection
    const paths = deleteTargetsSnapshot;
    deleteTargetsSnapshot = [];
    const result = await tauri.deleteItems(paths, deletePermanently.value);
    await refresh();
    if (result.failed.length > 0) {
      return {
        success: result.success.length,
        failed: result.failed.length,
        message: `Deleted ${result.success.length} items, ${result.failed.length} failed`,
      };
    }
    return {
      success: result.success.length,
      failed: 0,
      message: `Deleted ${result.success.length} items`,
    };
  }

  function cancelDelete() {
    showDeleteConfirm.value = false;
  }

  async function deleteSelected(permanently = false) {
    // Old direct delete - now we use requestDelete + confirmDelete
    requestDelete(permanently);
  }

  async function renameFile(oldPath: string, newName: string) {
    // Bug 1 fix: use the same separator style as the original path
    const sep = oldPath.includes("/") ? "/" : "\\";
    const lastSep = oldPath.lastIndexOf(sep);
    const parent = lastSep >= 0 ? oldPath.substring(0, lastSep) : "";
    const newPath = parent + sep + newName;
    await tauri.renameItem(oldPath, newPath);
    await refresh();
  }

  async function copySelected() {
    if (selectedFiles.value.size === 0) return;
    await tauri.copyClipboard([...selectedFiles.value]);
    // Clear cut state on copy
    cutFiles.value = new Set();
    isCutPending.value = false;
  }

  async function cutSelected() {
    if (selectedFiles.value.size === 0) return;
    await tauri.cutClipboard([...selectedFiles.value]);
    // Mark these files as cut for visual feedback
    cutFiles.value = new Set(selectedFiles.value);
    isCutPending.value = true;
    selectedFiles.value = new Set();
  }

  function cancelCut() {
    cutFiles.value = new Set();
    isCutPending.value = false;
  }

  async function paste() {
    await tauri.pasteClipboard(currentPath.value);
    cutFiles.value = new Set();
    isCutPending.value = false;
    await refresh();
  }

  async function openSelectedFile(file: FileEntry) {
    if (file.is_dir) {
      await navigateTo(file.path);
    } else {
      await tauri.openFile(file.path);
    }
  }

  async function cancelCurrentSearch() {
    await tauri.cancelSearch();
    isSearching.value = false;
  }

  async function openDrive(drive: DiskInfo) {
    await navigateTo(drive.name);
  }

  function setViewMode(mode: "details" | "list" | "grid" | "tree" | "column") {
    viewMode.value = mode;
    if (mode === "column") {
      const tab = getTabStore().getFocusedTab();
      if (tab && !tab.columnStack) {
        // Initialize column from current path
        tab.columnStack = currentPath.value
          ? [
              {
                path: currentPath.value,
                name: currentDirectoryName.value,
                files: [...files.value],
                selectedIndex: -1,
                loading: false,
              },
            ]
          : [];
      }
    }
  }

  // ── Column view operations (accept stack param for per-tab isolation) ──
  async function columnSelect(
    stack: ColumnState[],
    colIdx: number,
    idx: number,
  ) {
    const col = stack[colIdx];
    if (!col || idx < 0 || idx >= col.files.length) return;
    col.selectedIndex = idx;
    const file = col.files[idx];
    if (file.is_dir) {
      // Save current state to history
      if (stack.length >= 1) {
        const lastCol = stack[stack.length - 1];
        const histEntry: ColumnHistoryEntry = {
          path: lastCol.path || currentPath.value,
          stack: stack.map((c) => ({ ...c, files: [...c.files] })),
        };
        if (historyIndex.value < history.value.length - 1) {
          history.value = history.value.slice(0, historyIndex.value + 1);
        }
        history.value.push(histEntry);
        historyIndex.value = history.value.length - 1;
        if (history.value.length > 50) {
          history.value = history.value.slice(-50);
          historyIndex.value = history.value.length - 1;
        }
      }
      // Append new column
      const newCol: ColumnState = {
        path: file.path,
        name: file.name,
        files: [],
        selectedIndex: -1,
        loading: true,
      };
      // Mutate in-place: trim + append
      stack.length = colIdx + 1;
      stack.push(newCol);
      try {
        const children = await tauri.listDirectory(file.path);
        const sorted = sortDirFirst(children);
        const updated = stack[colIdx + 1];
        if (updated) {
          updated.files = sorted;
          updated.loading = false;
        }
      } catch (e) {
        const updated = stack[colIdx + 1];
        if (updated) {
          updated.files = [];
          updated.loading = false;
        }
        console.error("columnSelect failed:", e);
      }
    } else {
      openSelectedFile(file);
    }
  }

  function columnNavigateLeft(stack: ColumnState[]) {
    if (stack.length <= 1) return;
    const last = stack[stack.length - 1];
    const prev = stack[stack.length - 2];
    prev.selectedIndex = prev.files.findIndex((f) => f.path === last.path);
    stack.pop();
  }

  function columnNavigateUp(stack: ColumnState[], colIdx: number) {
    const col = stack[colIdx];
    if (!col) return;
    if (col.selectedIndex > 0) col.selectedIndex--;
  }

  function columnNavigateDown(stack: ColumnState[], colIdx: number) {
    const col = stack[colIdx];
    if (!col) return;
    if (col.selectedIndex < col.files.length - 1) col.selectedIndex++;
  }

  // ── Tree view operations ──
  async function toggleTreeExpand(dirPath: string) {
    const expanded = new Set(treeExpanded.value);
    if (expanded.has(dirPath)) {
      // Collapse: remove this and all descendants
      expanded.delete(dirPath);
      const cache = new Map(treeChildrenCache.value);
      cache.delete(dirPath);
      treeExpanded.value = expanded;
      treeChildrenCache.value = cache;
      return;
    }
    // Expand: fetch children
    try {
      const children = await tauri.listDirectory(dirPath);
      const cache = new Map(treeChildrenCache.value);
      cache.set(dirPath, children);
      expanded.add(dirPath);
      treeExpanded.value = expanded;
      treeChildrenCache.value = cache;
    } catch (e) {
      console.error("toggleTreeExpand failed for", dirPath, e);
    }
  }

  function isTreeExpanded(dirPath: string): boolean {
    return treeExpanded.value.has(dirPath);
  }

  function getTreeChildren(dirPath: string): FileEntry[] {
    return treeChildrenCache.value.get(dirPath) || [];
  }

  function setTreeExpanded(paths: string[]) {
    treeExpanded.value = new Set(paths);
  }

  function getTreeExpandedArray(): string[] {
    return [...treeExpanded.value];
  }

  function collapseAllTree() {
    treeExpanded.value = new Set();
    treeChildrenCache.value = new Map();
  }

  function isFileCut(path: string): boolean {
    return cutFiles.value.has(path);
  }

  // ── Tree view visible items computation ──
  const treeVisibleItems = computed(() => {
    const result: {
      file: FileEntry;
      depth: number;
      expanded: boolean;
      hasChildren: boolean;
    }[] = [];
    function walk(items: FileEntry[], depth: number) {
      for (const item of items) {
        const expanded = item.is_dir && treeExpanded.value.has(item.path);
        const hasChildren = item.is_dir;
        result.push({ file: item, depth, expanded, hasChildren });
        if (expanded) {
          const children = treeChildrenCache.value.get(item.path);
          if (children) {
            walk(children, depth + 1);
          }
        }
      }
    }
    // Sort: directories first, then alphabetical
    walk(sortDirFirst(files.value), 0);
    return result;
  });

  // Undo last action
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

  return {
    currentPath,
    files,
    drives,
    selectedFiles,
    history,
    historyIndex,
    isSearching,
    loading,
    error,
    viewMode,
    contextMenuTarget,
    specialDirs,
    cutFiles,
    isCutPending,
    showDeleteConfirm,
    deletePermanently,
    deleteTargetCount,
    canUndo,
    undoDescription,
    canGoBack,
    canGoForward,
    currentDirectoryName,
    pathSegments,
    loadDrives,
    navigateTo,
    navigateBack,
    navigateForward,
    navigateUp,
    navigateHome,
    refresh,
    toggleSelectFile,
    selectFile,
    selectAll,
    clearSelection,
    createNewFolder,
    createNewFile,
    deleteSelected,
    requestDelete,
    confirmDelete,
    cancelDelete,
    renameFile,
    copySelected,
    cutSelected,
    cancelCut,
    paste,
    openSelectedFile,
    setViewMode,
    openDrive,
    isFileCut,
    treeVisibleItems,
    toggleTreeExpand,
    isTreeExpanded,
    getTreeChildren,
    setTreeExpanded,
    getTreeExpandedArray,
    collapseAllTree,
    cancelCurrentSearch,
    performUndo,
    checkUndoStatus,
    syncToTab,
    loadFromTab,
    // Column view
    columnSelect,
    columnNavigateLeft,
    columnNavigateUp,
    columnNavigateDown,
  };
});
