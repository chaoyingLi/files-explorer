import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { FileEntry, DiskInfo, SpecialDirs } from "@/types";
import * as tauri from "@/utils/tauri";

export const useFileStore = defineStore("file", () => {
  const currentPath = ref("");
  const files = ref<FileEntry[]>([]);
  const drives = ref<DiskInfo[]>([]);
  const selectedFiles = ref<Set<string>>(new Set());
  const history = ref<string[]>([]);
  const historyIndex = ref(-1);
  const isSearching = ref(false);
  const loading = ref(false);
  const error = ref("");
  const viewMode = ref<"details" | "list" | "grid" | "tree">("details");
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

  async function navigateTo(path: string, addToHistory = true) {
    loading.value = true;
    error.value = "";
    // Cancel any in-progress search
    if (isSearching.value) {
      isSearching.value = false;
      cancelCurrentSearch();
    }
    cutFiles.value = new Set();
    isCutPending.value = false;
    treeExpanded.value = new Set();
    treeChildrenCache.value = new Map();

    try {
      files.value = await tauri.listDirectory(path);
      currentPath.value = path;

      if (addToHistory) {
        if (historyIndex.value < history.value.length - 1) {
          history.value = history.value.slice(0, historyIndex.value + 1);
        }
        history.value.push(path);
        historyIndex.value = history.value.length - 1;
      }

      selectedFiles.value.clear();
      // Keep max 50 history entries
      if (history.value.length > 50) {
        history.value = history.value.slice(-50);
        historyIndex.value = history.value.length - 1;
      }
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function navigateBack() {
    if (canGoBack.value) {
      historyIndex.value--;
      await navigateTo(history.value[historyIndex.value], false);
    }
  }

  async function navigateForward() {
    if (canGoForward.value) {
      historyIndex.value++;
      await navigateTo(history.value[historyIndex.value], false);
    }
  }

  async function navigateUp() {
    if (currentPath.value) {
      try {
        const parent = await tauri.getParentDirectory(currentPath.value);
        await navigateTo(parent);
      } catch (e) {
        // Already at root
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
    const newPath =
      currentPath.value +
      (currentPath.value.endsWith("/") || currentPath.value.endsWith("\\")
        ? ""
        : "/") +
      name;
    await tauri.createDirectory(newPath);
    await refresh();
  }

  async function createNewFile(name: string) {
    const newPath =
      currentPath.value +
      (currentPath.value.endsWith("/") || currentPath.value.endsWith("\\")
        ? ""
        : "/") +
      name;
    await tauri.createFile(newPath);
    await refresh();
  }

  // Show delete confirmation dialog
  function requestDelete(permanently = false) {
    if (selectedFiles.value.size === 0) return;
    deletePermanently.value = permanently;
    deleteTargetCount.value = selectedFiles.value.size;
    showDeleteConfirm.value = true;
  }

  // Execute delete after confirmation
  async function confirmDelete(): Promise<{
    success: number;
    failed: number;
    message: string;
  }> {
    showDeleteConfirm.value = false;
    const paths = [...selectedFiles.value];
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
    const parent = oldPath.substring(
      0,
      oldPath.lastIndexOf("/") !== -1
        ? oldPath.lastIndexOf("/")
        : oldPath.lastIndexOf("\\"),
    );
    const newPath = parent + "/" + newName;
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

  function setViewMode(mode: "details" | "list" | "grid" | "tree") {
    viewMode.value = mode;
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
      // Ignore errors (e.g., permission denied)
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
    const sorted = [...files.value].sort((a, b) => {
      if (a.is_dir && !b.is_dir) return -1;
      if (!a.is_dir && b.is_dir) return 1;
      return a.name.localeCompare(b.name, undefined, { sensitivity: "base" });
    });
    walk(sorted, 0);
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
          default:
            undoDescription.value = "Undo last action";
        }
      } else {
        undoDescription.value = "";
      }
    } catch {
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
  };
});
