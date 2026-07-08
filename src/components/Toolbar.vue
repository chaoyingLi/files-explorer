<template>
    <div class="toolbar" @contextmenu.prevent.stop>
        <div class="toolbar-left">
            <button
                class="icon-btn"
                :title="t('toolbar.back')"
                :disabled="!nav.canGoBack"
                @click="$emit('navigateBack')"
            >
                <svg viewBox="0 0 16 16" fill="none">
                    <path
                        d="M10 3L5 8l5 5"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    />
                </svg>
            </button>
            <button
                class="icon-btn"
                :title="t('toolbar.forward')"
                :disabled="!nav.canGoForward"
                @click="$emit('navigateForward')"
            >
                <svg viewBox="0 0 16 16" fill="none">
                    <path
                        d="M6 3l5 5-5 5"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    />
                </svg>
            </button>
            <button
                class="icon-btn"
                :title="t('toolbar.up')"
                @click="$emit('navigateUp')"
            >
                <svg viewBox="0 0 16 16" fill="none">
                    <path
                        d="M3 10l5-6 5 6"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    />
                </svg>
            </button>
            <button
                class="icon-btn"
                :title="t('toolbar.refresh')"
                @click="$emit('refresh')"
            >
                <svg viewBox="0 0 16 16" fill="none">
                    <path
                        d="M2 8a6 6 0 0111.5-2M14 8a6 6 0 01-11.5 2"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                    />
                    <polyline
                        points="12,1.5 13.5,6 8.5,6"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    />
                </svg>
            </button>
        </div>
        <div class="toolbar-center">
            <div class="address-bar-wrapper">
                <div class="address-bar">
                    <input
                        ref="addressInput"
                        v-model="addressValue"
                        class="address-input"
                        spellcheck="false"
                        @keydown.enter="onAddressEnter"
                        @keydown.escape="showDropdown = false"
                        @keydown.down.prevent="onAddressArrowDown"
                        @keydown.up.prevent="onAddressArrowUp"
                        @focus="onAddressFocus"
                        @input="onAddressInput"
                        @blur="onAddressBlur"
                    />
                </div>
                <!-- Address autocomplete dropdown -->
                <div
                    v-if="showDropdown && suggestions.length > 0"
                    class="address-dropdown"
                    @mousedown.prevent
                >
                    <div
                        v-for="(item, idx) in suggestions"
                        :key="item.path"
                        class="address-suggestion"
                        :class="{ highlighted: idx === selectedIdx }"
                        @mousedown.prevent="selectSuggestion(item)"
                        @mouseenter="selectedIdx = idx"
                    >
                        <span class="suggestion-icon">📁</span>
                        <span>{{ item.name }}</span>
                    </div>
                </div>
            </div>
            <div class="search-bar-wrapper">
                <div class="search-bar">
                    <svg class="search-icon" viewBox="0 0 20 20" fill="none">
                        <circle
                            cx="8.5"
                            cy="8.5"
                            r="5.5"
                            stroke="currentColor"
                            stroke-width="1.5"
                        />
                        <path
                            d="M12.5 12.5L17 17"
                            stroke="currentColor"
                            stroke-width="1.5"
                            stroke-linecap="round"
                        />
                    </svg>
                    <input
                        ref="searchInput"
                        v-model="searchQuery"
                        class="search-input"
                        :placeholder="t('toolbar.search')"
                        :title="t('toolbar.searchWildcardHint')"
                        spellcheck="false"
                        @keydown.enter="onSearchEnter"
                        @keydown.escape="searchHistoryShow = false"
                        @keydown.down.prevent="onSearchHistoryDown"
                        @keydown.up.prevent="onSearchHistoryUp"
                        @focus="onSearchFocus"
                        @blur="onSearchBlur"
                    />
                    <!-- Stop button when searching -->
                    <button
                        v-if="store.isSearching && searchQuery"
                        class="icon-btn stop-btn"
                        :title="t('toolbar.stopSearch')"
                        @click="cancelSearch"
                    >
                        <svg viewBox="0 0 20 20" fill="none">
                            <rect
                                x="5"
                                y="5"
                                width="10"
                                height="10"
                                rx="1.5"
                                fill="currentColor"
                                opacity="0.7"
                            />
                        </svg>
                    </button>
                    <!-- Clear button when not searching -->
                    <button
                        v-else-if="searchQuery"
                        class="icon-btn clear-btn"
                        @click="clearSearch"
                    >
                        <svg viewBox="0 0 20 20" fill="none">
                            <circle
                                cx="10"
                                cy="10"
                                r="7"
                                fill="currentColor"
                                opacity="0.15"
                            />
                            <path
                                d="M7.5 7.5l5 5M12.5 7.5l-5 5"
                                stroke="currentColor"
                                stroke-width="1.3"
                                stroke-linecap="round"
                            />
                        </svg>
                    </button>
                </div>
                <!-- Search history dropdown -->
                <div
                    v-if="searchHistoryShow && searchHistory.length > 0"
                    class="search-history-dropdown"
                    @mousedown.prevent
                >
                    <div class="search-history-header">
                        <span class="search-history-label">最近搜索</span>
                        <button
                            class="search-history-clear"
                            @mousedown.prevent="clearSearchHistory"
                        >
                            清除
                        </button>
                    </div>
                    <div
                        v-for="(item, idx) in searchHistory"
                        :key="item"
                        class="search-history-item"
                        :class="{ highlighted: idx === searchHistoryIdx }"
                        @mousedown.prevent="selectSearchHistory(item)"
                        @mouseenter="searchHistoryIdx = idx"
                    >
                        <svg
                            class="search-history-clock"
                            viewBox="0 0 14 14"
                            fill="none"
                        >
                            <circle
                                cx="7"
                                cy="7"
                                r="5.5"
                                stroke="currentColor"
                                stroke-width="1"
                            />
                            <path
                                d="M7 4.5V7l2 1.5"
                                stroke="currentColor"
                                stroke-width="1"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                            />
                        </svg>
                        <span>{{ item }}</span>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import { useNavigationStore } from "@/stores/navigationStore";
