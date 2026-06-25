import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import { useTabStore, type Tab } from "@/stores/tabStore";
import * as tauri from "@/utils/tauri";

export function useSearchService(
  saveFileStateToTab: (tab: Tab) => void,
  showToast: (msg: string, isError?: boolean) => void,
) {
  const { t } = useI18n();
  const store = useFileStore();
  const tabStore = useTabStore();
  let searchTabUnlisten: (() => void) | null = null;

  function cleanupSearch() {
    if (searchTabUnlisten) {
      searchTabUnlisten();
      searchTabUnlisten = null;
    }
  }

  async function submitSearch(query: string, contentQuery: string = "") {
    await store.cancelCurrentSearch();
    cleanupSearch();

    const fp = tabStore.getFocusedPane();
    if (!fp) return;

    const ot = tabStore.getFocusedTab();
    if (ot) saveFileStateToTab(ot);

    const searchDir = store.currentPath || ot?.path || "";
    if (!searchDir) {
      showToast(t("search.noDirectory"), true);
      return;
    }

    const searchTitle = t("search.resultsTabTitle", {
      query,
      folder: searchDir,
    });
    tabStore.addTab(fp.id, searchDir, searchTitle);
    tabStore.focusPane(fp.id);

    const nt = tabStore.getFocusedTab();
    if (!nt) return;
    nt.files = [];
    nt.isSearch = true;
    nt.searchQuery = query;
    nt.searchDone = false;
    nt.searchTotal = 0;

    store.files = [];
    store.isSearching = true;
    store.currentPath = searchDir;

    const { listen } = await import("@tauri-apps/api/event");
    searchTabUnlisten = await listen<{
      files: any[];
      total: number;
      done: boolean;
      truncated: boolean;
    }>("search-progress", (event) => {
      const p = event.payload;
      if (p.files.length > 0) {
        if (!nt.files) nt.files = [];
        for (const f of p.files) {
          nt.files.push(f);
        }
      }
      if (p.done) {
        nt.searchDone = true;
        nt.searchTotal = p.total;
        store.isSearching = false;
        if (p.truncated) {
          nt.title = t("search.resultsTabTruncated", {
            query,
            count: p.total,
            folder: searchDir,
          });
          // Show toast to inform user
          showToast(
            t("search.resultsTruncatedHint", {
              max: p.total,
            }),
            false,
          );
        } else {
          nt.title = t("search.resultsTab", {
            query,
            count: p.total,
            folder: searchDir,
          });
        }
        cleanupSearch();
      }
    });

    try {
      await tauri.searchFiles(searchDir, query, contentQuery);
    } catch (e: any) {
      store.isSearching = false;
      showToast(e, true);
      cleanupSearch();
    }
  }

  return { submitSearch, cleanupSearch };
}
