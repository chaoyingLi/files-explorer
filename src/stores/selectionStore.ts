import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { FileEntry } from "@/types";
import * as tauri from "@/utils/tauri";

export const useSelectionStore = defineStore("selection", () => {
  const selectedFiles = ref<Set<string>>(new Set());
  const cutFiles = ref<Set<string>>(new Set());
  const isCutPending = ref(false);

  // ── Keyboard navigation state ──
  const focusedIndex = ref(0);
  const anchorIndex = ref(0);

  function setFocus(idx: number, files: FileEntry[], select = true) {
    const clamped = Math.max(0, Math.min(idx, files.length - 1));
    focusedIndex.value = clamped;
    anchorIndex.value = clamped;
    if (select && files[clamped]) {
      selectFile(files[clamped], false);
    }
  }

  function moveFocus(
    delta: number,
    files: FileEntry[],
    shiftKey = false,
    ctrlKey = false,
  ) {
    const filesLen = files.length;
    if (filesLen === 0) return;
    const newIdx = Math.max(
      0,
      Math.min(focusedIndex.value + delta, filesLen - 1),
    );
    if (ctrlKey) {
      // Ctrl+Arrow: move focus without changing selection
      focusedIndex.value = newIdx;
      return;
    }
    if (shiftKey) {
      // Shift+Arrow: extend selection range
      focusedIndex.value = newIdx;
      const from = Math.min(anchorIndex.value, focusedIndex.value);
      const to = Math.max(anchorIndex.value, focusedIndex.value);
      const newSet = new Set<string>();
      for (let i = from; i <= to; i++) {
        if (files[i]) newSet.add(files[i].path);
      }
      selectedFiles.value = newSet;
      return;
    }
    // Plain Arrow: move and select single
    focusedIndex.value = newIdx;
    anchorIndex.value = newIdx;
    if (files[newIdx]) {
      selectFile(files[newIdx], false);
    }
  }

  function moveToEdge(
    dir: "first" | "last",
    files: FileEntry[],
    shiftKey = false,
  ) {
    const target = dir === "first" ? 0 : files.length - 1;
    if (shiftKey) {
      focusedIndex.value = target;
      const from = Math.min(anchorIndex.value, focusedIndex.value);
      const to = Math.max(anchorIndex.value, focusedIndex.value);
      const newSet = new Set<string>();
      for (let i = from; i <= to; i++) {
        if (files[i]) newSet.add(files[i].path);
      }
      selectedFiles.value = newSet;
    } else {
      setFocus(target, files, true);
    }
  }

  function movePage(
    dir: 1 | -1,
    files: FileEntry[],
    pageSize: number,
    shiftKey = false,
  ) {
    moveFocus(dir * pageSize, files, shiftKey, false);
  }

  function toggleFocusSelection(files: FileEntry[]) {
    const f = files[focusedIndex.value];
    if (!f) return;
    toggleSelectFile(f);
    anchorIndex.value = focusedIndex.value;
  }

  // ── File selection ──

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

  /** Shift+Click 范围选择: 从 anchor 到 target 全选 */
  function selectRange(files: FileEntry[], fromIdx: number, toIdx: number) {
    const start = Math.min(fromIdx, toIdx);
    const end = Math.max(fromIdx, toIdx);
    const newSet = new Set<string>();
    for (let i = start; i <= end; i++) {
      if (files[i]) newSet.add(files[i].path);
    }
    selectedFiles.value = newSet;
  }

  function selectAll(files: FileEntry[]) {
    selectedFiles.value = new Set(files.map((f) => f.path));
  }

  function clearSelection() {
    selectedFiles.value = new Set();
  }

  function isSelected(path: string): boolean {
    return selectedFiles.value.has(path);
  }

  function selectionCount(): number {
    return selectedFiles.value.size;
  }

  // ── Cut / Copy / Paste ──

  async function copySelected() {
    if (selectedFiles.value.size === 0) return;
    await tauri.copyClipboard([...selectedFiles.value]);
    cutFiles.value = new Set();
    isCutPending.value = false;
  }

  async function cutSelected() {
    if (selectedFiles.value.size === 0) return;
    await tauri.cutClipboard([...selectedFiles.value]);
    cutFiles.value = new Set(selectedFiles.value);
    isCutPending.value = true;
    selectedFiles.value = new Set();
  }

  function cancelCut() {
    cutFiles.value = new Set();
    isCutPending.value = false;
  }

  function isFileCut(path: string): boolean {
    return cutFiles.value.has(path);
  }

  async function paste(currentPath: string): Promise<void> {
    await tauri.pasteClipboard(currentPath);
    cutFiles.value = new Set();
    isCutPending.value = false;
  }

  // ── Reset (called on navigation) ──

  function resetCutState() {
    cutFiles.value = new Set();
    isCutPending.value = false;
  }

  return {
    selectedFiles,
    cutFiles,
    isCutPending,
    focusedIndex,
    anchorIndex,
    toggleSelectFile,
    selectFile,
    selectRange,
    selectAll,
    clearSelection,
    isSelected,
    selectionCount,
    copySelected,
    cutSelected,
    cancelCut,
    isFileCut,
    paste,
    resetCutState,
    setFocus,
    moveFocus,
    moveToEdge,
    movePage,
    toggleFocusSelection,
  };
});
