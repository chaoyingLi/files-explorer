import { onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import { useSelectionStore } from "@/stores/selectionStore";
import { useDeleteStore } from "@/stores/deleteStore";
import { useTabStore } from "@/stores/tabStore";

export function useKeyboardShortcuts(handlers: {
  onTabClose: (paneId: string, tabId: string) => void;
  onTabClick: (paneId: string, tabId: string) => void;
  showToast: (msg: string, isError?: boolean) => void;
  openNewDialog: (type: "file" | "folder") => void;
  openRenameDialog: (target: string) => void;
  openSettings: () => void;
  openProperties: () => void;
}) {
  const { t } = useI18n();
  const store = useFileStore();
  const sel = useSelectionStore();
  const del = useDeleteStore();
  const tabStore = useTabStore();

  function onKeydown(e: KeyboardEvent) {
    if (e.target instanceof HTMLInputElement) return;
    const ctrl = e.ctrlKey || e.metaKey;

    if (e.key === "Enter" && sel.selectedFiles.size > 0) {
      e.preventDefault();
      const first = [...sel.selectedFiles][0];
      const file = store.files.find((f) => f.path === first);
      if (file) store.openSelectedFile(file);
    } else if (e.key === "Escape" && sel.isCutPending) {
      e.preventDefault();
      sel.cancelCut();
      handlers.showToast(t("toast.cutCancelled"));
    } else if (
      e.key === " " &&
      !ctrl &&
      store.files.length > 0 &&
      sel.selectedFiles.size > 0
    ) {
    } else if (
      e.key === " " &&
      !ctrl &&
      store.files.length > 0 &&
      sel.selectedFiles.size > 0
    ) {
      // Space: preview / open selected file (handled by FileList for focused navigation)
      // Keep as fallback for when FileList is not focused
      e.preventDefault();
      const first = [...sel.selectedFiles][0];
      const file = store.files.find((f) => f.path === first);
      if (file) store.openSelectedFile(file);
    } else if (ctrl && (e.key === "ArrowDown" || e.key === "ArrowRight")) {
      // Cmd+↓ / Cmd+→ : open selected item (keep for non-FileList focus)
      e.preventDefault();
      if (sel.selectedFiles.size > 0) {
        const first = [...sel.selectedFiles][0];
        const file = store.files.find((f) => f.path === first);
        if (file) store.openSelectedFile(file);
      }
    } else if (ctrl && (e.key === "ArrowUp" || e.key === "ArrowLeft")) {
      // Cmd+↑ / Cmd+← : go to parent directory
      e.preventDefault();
      store.navigateUp();
    } else if (ctrl && e.key === "[") {
      // Cmd+[ : navigate back
      e.preventDefault();
      store.navigateBack();
    } else if (ctrl && e.key === "]") {
      // Cmd+] : navigate forward
      e.preventDefault();
      store.navigateForward();
    } else if (ctrl && e.key === "z") {
      e.preventDefault();
      store
        .performUndo()
        .then((msg) => handlers.showToast(msg))
        .catch((e) =>
          handlers.showToast(t("toast.undoFailed", { error: String(e) }), true),
        );
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
      // If user has selected text (e.g. in Markdown preview), let browser handle copy
      if (window.getSelection()?.toString().trim()) {
        return;
      }
      e.preventDefault();
      sel.copySelected();
      handlers.showToast(t("toast.copied"));
    } else if (ctrl && e.key === "x") {
      if (window.getSelection()?.toString().trim()) {
        return;
      }
      e.preventDefault();
      sel.cutSelected();
      handlers.showToast(t("toast.cut"));
    } else if (ctrl && e.key === "v") {
      e.preventDefault();
      sel
        .paste(store.currentPath)
        .then(async () => {
          await store.refresh();
          handlers.showToast(t("toast.pasted"));
        })
        .catch((e) => handlers.showToast(t("toast.error") + ": " + e, true));
    } else if (ctrl && e.key === "a") {
      e.preventDefault();
      sel.selectAll(store.files);
    } else if (e.key === "Delete" && !e.shiftKey) {
      e.preventDefault();
      del.requestDelete([...sel.selectedFiles], false);
    } else if (e.key === "Delete" && e.shiftKey) {
      e.preventDefault();
      del.requestDelete([...sel.selectedFiles], true);
    } else if (e.key === "F2" && sel.selectedFiles.size === 1) {
      e.preventDefault();
      const path = [...sel.selectedFiles][0];
      const file = store.files.find((f) => f.path === path);
      if (file) handlers.openRenameDialog(file.name);
    } else if (e.key === "F5") {
      e.preventDefault();
      store.refresh();
    } else if (ctrl && e.key === "p") {
      e.preventDefault();
      handlers.openProperties();
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
