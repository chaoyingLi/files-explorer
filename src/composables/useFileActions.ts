import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import { useTabStore, type Tab } from "@/stores/tabStore";
import * as tauri from "@/utils/tauri";

export function useFileActions(
  toast: ReturnType<typeof import("./useToast").useToast>,
) {
  const { t } = useI18n();
  const store = useFileStore();
  const tabStore = useTabStore();

  const showNewDialog = ref(false);
  const newDialogType = ref<"folder" | "file">("folder");
  const showRenameDialog = ref(false);
  const renameTarget = ref("");

  function saveFileStateToTab(tab: Tab) {
    store.syncToTab();
    // Update tab title separately (not in fileStore)
    tab.title = store.currentDirectoryName || t("sidebar.thisPc");
  }

  function loadFileStateFromTab(tab: Tab) {
    if (
      tab.isSearch &&
      tab.files &&
      tab.files.length > 0 &&
      store.files.length === 0
    ) {
      store.files = tab.files;
    }
    store.loadFromTab();
  }

  async function handleNewItem(name: string, type: string) {
    if (type === "folder") {
      await store.createNewFolder(name);
    } else {
      await store.createNewFile(name);
    }
    showNewDialog.value = false;
  }

  async function handleDeleteConfirm(permanently: boolean) {
    store.deletePermanently = permanently;
    try {
      const result = await store.confirmDelete();
      if (result.failed > 0) {
        toast.show(`${result.message}`, true);
      } else {
        toast.show(t("toast.deleted"));
      }
    } catch (e: any) {
      toast.show(t("toast.error") + ": " + e, true);
    }
  }

  async function handleRename(newName: string) {
    if (store.selectedFiles.size === 1) {
      const oldPath = [...store.selectedFiles][0];
      await store.renameFile(oldPath, newName);
    }
    showRenameDialog.value = false;
  }

  async function executeAction(action: string) {
    switch (action) {
      case "newFolder":
        newDialogType.value = "folder";
        showNewDialog.value = true;
        break;
      case "newFile":
        newDialogType.value = "file";
        showNewDialog.value = true;
        break;
      case "open":
        if (store.selectedFiles.size > 0) {
          const first = [...store.selectedFiles][0];
          const file = store.files.find((f) => f.path === first);
          if (file) await store.openSelectedFile(file);
        }
        break;
      case "cut":
        await store.cutSelected();
        toast.show(t("toast.cut"));
        break;
      case "copy":
        await store.copySelected();
        toast.show(t("toast.copied"));
        break;
      case "paste":
        try {
          await store.paste();
          toast.show(t("toast.pasted"));
        } catch (e: any) {
          toast.show(t("toast.error") + ": " + e, true);
        }
        break;
      case "delete":
        store.requestDelete(false);
        break;
      case "deletePermanent":
        store.requestDelete(true);
        break;
      case "rename":
        if (store.selectedFiles.size === 1) {
          const path = [...store.selectedFiles][0];
          const file = store.files.find((f) => f.path === path);
          if (file) {
            renameTarget.value = file.name;
            showRenameDialog.value = true;
          }
        }
        break;
      case "selectAll":
        store.selectAll();
        break;
      case "refresh":
        await store.refresh();
        break;
      case "openInTerminal":
        try {
          const target =
            store.selectedFiles.size === 1
              ? [...store.selectedFiles][0]
              : store.currentPath;
          await tauri.openInTerminal(target);
        } catch (e: any) {
          toast.show(t("toast.error") + ": " + e, true);
        }
        break;
      case "properties":
        if (store.selectedFiles.size === 1) {
          const path = [...store.selectedFiles][0];
          console.log("properties: opening for", path);
          try {
            await tauri.showFileProperties(path);
          } catch (e: any) {
            toast.show(t("toast.error") + ": " + e, true);
          }
        }
        break;
      case "showInExplorer":
        {
          const path =
            store.selectedFiles.size === 1
              ? [...store.selectedFiles][0]
              : store.currentPath;
          if (path) {
            try {
              await tauri.showInExplorer(path);
            } catch (e: any) {
              toast.show(t("toast.error") + ": " + e, true);
            }
          }
        }
        break;
    }
  }

  return {
    showNewDialog,
    newDialogType,
    showRenameDialog,
    renameTarget,
    saveFileStateToTab,
    loadFileStateFromTab,
    handleNewItem,
    handleDeleteConfirm,
    handleRename,
    executeAction,
  };
}