import { useSettingsStore } from "@/stores/settingsStore";
import { displayPath } from "@/utils/platform";

const { t, locale } = useI18n();
const store = useFileStore();
const nav = useNavigationStore();
const settings = useSettingsStore();
const addressInput = ref<HTMLInputElement | null>(null);
const searchInput = ref<HTMLInputElement | null>(null);
const addressValue = ref("");
const searchQuery = ref("");

// ── Search history ──
const SEARCH_HISTORY_KEY = "search-history";
const SEARCH_HISTORY_MAX = 15;
const searchHistory = ref<string[]>([]);
const searchHistoryShow = ref(false);
const searchHistoryIdx = ref(0);

function loadSearchHistory() {
    try {
        const raw = localStorage.getItem(SEARCH_HISTORY_KEY);
        if (raw) searchHistory.value = JSON.parse(raw);
    } catch {
        searchHistory.value = [];
    }
}
function saveSearchHistory() {
    localStorage.setItem(
        SEARCH_HISTORY_KEY,
        JSON.stringify(searchHistory.value),
    );
}
function addSearchHistory(query: string) {
    const q = query.trim();
    if (!q) return;
    // Deduplicate: remove existing, insert at front
    searchHistory.value = searchHistory.value.filter((h) => h !== q);
    searchHistory.value.unshift(q);
    // Cap at 15
    if (searchHistory.value.length > SEARCH_HISTORY_MAX) {
        searchHistory.value = searchHistory.value.slice(0, SEARCH_HISTORY_MAX);
    }
    saveSearchHistory();
}
function clearSearchHistory() {
    searchHistory.value = [];
    saveSearchHistory();
    searchHistoryShow.value = false;
}
function onSearchFocus() {
    if (searchHistory.value.length > 0 && !searchQuery.value) {
        searchHistoryShow.value = true;
        searchHistoryIdx.value = 0;
    }
}
function onSearchBlur() {
    setTimeout(() => {
        searchHistoryShow.value = false;
    }, 150);
}
function onSearchHistoryDown() {
    if (!searchHistoryShow.value) {
        if (searchHistory.value.length > 0 && !searchQuery.value) {
            searchHistoryShow.value = true;
            searchHistoryIdx.value = 0;
        }
        return;
    }
    if (searchHistoryIdx.value < searchHistory.value.length - 1) {
        searchHistoryIdx.value++;
    } else {
        searchHistoryIdx.value = 0;
    }
}
function onSearchHistoryUp() {
    if (!searchHistoryShow.value) return;
    if (searchHistoryIdx.value > 0) {
        searchHistoryIdx.value--;
    } else {
        searchHistoryIdx.value = searchHistory.value.length - 1;
    }
}
function selectSearchHistory(item: string) {
    searchQuery.value = item;
    searchHistoryShow.value = false;
    emit("searchSubmit", item);
}

