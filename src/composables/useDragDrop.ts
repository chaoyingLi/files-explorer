import { useFileStore } from "@/stores/fileStore";
import { useTabStore, type Tab } from "@/stores/tabStore";
import * as tauri from "@/utils/tauri";

export function useDragDrop(
  showToast: (msg: string, isError?: boolean) => void,
  tFn: (key: string) => string,
) {
  const store = useFileStore();
  const tabStore = useTabStore();

  async function handleFileDrop(
    _paneId: string,
    dir: string,
    paths: string[],
    ctrl: boolean,
  ) {
    try {
      await tauri.moveFiles(paths, dir, ctrl);
      showToast(ctrl ? tFn("toast.copied") : tFn("toast.moved"));
      await store.refresh();
    } catch (e: any) {
      showToast(tFn("toast.error") + ": " + e, true);
    }
  }

  async function onTabDrop(paneId: string, tabId: string, e: DragEvent) {
    e.preventDefault();
    tabStore.onTabDragLeave(tabId);
    const raw = e.dataTransfer?.getData("text/plain");
    if (!raw) return;
    const paths = raw.split("\n").filter(Boolean);
    const tp = tabStore.findPaneById(paneId);
    if (!tp) return;
    const tt =
      tp.tabs.find((t: Tab) => t.id === tabId) ||
      tp.tabs.find((t: Tab) => t.id === tp.activeTabId);
    if (!tt?.path) return;
    for (const sp of paths) {
      const dp = tauri.joinPath(
        tt.path,
        sp.replace(/\\/g, "/").split("/").pop() || "",
      );
      if (sp !== dp) {
        try {
          await tauri.renameItem(sp, dp);
        } catch (_) {}
      }
    }
    store.navigateTo(tt.path, false);
  }

  return { handleFileDrop, onTabDrop };
}
