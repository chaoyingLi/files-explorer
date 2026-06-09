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
            <div class="address-bar">
                <input
                    ref="addressInput"
                    v-model="addressValue"
                    class="address-input"
                    spellcheck="false"
                    @keydown.enter="navigateToAddress"
                    @focus="onAddressFocus"
                />
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
                    spellcheck="false"
                    @input="onSearchInput"
                />
                <button
                    v-if="searchQuery"
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
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import { useSettingsStore } from "@/stores/settingsStore";

const { t, locale } = useI18n();
const store = useFileStore();
const settings = useSettingsStore();
const addressInput = ref<HTMLInputElement | null>(null);
const addressValue = ref("");
const searchQuery = ref("");
let searchTimeout: ReturnType<typeof setTimeout> | null = null;

const emit = defineEmits<{
    openSettings: [];
    navigateBack: [];
    navigateForward: [];
    navigateUp: [];
    refresh: [];
    navigateAddress: [path: string];
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

function onAddressFocus() {
    addressInput.value?.select();
}
function navigateToAddress() {
    const p = addressValue.value.trim();
    if (p && p !== t("statusBar.thisPc")) emit("navigateAddress", p);
}
function onSearchInput() {
    if (searchTimeout) clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => {
        if (searchQuery.value.trim()) store.search(searchQuery.value.trim());
        else {
            store.isSearching = false;
            store.searchResults = [];
        }
    }, 300);
}
function clearSearch() {
    searchQuery.value = "";
    store.isSearching = false;
    store.searchResults = [];
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
    -webkit-app-region: drag;
}
.toolbar-left {
    display: flex;
    gap: 2px;
    -webkit-app-region: no-drag;
}
.toolbar-center {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    -webkit-app-region: no-drag;
}
.address-bar {
    flex: 1;
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
.clear-btn {
    padding: 1px;
    min-width: 18px;
    height: 18px;
    flex-shrink: 0;
}
.settings-btn {
    flex-shrink: 0;
}
</style>
