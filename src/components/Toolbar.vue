<template>
    <div class="toolbar">
        <div class="toolbar-left">
            <button
                class="icon-btn"
                :title="t('toolbar.back')"
                :disabled="!store.canGoBack"
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
                :disabled="!store.canGoForward"
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
                        @keydown.escape="showAddressDropdown = false"
                        @keydown.down.prevent="onAddressArrowDown"
                        @keydown.up.prevent="onAddressArrowUp"
                        @focus="onAddressFocus"
                        @input="onAddressInput"
                        @blur="onAddressBlur"
                    />
                </div>
                <!-- Address autocomplete dropdown -->
                <div
                    v-if="showAddressDropdown && addressSuggestions.length > 0"
                    class="address-dropdown"
                    @mousedown.prevent
                >
                    <div
                        v-for="(item, idx) in addressSuggestions"
                        :key="item.path"
                        class="address-suggestion"
                        :class="{ highlighted: idx === addressSelectedIndex }"
                        @mousedown.prevent="selectAddressSuggestion(item)"
                        @mouseenter="addressSelectedIndex = idx"
                    >
                        <svg
                            class="suggestion-icon"
                            viewBox="0 0 16 16"
                            fill="none"
                        >
                            <path
                                v-if="item.is_dir"
                                d="M2 4.5c0-.83.67-1.5 1.5-1.5h2.5a1.5 1.5 0 011.1.5l.7.85a.5.5 0 00.38.18H12c.83 0 1.5.67 1.5 1.5v4.5a1.5 1.5 0 01-1.5 1.5h-9A1.5 1.5 0 012 11V4.5z"
                                fill="var(--accent)"
                                opacity="0.8"
                            />
                            <path
                                v-else
                                d="M4 2h3.8l2.7 2.7V12a1 1 0 01-1 1H4a1 1 0 01-1-1V3a1 1 0 011-1z"
                                fill="var(--text-muted)"
                                opacity="0.6"
                            />
                        </svg>
                        <span class="suggestion-name">{{ item.name }}</span>
                        <span class="suggestion-meta">{{
                            item.is_dir
                                ? t("fileTypes.folder")
                                : item.extension.toUpperCase()
                        }}</span>
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
            <button
                class="icon-btn settings-btn"
                :title="t('settings.title')"
                @click="$emit('openSettings')"
            >
                <svg viewBox="0 0 20 20" fill="none">
                    <circle
                        cx="10"
                        cy="10"
                        r="2.5"
                        stroke="currentColor"
                        stroke-width="1.5"
                    />
                    <path
                        d="M10 1.5v2M10 16.5v2M3.5 10h-2M18.5 10h-2M4.3 4.3l1.4 1.4M14.3 14.3l1.4 1.4M4.3 15.7l1.4-1.4M14.3 5.7l1.4-1.4"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                    />
                </svg>
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import { useSettingsStore } from "@/stores/settingsStore";
import type { FileEntry } from "@/types";

const { t, locale } = useI18n();
const store = useFileStore();
const settings = useSettingsStore();
const addressInput = ref<HTMLInputElement | null>(null);
const addressValue = ref("");
const searchQuery = ref("");

// ── Address autocomplete ──
const showAddressDropdown = ref(false);
const addressSelectedIndex = ref(0);

const addressSuggestions = computed(() => {
    const val = addressValue.value.trim();
    if (!val || !store.currentPath || !showAddressDropdown.value) return [];
    const lower = val.toLowerCase();

    // Filter and sort: directories first, then by name
    const matched = store.files.filter((f: FileEntry) =>
        f.name.toLowerCase().startsWith(lower),
    );
    matched.sort((a: FileEntry, b: FileEntry) => {
        if (a.is_dir && !b.is_dir) return -1;
        if (!a.is_dir && b.is_dir) return 1;
        return a.name.localeCompare(b.name);
    });
    return matched.slice(0, 15);
});

const emit = defineEmits<{
    openSettings: [];
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

watch(
    () => store.currentPath,
    (val) => {
        addressValue.value = val || t("statusBar.thisPc");
    },
);

function navigateToAddress(path?: string) {
    const p = path || addressValue.value.trim();
    if (p && p !== t("statusBar.thisPc")) {
        emit("navigateAddress", p);
    }
    showAddressDropdown.value = false;
}

function onAddressFocus() {
    addressInput.value?.select();
    showAddressDropdown.value = true;
    addressSelectedIndex.value = 0;
}
function onAddressInput() {
    showAddressDropdown.value = true;
    addressSelectedIndex.value = 0;
}
function onAddressBlur() {
    // Delay to allow click on suggestion
    setTimeout(() => {
        showAddressDropdown.value = false;
    }, 150);
}
function onAddressEnter() {
    if (showAddressDropdown.value && addressSuggestions.value.length > 0) {
        const sel = addressSuggestions.value[addressSelectedIndex.value];
        if (sel) {
            const parent =
                store.currentPath.endsWith("\\") ||
                store.currentPath.endsWith("/")
                    ? store.currentPath
                    : store.currentPath +
                      (store.currentPath.includes("/") ? "/" : "\\");
            navigateToAddress(parent + sel.name);
            return;
        }
    }
    navigateToAddress();
}
function onAddressArrowDown() {
    const max = addressSuggestions.value.length - 1;
    addressSelectedIndex.value = Math.min(addressSelectedIndex.value + 1, max);
}
function onAddressArrowUp() {
    addressSelectedIndex.value = Math.max(addressSelectedIndex.value - 1, 0);
}
function selectAddressSuggestion(item: FileEntry) {
    const parent =
        store.currentPath.endsWith("\\") || store.currentPath.endsWith("/")
            ? store.currentPath
            : store.currentPath +
              (store.currentPath.includes("/") ? "/" : "\\");
    navigateToAddress(parent + item.name);
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
.settings-btn {
    flex-shrink: 0;
}
</style>
