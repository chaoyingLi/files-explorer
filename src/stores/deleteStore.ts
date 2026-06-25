import { defineStore } from "pinia";
import { ref } from "vue";
import * as tauri from "@/utils/tauri";

// Module-level snapshot (not reactive — captured when dialog opens)
let deleteTargetsSnapshot: string[] = [];

export const useDeleteStore = defineStore("delete", () => {
  const showDeleteConfirm = ref(false);
  const deletePermanently = ref(false);
  const deleteTargetCount = ref(0);

  function requestDelete(paths: string[], permanently = false) {
    if (paths.length === 0) return;
    deleteTargetsSnapshot = [...paths];
    deletePermanently.value = permanently;
    deleteTargetCount.value = deleteTargetsSnapshot.length;
    showDeleteConfirm.value = true;
  }

  function cancelDelete() {
    showDeleteConfirm.value = false;
  }

  async function confirmDelete(): Promise<{
    success: number;
    failed: number;
    message: string;
  }> {
    showDeleteConfirm.value = false;
    const paths = deleteTargetsSnapshot;
    deleteTargetsSnapshot = [];
    const result = await tauri.deleteItems(paths, deletePermanently.value);
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

  return {
    showDeleteConfirm,
    deletePermanently,
    deleteTargetCount,
    requestDelete,
    confirmDelete,
    cancelDelete,
  };
});
