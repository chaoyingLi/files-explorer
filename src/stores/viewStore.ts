import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { FileEntry } from "@/types";
import * as tauri from "@/utils/tauri";

// ── Column state (exported for component imports) ──
export interface ColumnState {
  path: string;
  name: string;
  files: FileEntry[];
  selectedIndex: number;
  loading: boolean;
}

export const useViewStore = defineStore("view", () => {
  const viewMode = ref<"details" | "list" | "grid" | "tree" | "column">(
    "details",
  );

  // ── Tree view state ──
  const treeExpanded = ref<Set<string>>(new Set());
  const treeChildrenCache = ref<Map<string, FileEntry[]>>(new Map());

  async function toggleTreeExpand(dirPath: string) {
    const expanded = new Set(treeExpanded.value);
    if (expanded.has(dirPath)) {
      expanded.delete(dirPath);
      const cache = new Map(treeChildrenCache.value);
      cache.delete(dirPath);
      treeExpanded.value = expanded;
      treeChildrenCache.value = cache;
      return;
    }
    try {
      const children = await tauri.listDirectory(dirPath);
      const sorted = [...children].sort((a, b) => {
        if (a.is_dir && !b.is_dir) return -1;
        if (!a.is_dir && b.is_dir) return 1;
        return a.name.localeCompare(b.name, undefined, { sensitivity: "base" });
      });
      const cache = new Map(treeChildrenCache.value);
      cache.set(dirPath, sorted);
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

  function resetTreeState() {
    treeExpanded.value = new Set();
    treeChildrenCache.value = new Map();
  }

  // ── Tree visible items (needs external files source) ──
  function computeTreeVisible(
    files: FileEntry[],
  ): { file: FileEntry; depth: number; expanded: boolean; hasChildren: boolean }[] {
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
          if (children) walk(children, depth + 1);
        }
      }
    }
    const sorted = [...files].sort((a, b) => {
      if (a.is_dir && !b.is_dir) return -1;
      if (!a.is_dir && b.is_dir) return 1;
      return a.name.localeCompare(b.name, undefined, { sensitivity: "base" });
    });
    walk(sorted, 0);
    return result;
  }

  // ── View mode ──

  function setViewMode(mode: "details" | "list" | "grid" | "tree" | "column") {
    viewMode.value = mode;
  }

  // ── Column view operations (accept stack param for per-tab isolation) ──

  async function columnLoadDirectory(
    stack: ColumnState[],
    colIdx: number,
    file: FileEntry,
  ): Promise<void> {
    const newCol: ColumnState = {
      path: file.path,
      name: file.name,
      files: [],
      selectedIndex: -1,
      loading: true,
    };
    stack.length = colIdx + 1;
    stack.push(newCol);
    try {
      const children = await tauri.listDirectory(file.path);
      const sorted = [...children].sort((a, b) => {
        if (a.is_dir && !b.is_dir) return -1;
        if (!a.is_dir && b.is_dir) return 1;
        return a.name.localeCompare(b.name, undefined, { sensitivity: "base" });
      });
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
      console.error("columnLoadDirectory failed:", e);
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

  return {
    viewMode,
    treeExpanded,
    treeChildrenCache,
    toggleTreeExpand,
    isTreeExpanded,
    getTreeChildren,
    setTreeExpanded,
    getTreeExpandedArray,
    collapseAllTree,
    resetTreeState,
    computeTreeVisible,
    setViewMode,
    columnLoadDirectory,
    columnNavigateLeft,
    columnNavigateUp,
    columnNavigateDown,
  };
});