// ── Address autocomplete ──
const showDropdown = ref(false);
const suggestions = ref<{ name: string; path: string }[]>([]);
const selectedIdx = ref(0);

watch(
    () => store.currentPath,
    (p) => {
        if (p) addressValue.value = displayPath(p);
    },
    { immediate: true },
);

const emit = defineEmits<{
    navigateBack: [];
    navigateForward: [];
    navigateUp: [];
    refresh: [];
    navigateAddress: [path: string];
    searchSubmit: [query: string];
}>();

// Sync locale from settings store
watch(
    () => settings.locale,
    (l) => {
        locale.value = l;
    },
    { immediate: true },
);

function onAddressFocus() {
    if (addressValue.value) {
        updateSuggestions(addressValue.value);
        showDropdown.value = suggestions.value.length > 0;
    }
}

async function onAddressInput() {
    const val = addressValue.value;
    if (val) {
        await updateSuggestions(val);
        showDropdown.value = suggestions.value.length > 0;
        selectedIdx.value = 0;
    } else {
        showDropdown.value = false;
    }
}

function onAddressBlur() {
    setTimeout(() => {
        showDropdown.value = false;
    }, 150);
}

async function updateSuggestions(input: string) {
    const lastSep = Math.max(input.lastIndexOf("/"), input.lastIndexOf("\\"));
    const dirPath = lastSep >= 0 ? input.substring(0, lastSep) || "/" : "";
    const partial =
        lastSep >= 0
            ? input.substring(lastSep + 1).toLowerCase()
            : input.toLowerCase();

    try {
        const { listDirectory } = await import("@/utils/tauri");
        const entries = await listDirectory(dirPath || "/");
        suggestions.value = entries
            .filter((e) => e.is_dir && e.name.toLowerCase().startsWith(partial))
            .map((e) => ({ name: e.name, path: e.path }))
            .slice(0, 8);
    } catch {
        suggestions.value = [];
    }
}

function onAddressArrowDown() {
    if (selectedIdx.value < suggestions.value.length - 1) selectedIdx.value++;
    else selectedIdx.value = 0;
}

function onAddressArrowUp() {
    if (selectedIdx.value > 0) selectedIdx.value--;
    else selectedIdx.value = suggestions.value.length - 1;
}

function selectSuggestion(item: { name: string; path: string }) {
    addressValue.value = item.path;
    showDropdown.value = false;
    emit("navigateAddress", item.path);
}

