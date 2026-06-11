import { onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import { useTabStore } from "@/stores/tabStore";

export function useKeyboardShortcuts(handlers: {
  onTabClose: (paneId: string, tabId: string) => void;
  onTabClick: (paneId: string, tabId: string) => void;
  showToast: (msg: string, isError?: boolean) => void;
  openNewDialog: (type: "file" | "folder") => void;
  openRenameDialog: (target: string) => void;
  openSettings: () => void;
}) {
  const { t } = useI18n();
  const store = useFileStore();
  const tabStore = useTabStore();

  function onKeydown(e: KeyboardEvent) {
    if (e.target instanceof HTMLInputElement) return;
    const ctrl = e.ctrlKey || e.metaKey;

    if (e.key === "Enter" && store.selectedFiles.size > 0) {
      e.preventDefault();
      const first = [...store.selectedFiles][0];
      const file = store.files.find((f) => f.path === first);
      if (file) store.openSelectedFile(file);
    } else if (e.key === "Escape" && store.isCutPending) {
      e.preventDefault();
      store.cancelCut();
      handlers.showToast("Cut cancelled");
    } else if (ctrl && e.key === "z") {
      e.preventDefault();
      store
        .performUndo()
        .then((msg) => handlers.showToast(msg))
        .catch((e) => handlers.showToast("Undo: " + e, true));
    } else if (ctrl && e.key === "w") {
      e.preventDefault();
      const fp = tabStore.getFocusedPane();
      if (fp) handlers.onTabClose(fp.id, fp.activeTabId);
    } else if (ctrl && e.key === "Tab" && !e.shiftKey) {
      e.preventDefault();
      const fp = tabStore.getFocusedPane();
      if (fp && fp.tabs.length > 1) {
        const idx = fp.tabs.findIndex((t) => t.id === fp.activeTabId);
        const next = (idx + 1) % fp.tabs.length;
        handlers.onTabClick(fp.id, fp.tabs[next].id);
      }
    } else if (ctrl && e.shiftKey && e.key === "Tab") {
      e.preventDefault();
      const fp = tabStore.getFocusedPane();
      if (fp && fp.tabs.length > 1) {
        const idx = fp.tabs.findIndex((t) => t.id === fp.activeTabId);
        const prev = (idx - 1 + fp.tabs.length) % fp.tabs.length;
        handlers.onTabClick(fp.id, fp.tabs[prev].id);
      }
    } else if (ctrl && e.key === "n") {
      e.preventDefault();
      handlers.openNewDialog("file");
    } else if (ctrl && e.shiftKey && e.key === "N") {
      e.preventDefault();
      handlers.openNewDialog("folder");
    } else if (ctrl && e.key === "c") {
      e.preventDefault();
      store.copySelected();
      handlers.showToast(t("toast.copied"));
    } else if (ctrl && e.key === "x") {
      e.preventDefault();
      store.cutSelected();
      handlers.showToast(t("toast.cut"));
    } else if (ctrl && e.key === "v") {
      e.preventDefault();
      store
        .paste()
        .then(() => handlers.showToast(t("toast.pasted")))
        .catch((e) => handlers.showToast(t("toast.error") + ": " + e));
    } else if (ctrl && e.key === "a") {
      e.preventDefault();
      store.selectAll();
    } else if (e.key === "Delete" && !e.shiftKey) {
      e.preventDefault();
      store.requestDelete(false);
    } else if (e.key === "Delete" && e.shiftKey) {
      e.preventDefault();
      store.requestDelete(true);
    } else if (e.key === "F2" && store.selectedFiles.size === 1) {
      e.preventDefault();
      const path = [...store.selectedFiles][0];
      const file = store.files.find((f) => f.path === path);
      if (file) handlers.openRenameDialog(file.name);
    } else if (e.key === "F5") {
      e.preventDefault();
      store.refresh();
    } else if (ctrl && e.key === ",") {
      e.preventDefault();
      handlers.openSettings();
    } else if (e.key === "Backspace" && !ctrl) {
      e.preventDefault();
      store.navigateUp();
    }
  }

  onMounted(() => {
    document.addEventListener("keydown", onKeydown);
  });

  onUnmounted(() => {
    document.removeEventListener("keydown", onKeydown);
  });
}
