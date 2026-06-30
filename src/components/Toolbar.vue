<template>
    <div class="toolbar">
        <div class="toolbar-left">
            <button
                class="icon-btn"
                :title="t('toolbar.back')"
                :disabled="!nav.canGoBack"
                @click="$emit('navigateBack')"
            >
                <svg viewBox="0 0 20 20" fill="none">
                    <path
                        d="M12.5 4.5L7 10l5.5 5.5"
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
                <svg viewBox="0 0 20 20" fill="none">
                    <path
                        d="M7.5 4.5L13 10l-5.5 5.5"
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
                <svg viewBox="0 0 20 20" fill="none">
                    <path d="M10 4l-5 6h3v5h4v-5h3l-5-6z" fill="currentColor" />
                </svg>
            </button>
            <button
                class="icon-btn"
                :title="t('toolbar.refresh')"
                @click="$emit('refresh')"
            >
                <svg viewBox="0 0 20 20" fill="none">
                    <path
                        d="M3 10a7 7 0 0113.9-1M17 10a7 7 0 01-13.9 1"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                    />
                    <path
                        d="M14 2.5L17 8.5h-5M6 17.5L3 11.5h5"
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
                    v-model="searchQuery"
                    class="search-input"
                    :placeholder="t('toolbar.search')"
                    :title="t('toolbar.searchWildcardHint')"
                    spellcheck="false"
                    @keydown.enter="onSearchEnter"
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
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import { useNavigationStore } from "@/stores/navigationStore";
import { useSettingsStore } from "@/stores/settingsStore";

const { t, locale } = useI18n();
const store = useFileStore();
const nav = useNavigationStore();
const settings = useSettingsStore();
const addressInput = ref<HTMLInputElement | null>(null);
const addressValue = ref("");
const searchQuery = ref("");

// ── Address autocomplete ──
const showDropdown = ref(false);
const suggestions = ref<{ name: string; path: string }[]>([]);
const selectedIdx = ref(0);

watch(
    () => store.currentPath,
    (p) => {
        if (p) addressValue.value = p;
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
    const q = searchQuery.value.trim();
    if (q) {
        emit("searchSubmit", q);
    } else {
        store.cancelCurrentSearch();
    }
}
async function cancelSearch() {
    searchQuery.value = "";
    await store.cancelCurrentSearch();
}
function clearSearch() {
    searchQuery.value = "";
    store.cancelCurrentSearch();
}
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
    font-size: 13px;
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
    font-size: 13px;
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
    font-size: 11px;
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
    font-size: 13px;
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
    font-size: 11px;
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
</style>
