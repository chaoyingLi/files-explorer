import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { ColumnState } from "./viewStore";

// ── Column history entry (for column-view snapshot support) ──
interface ColumnHistoryEntry {
  path: string;
  stack: ColumnState[];
}

export const useNavigationStore = defineStore("navigation", () => {
  const history = ref<(string | ColumnHistoryEntry)[]>([]);
  const historyIndex = ref(-1);

  const canGoBack = computed(() => historyIndex.value > 0);
  const canGoForward = computed(
    () => historyIndex.value < history.value.length - 1,
  );

  function pushHistory(entry: string | ColumnHistoryEntry) {
    // Avoid duplicate consecutive entries
    const last =
      historyIndex.value >= 0 && historyIndex.value < history.value.length
        ? history.value[historyIndex.value]
        : undefined;
    const lastPath = typeof last === "string" ? last : last?.path;
    const newPath = typeof entry === "string" ? entry : entry.path;
    if (newPath === lastPath) return;

    // Truncate forward history when navigating from middle of stack
    if (historyIndex.value < history.value.length - 1) {
      history.value = history.value.slice(0, historyIndex.value + 1);
    }
    history.value.push(entry);
    historyIndex.value = history.value.length - 1;

    // Cap history size
    if (history.value.length > 50) {
      history.value = history.value.slice(-50);
      historyIndex.value = history.value.length - 1;
    }
  }

  /** Get the current history entry (for column-view snapshot restoration) */
  function currentEntry(): string | ColumnHistoryEntry | undefined {
    if (historyIndex.value < 0 || historyIndex.value >= history.value.length)
      return undefined;
    return history.value[historyIndex.value];
  }

  /** Peek at previous entry without popping */
  function peekBack(): string | ColumnHistoryEntry | undefined {
    if (historyIndex.value <= 0) return undefined;
    return history.value[historyIndex.value - 1];
  }

  /** Peek at next entry without advancing */
  function peekForward(): string | ColumnHistoryEntry | undefined {
    if (historyIndex.value >= history.value.length - 1) return undefined;
    return history.value[historyIndex.value + 1];
  }

  function advanceBack(): string | ColumnHistoryEntry | undefined {
    if (!canGoBack.value) return undefined;
    historyIndex.value--;
    return history.value[historyIndex.value];
  }

  function advanceForward(): string | ColumnHistoryEntry | undefined {
    if (!canGoForward.value) return undefined;
    historyIndex.value++;
    return history.value[historyIndex.value];
  }

  return {
    history,
    historyIndex,
    canGoBack,
    canGoForward,
    pushHistory,
    currentEntry,
    peekBack,
    peekForward,
    advanceBack,
    advanceForward,
  };
});
