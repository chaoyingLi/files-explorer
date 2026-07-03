import { ref } from "vue";
import { useFileStore } from "@/stores/fileStore";
import { useTabStore, type Tab } from "@/stores/tabStore";
import * as tauri from "@/utils/tauri";

export function usePanelNavigation(
  saveFileStateToTab: (tab: Tab) => void,
  loadFileStateFromTab: (tab: Tab) => void,
  tFn: (key: string) => string,
) {
  const store = useFileStore();
  const tabStore = useTabStore();
  const focusedPaneId = ref("");

  function onPaneFocus(paneId: string) {
    const ot = tabStore.getFocusedTab();
    if (ot) saveFileStateToTab(ot);
    tabStore.focusPane(paneId);
    focusedPaneId.value = paneId;
    const p = tabStore.findPaneById(paneId);
    if (p) {
      const t = p.tabs.find((x: Tab) => x.id === p.activeTabId);
      if (t) {
        loadFileStateFromTab(t);
        if (!t.path) store.loadDrives();
      }
    }
  }

  async function onTabClick(
    paneId: string,
    tabId: string,
    onSearchCleanup?: () => void,
  ) {
    const p = tabStore.findPaneById(paneId);
    if (p?.activeTabId === tabId) return;
    const ot = tabStore.getFocusedTab();
    if (ot?.isSearch && onSearchCleanup) onSearchCleanup();
    if (ot) saveFileStateToTab(ot);
    tabStore.switchTab(paneId, tabId);
    tabStore.focusPane(paneId);
    focusedPaneId.value = paneId;
    if (p) {
      const t = p.tabs.find((x: Tab) => x.id === tabId);
      if (t) {
        loadFileStateFromTab(t);
        if (!t.path) store.loadDrives();
      }
    }
  }

  async function onTabClose(
    paneId: string,
    tabId: string,
    onSearchCleanup?: () => void,
  ) {
    const pane = tabStore.findPaneById(paneId);
    const isClosingSearch = pane?.tabs.find((t) => t.id === tabId)?.isSearch;
    if (isClosingSearch && onSearchCleanup) onSearchCleanup();

    tabStore.closeTab(paneId, tabId);
    const p = tabStore.findPaneById(paneId);
    if (!p) return;
    const t = p.tabs.find((x: Tab) => x.id === p.activeTabId);
    if (t) {
      loadFileStateFromTab(t);
      if (!t.path) store.loadDrives();
    }
  }

  async function onNewTab(paneId: string) {
    const ot = tabStore.getFocusedTab();
    if (ot) saveFileStateToTab(ot);
    tabStore.focusPane(paneId);
    focusedPaneId.value = paneId;
    tabStore.addTab(
      paneId,
      store.currentPath || "",
      store.currentDirectoryName || "This PC",
    );

    const fp = tabStore.getFocusedPane();
    if (fp) {
      const nt = fp.tabs.find((t: Tab) => t.id === fp.activeTabId);
      if (nt) {
        loadFileStateFromTab(nt);
        if (nt.path) {
          await store.navigateTo(nt.path, false);
          const tab = tabStore.getFocusedTab();
          if (tab) saveFileStateToTab(tab);
        } else {
          await store.loadDrives();
        }
      }
    }
  }

  async function navigatePane(paneId: string, path: string) {
    const samePane = tabStore.getFocusedPane()?.id === paneId;
    if (!samePane) {
      const ot = tabStore.getFocusedTab();
      if (ot) saveFileStateToTab(ot);
      tabStore.focusPane(paneId);
      focusedPaneId.value = paneId;
    }
    if (!path) {
      store.currentPath = "";
      store.files = [];
      await store.loadDrives();
      store.syncToTab();
    } else {
      await store.navigateTo(path, false);
    }
  }

  async function sidebarNavigate(path: string) {
    const fp = tabStore.getFocusedPane();
    if (!fp) return;
    await navigatePane(fp.id, path);
  }

  async function sidebarHome() {
    const fp = tabStore.getFocusedPane();
    if (!fp) return;
    await navigatePane(fp.id, "");
  }

  async function toolbarBack() {
    await store.navigateBack();
  }
  async function toolbarForward() {
    await store.navigateForward();
  }
  async function toolbarUp() {
    await store.navigateUp();
  }
  async function toolbarRefresh() {
    await store.refresh();
  }

  async function toolbarAddress(path: string) {
    const fp = tabStore.getFocusedPane();
    if (!fp) return;
    await navigatePane(fp.id, path);
  }

  return {
    focusedPaneId,
    onPaneFocus,
    onTabClick,
    onTabClose,
    onNewTab,
    navigatePane,
    sidebarNavigate,
    sidebarHome,
    toolbarBack,
    toolbarForward,
    toolbarUp,
    toolbarRefresh,
    toolbarAddress,
  };
}
