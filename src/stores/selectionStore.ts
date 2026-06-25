import { defineStore } from "pinia";
import { ref } from "vue";
import type { FileEntry } from "@/types";
import * as tauri from "@/utils/tauri";

export const useSelectionStore = defineStore("selection", () => {
  const selectedFiles = ref<Set<string>>(new Set());
  const cutFiles = ref<Set<string>>(new Set());
  const isCutPending = ref(false);

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
    toggleSelectFile,
    selectFile,
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
  };
});
