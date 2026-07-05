import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import { useSelectionStore } from "@/stores/selectionStore";
import { useDeleteStore } from "@/stores/deleteStore";
import { useSettingsStore } from "@/stores/settingsStore";
import { useTabStore, type Tab } from "@/stores/tabStore";
import * as tauri from "@/utils/tauri";

export function useFileActions(
  toast: ReturnType<typeof import("./useToast").useToast>,
  toggleProperties: () => void,
) {
  const { t } = useI18n();
  const store = useFileStore();
  const sel = useSelectionStore();
  const del = useDeleteStore();
  const tabStore = useTabStore();

  const showNewDialog = ref(false);
  const newDialogType = ref<"folder" | "file">("folder");
  const showRenameDialog = ref(false);
  const renameTarget = ref("");

  function saveFileStateToTab(tab: Tab) {
    store.syncToTab();
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
    try {
      const result = await del.confirmDelete();
      await store.refresh();
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
    if (sel.selectedFiles.size === 1) {
      const oldPath = [...sel.selectedFiles][0];
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
        if (sel.selectedFiles.size > 0) {
          const first = [...sel.selectedFiles][0];
          const file = store.files.find((f) => f.path === first);
          if (file) await store.openSelectedFile(file);
        }
        break;
      case "cut":
        await sel.cutSelected();
        toast.show(t("toast.cut"));
        break;
      case "copy":
        await sel.copySelected();
        toast.show(t("toast.copied"));
        break;
      case "paste":
        try {
          await sel.paste(store.currentPath);
          await store.refresh();
          toast.show(t("toast.pasted"));
        } catch (e: any) {
          toast.show(t("toast.error") + ": " + e, true);
        }
        break;
      case "copyPath":
        try {
          const isWin = /Win/.test(navigator.platform);
          const paths = [...sel.selectedFiles].map((p) =>
            isWin ? p.replace(/\//g, "\\") : p.replace(/\\/g, "/"),
          );
          await navigator.clipboard.writeText(paths.join("\n"));
          toast.show(t("toast.copied"));
        } catch (e: any) {
          toast.show(t("toast.error") + ": " + e, true);
        }
        break;
      case "delete":
        del.requestDelete([...sel.selectedFiles], false);
        break;
      case "deletePermanent":
        del.requestDelete([...sel.selectedFiles], true);
        break;
      case "rename":
        if (sel.selectedFiles.size === 1) {
          const path = [...sel.selectedFiles][0];
          const file = store.files.find((f) => f.path === path);
          if (file) {
            renameTarget.value = file.name;
            showRenameDialog.value = true;
          }
        }
        break;
      case "selectAll":
        sel.selectAll(store.files);
        break;
      case "refresh":
        await store.refresh();
        break;
      case "openInTerminal":
        try {
          const target =
            sel.selectedFiles.size === 1
              ? [...sel.selectedFiles][0]
              : store.currentPath;
          await tauri.openInTerminal(target);
        } catch (e: any) {
          toast.show(t("toast.error") + ": " + e, true);
        }
        break;
      case "properties":
        if (sel.selectedFiles.size === 1) {
          const path = [...sel.selectedFiles][0];
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
            sel.selectedFiles.size === 1
              ? [...sel.selectedFiles][0]
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
      case "compress":
        if (sel.selectedFiles.size > 0) {
          const paths = [...sel.selectedFiles];
          try {
            const { save } = await import("@tauri-apps/plugin-dialog");
            const filePath = await save({
              defaultPath: "archive.zip",
              filters: [{ name: "Zip Archive", extensions: ["zip"] }],
            });
            if (filePath) {
              await tauri.compressFiles(paths, filePath);
              toast.show(t("toast.compressed"));
              await store.refresh();
            }
          } catch (e: any) {
            toast.show(t("toast.error") + ": " + e, true);
          }
        }
        break;
      case "toggleProperties":
        toggleProperties();
        break;
      case "addToFavorites":
        if (sel.selectedFiles.size === 1) {
          const path = [...sel.selectedFiles][0];
          const file = store.files.find((f) => f.path === path);
          if (file?.is_dir) {
            const settings = useSettingsStore();
            if (settings.hasBookmark(path)) {
              settings.removeBookmark(path);
              toast.show(t("sidebar.removeBookmark", { label: file.name }));
            } else {
              settings.addBookmark(path, file.name);
              toast.show(t("sidebar.addToFavorites"));
            }
          }
        }
        break;
      case "extract":
        if (sel.selectedFiles.size === 1) {
          const archivePath = [...sel.selectedFiles][0];
          try {
            const base =
              archivePath
                .replace(/\\/g, "/")
                .split("/")
                .slice(0, -1)
                .join("/") || ".";
            const name =
              archivePath
                .replace(/\\/g, "/")
                .split("/")
                .pop()
                ?.replace(/\.[^.]+$/, "") || "extracted";
            const destDir = base + "/" + name;
            await tauri.extractArchive(archivePath, destDir);
            toast.show(t("toast.extracted"));
            await store.refresh();
          } catch (e: any) {
            toast.show(t("toast.error") + ": " + e, true);
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