function onAddressEnter() {
    if (showDropdown.value && suggestions.value.length > 0) {
        const sel = suggestions.value[selectedIdx.value];
        if (sel) {
            selectSuggestion(sel);
            return;
        }
    }
    showDropdown.value = false;
    const path = addressValue.value.trim();
    if (path) {
        emit("navigateAddress", path);
    }
}
function onSearchEnter() {
    // If history dropdown is open, select the highlighted item
    if (searchHistoryShow.value && searchHistory.value.length > 0) {
        const sel = searchHistory.value[searchHistoryIdx.value];
        if (sel) {
            selectSearchHistory(sel);
            return;
        }
    }
    searchHistoryShow.value = false;
    const q = searchQuery.value.trim();
    if (q) {
        addSearchHistory(q);
        emit("searchSubmit", q);
    } else {
        store.cancelCurrentSearch();
    }
}
function clearSearch() {
    searchQuery.value = "";
    store.cancelCurrentSearch();
    searchHistoryShow.value = false;
}
async function cancelSearch() {
    searchQuery.value = "";
    searchHistoryShow.value = false;
    await store.cancelCurrentSearch();
}

// Initialize search history on mount
loadSearchHistory();
</script>

<style scoped>
.toolbar {
    display: flex;
    align-items: center;
    padding: 6px 10px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    gap: 8px;
    height: 44px;
}
.toolbar-left {
    display: flex;
    gap: 2px;
}
.toolbar-center {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
}
.address-bar-wrapper {
    flex: 1;
    position: relative;
}

.address-bar {
    width: 100%;
}
.address-input {
    width: 100%;
    height: 30px;
    font-size: var(--font-size-base);
    background: var(--input-bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 0 12px;
    color: var(--text-primary);
}

.address-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    margin-top: 2px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 4px;
    max-height: 360px;
    overflow-y: auto;
    z-index: 1000;
    box-shadow: 0 8px 32px var(--shadow);
}

.address-suggestion {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    border-radius: 6px;
    cursor: pointer;
    font-size: var(--font-size-base);
    transition: background 0.05s;
}

.address-suggestion:hover,
.address-suggestion.highlighted {
    background: var(--bg-hover);
}

.suggestion-icon {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
}

.suggestion-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.suggestion-meta {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    flex-shrink: 0;
}
.search-bar {
    display: flex;
    align-items: center;
    background: var(--input-bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 0 8px;
    width: 200px;
    height: 30px;
}
.search-icon {
    width: 14px;
    height: 14px;
    color: var(--text-muted);
    margin-right: 6px;
    flex-shrink: 0;
}
.search-input {
    border: none;
    background: transparent;
    flex: 1;
    height: 100%;
    font-size: var(--font-size-base);
    padding: 0;
    min-width: 0;
}
.search-input:focus {
    border: none;
    outline: none;
}
.clear-btn,
.stop-btn {
    padding: 1px;
    min-width: 18px;
    height: 18px;
    flex-shrink: 0;
}

.stop-btn:hover {
    color: var(--danger) !important;
}
.content-search-toggle {
    padding: 1px 4px;
    min-width: 22px;
    height: 20px;
    flex-shrink: 0;
    font-size: var(--font-size-sm);
    opacity: 0.5;
    transition: opacity 0.15s;
}
.content-search-toggle.active {
    opacity: 1;
    color: var(--accent);
}
.content-search-toggle:hover {
    opacity: 0.8;
}

.search-bar-wrapper {
    position: relative;
}
.search-history-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    margin-top: 2px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 4px;
    max-height: 340px;
    overflow-y: auto;
    z-index: 1000;
    box-shadow: 0 8px 32px var(--shadow);
}
.search-history-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 8px;
    border-bottom: 1px solid var(--border);
    margin-bottom: 2px;
}
.search-history-label {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
}
.search-history-clear {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    background: none;
    border: none;
    cursor: pointer;
    padding: 2px 6px;
    border-radius: 4px;
}
.search-history-clear:hover {
    color: var(--danger);
    background: var(--bg-hover);
}
.search-history-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    border-radius: 6px;
    cursor: pointer;
    font-size: var(--font-size-base);
    transition: background 0.05s;
}
.search-history-item:hover,
.search-history-item.highlighted {
    background: var(--bg-hover);
}
.search-history-clock {
    width: 14px;
    height: 14px;
    color: var(--text-muted);
    flex-shrink: 0;
}
</style>
