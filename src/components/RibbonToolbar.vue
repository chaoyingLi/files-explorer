<template>
    <div class="ribbon">
        <!-- New group -->
        <div class="ribbon-group">
            <button
                class="ribbon-btn"
                :title="t('contextMenu.newFolder')"
                @click="$emit('action', 'newFolder')"
            >
                <svg viewBox="0 0 20 20" fill="none">
                    <path
                        d="M3 5.5A1.5 1.5 0 014.5 4h3.3a1.5 1.5 0 011.2.6l1 1.4h5.5A1.5 1.5 0 0117 7.5v7a1.5 1.5 0 01-1.5 1.5h-11A1.5 1.5 0 013 14.5V5.5z"
                        fill="#F5C542"
                        opacity="0.85"
                    />
                    <path
                        d="M3 6.5A1.5 1.5 0 014.5 5h3.3a1.5 1.5 0 011.2.6l1 1.4h5.5A1.5 1.5 0 0117 8.5v6a1.5 1.5 0 01-1.5 1.5h-11A1.5 1.5 0 013 14.5V5.5z"
                        fill="#F9E2AF"
                    />
                    <path
                        d="M7.5 9.5v3M6 11h3"
                        stroke="#1E1E2E"
                        stroke-width="1.2"
                        stroke-linecap="round"
                        opacity="0.4"
                    />
                </svg>
                <span class="ribbon-label">{{
                    t("contextMenu.newFolder")
                }}</span>
            </button>
            <button
                class="ribbon-btn"
                :title="t('contextMenu.newFile')"
                @click="$emit('action', 'newFile')"
            >
                <svg viewBox="0 0 20 20" fill="none">
                    <path
                        d="M5.5 3h5.7l4.8 4.8V15.5a1.5 1.5 0 01-1.5 1.5H5.5A1.5 1.5 0 014 15.5V4.5A1.5 1.5 0 015.5 3z"
                        fill="#89B4FA"
                        opacity="0.8"
                    />
                    <path d="M11 3v4.5a.5.5 0 00.5.5H16" fill="#74C7EC" />
                    <path
                        d="M7 11v3M5.5 12.5h3"
                        stroke="#1E1E2E"
                        stroke-width="1.2"
                        stroke-linecap="round"
                        opacity="0.35"
                    />
                </svg>
                <span class="ribbon-label">{{ t("contextMenu.newFile") }}</span>
            </button>
        </div>

        <div class="ribbon-sep"></div>

        <!-- Clipboard group -->
        <div class="ribbon-group">
            <button
                class="ribbon-btn"
                :disabled="!hasSelection"
                :title="t('contextMenu.cut')"
                @click="$emit('action', 'cut')"
            >
                <svg viewBox="0 0 20 20" fill="none">
                    <path
                        d="M6.5 4.5l-2 2 2 2M8.5 8.5l2-2-2-2"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    />
                    <path
                        d="M5.5 6.5h4a3 3 0 013 3v6M9.5 16.5h-4a3 3 0 01-3-3v-6"
                        stroke="currentColor"
                        stroke-width="1.3"
                        stroke-linecap="round"
                        opacity="0.5"
                    />
                </svg>
                <span class="ribbon-label">{{ t("contextMenu.cut") }}</span>
            </button>
            <button
                class="ribbon-btn"
                :disabled="!hasSelection"
                :title="t('contextMenu.copy')"
                @click="$emit('action', 'copy')"
            >
                <svg viewBox="0 0 20 20" fill="none">
                    <rect
                        x="6"
                        y="4"
                        width="10"
                        height="12"
                        rx="1.5"
                        stroke="currentColor"
                        stroke-width="1.5"
                    />
                    <path
                        d="M4 7v10a1 1 0 001 1h10"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                    />
                </svg>
                <span class="ribbon-label">{{ t("contextMenu.copy") }}</span>
            </button>
            <button
                class="ribbon-btn"
                :title="t('contextMenu.paste')"
                @click="$emit('action', 'paste')"
            >
                <svg viewBox="0 0 20 20" fill="none">
                    <rect
                        x="5"
                        y="6"
                        width="11"
                        height="10"
                        rx="1.5"
                        fill="currentColor"
                        opacity="0.15"
                    />
                    <rect
                        x="5"
                        y="6"
                        width="11"
                        height="10"
                        rx="1.5"
                        stroke="currentColor"
                        stroke-width="1.5"
                    />
                    <path
                        d="M7 3.5h7.2l2.8 2.8V14"
                        stroke="currentColor"
                        stroke-width="1.3"
                        stroke-linecap="round"
                        opacity="0.5"
                    />
                </svg>
                <span class="ribbon-label">{{ t("contextMenu.paste") }}</span>
            </button>
        </div>

        <div class="ribbon-sep"></div>

        <!-- Organize group -->
        <div class="ribbon-group">
            <button
                class="ribbon-btn"
                :disabled="!hasSelection"
                :title="t('contextMenu.delete')"
                @click="$emit('action', 'delete')"
            >
                <svg viewBox="0 0 20 20" fill="none">
                    <path
                        d="M4.5 5.5h11M8 5.5V4a1 1 0 011-1h2a1 1 0 011 1v1.5"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                    />
                    <path
                        d="M5.5 5.5l1 10a1 1 0 001 1h5a1 1 0 001-1l1-10"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                    />
                </svg>
                <span class="ribbon-label">{{ t("contextMenu.delete") }}</span>
            </button>
            <button
                class="ribbon-btn"
                :disabled="selectedCount !== 1"
                :title="t('contextMenu.rename')"
                @click="$emit('action', 'rename')"
            >
                <svg viewBox="0 0 20 20" fill="none">
                    <path
                        d="M4 14.5L5.5 16l2.5-2.5M14 9l-4-4L5 10l4 4 5-5z"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    />
                    <path
                        d="M10 5l1.5-1.5a2 2 0 012.8 2.8L12.5 8"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                    />
                </svg>
                <span class="ribbon-label">{{ t("contextMenu.rename") }}</span>
            </button>
        </div>

        <div class="ribbon-sep"></div>

        <!-- View group -->
        <div class="ribbon-group">
            <button
                class="ribbon-btn"
                :title="t('contextMenu.selectAll')"
                @click="$emit('action', 'selectAll')"
            >
                <svg viewBox="0 0 20 20" fill="none">
                    <rect
                        x="3"
                        y="3"
                        width="14"
                        height="14"
                        rx="2"
                        stroke="currentColor"
                        stroke-width="1.5"
                    />
                    <path
                        d="M6.5 10l2.5 2.5 4.5-5"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    />
                </svg>
                <span class="ribbon-label">{{
                    t("contextMenu.selectAll")
                }}</span>
            </button>
            <button
                class="ribbon-btn"
                :title="t('contextMenu.refresh')"
                @click="$emit('action', 'refresh')"
            >
                <svg viewBox="0 0 20 20" fill="none">
                    <path
                        d="M14 2.5L17 8.5h-5M6 17.5L3 11.5h5"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    />
                    <path
                        d="M3 10a7 7 0 0113.9-1M17 10a7 7 0 01-13.9 1"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                    />
                </svg>
                <span class="ribbon-label">{{ t("contextMenu.refresh") }}</span>
            </button>
            <!-- View mode buttons -->
            <button
                class="ribbon-btn"
                :class="{ active: store.viewMode === 'details' }"
                :title="t('fileList.details')"
                @click="store.setViewMode('details')"
            >
                <svg viewBox="0 0 20 20" fill="none">
                    <rect
                        x="2"
                        y="3"
                        width="16"
                        height="3"
                        rx="1"
                        fill="currentColor"
                        opacity="0.7"
                    />
                    <rect
                        x="2"
                        y="8"
                        width="16"
                        height="3"
                        rx="1"
                        fill="currentColor"
                        opacity="0.3"
                    />
                    <rect
                        x="2"
                        y="13"
                        width="16"
                        height="3"
                        rx="1"
                        fill="currentColor"
                        opacity="0.3"
                    />
                </svg>
                <span class="ribbon-label">{{ t("fileList.details") }}</span>
            </button>
            <button
                class="ribbon-btn"
                :class="{ active: store.viewMode === 'list' }"
                :title="t('fileList.list')"
                @click="store.setViewMode('list')"
            >
                <svg viewBox="0 0 20 20" fill="none">
                    <rect
                        x="2"
                        y="4"
                        width="10"
                        height="2"
                        rx="0.5"
                        fill="currentColor"
                        opacity="0.7"
                    />
                    <rect
                        x="2"
                        y="9"
                        width="10"
                        height="2"
                        rx="0.5"
                        fill="currentColor"
                        opacity="0.3"
                    />
                    <rect
                        x="2"
                        y="14"
                        width="10"
                        height="2"
                        rx="0.5"
                        fill="currentColor"
                        opacity="0.3"
                    />
                </svg>
                <span class="ribbon-label">{{ t("fileList.list") }}</span>
            </button>
            <button
                class="ribbon-btn"
                :class="{ active: store.viewMode === 'grid' }"
                :title="t('fileList.grid')"
                @click="store.setViewMode('grid')"
            >
                <svg viewBox="0 0 20 20" fill="none">
                    <rect
                        x="2"
                        y="2"
                        width="7"
                        height="7"
                        rx="1"
                        fill="currentColor"
                        opacity="0.5"
                    />
                    <rect
                        x="11"
                        y="2"
                        width="7"
                        height="7"
                        rx="1"
                        fill="currentColor"
                        opacity="0.2"
                    />
                    <rect
                        x="2"
                        y="11"
                        width="7"
                        height="7"
                        rx="1"
                        fill="currentColor"
                        opacity="0.2"
                    />
                </svg>
                <span class="ribbon-label">{{ t("fileList.grid") }}</span>
            </button>
            <!-- Tree view -->
            <button
                class="ribbon-btn"
                :class="{ active: store.viewMode === 'tree' }"
                :title="t('fileList.tree')"
                @click="switchToTree"
            >
                <svg viewBox="0 0 20 20" fill="none">
                    <rect
                        x="3"
                        y="3"
                        width="4"
                        height="2"
                        rx="0.5"
                        fill="currentColor"
                        opacity="0.7"
                    />
                    <rect
                        x="9"
                        y="3"
                        width="8"
                        height="2"
                        rx="0.5"
                        fill="currentColor"
                        opacity="0.4"
                    />
                    <rect
                        x="3"
                        y="8"
                        width="4"
                        height="2"
                        rx="0.5"
                        fill="currentColor"
                        opacity="0.7"
                    />
                    <rect
                        x="9"
                        y="8"
                        width="8"
                        height="2"
                        rx="0.5"
                        fill="currentColor"
                        opacity="0.4"
                    />
                    <rect
                        x="3"
                        y="13"
                        width="4"
                        height="2"
                        rx="0.5"
                        fill="currentColor"
                        opacity="0.7"
                    />
                    <rect
                        x="9"
                        y="13"
                        width="8"
                        height="2"
                        rx="0.5"
                        fill="currentColor"
                        opacity="0.4"
                    />
                    <path
                        d="M5 5v3M5 10v3"
                        stroke="currentColor"
                        stroke-width="1.2"
                        opacity="0.5"
                    />
                </svg>
                <span class="ribbon-label">{{ t("fileList.tree") }}</span>
            </button>
            <!-- Column view -->
            <button
                class="ribbon-btn"
                :class="{ active: store.viewMode === 'column' }"
                :title="t('fileList.column')"
                @click="store.setViewMode('column')"
            >
                <svg viewBox="0 0 20 20" fill="none">
                    <rect
                        x="2"
                        y="3"
                        width="5"
                        height="14"
                        rx="0.5"
                        fill="currentColor"
                        opacity="0.15"
                    />
                    <rect
                        x="2"
                        y="3"
                        width="5"
                        height="3"
                        rx="0.5"
                        fill="currentColor"
                        opacity="0.4"
                    />
                    <rect
                        x="8"
                        y="3"
                        width="5"
                        height="14"
                        rx="0.5"
                        fill="currentColor"
                        opacity="0.15"
                    />
                    <rect
                        x="8"
                        y="3"
                        width="5"
                        height="3"
                        rx="0.5"
                        fill="currentColor"
                        opacity="0.35"
                    />
                    <rect
                        x="14"
                        y="3"
                        width="4"
                        height="14"
                        rx="0.5"
                        fill="currentColor"
                        opacity="0.12"
                    />
                    <rect
                        x="14"
                        y="3"
                        width="4"
                        height="3"
                        rx="0.5"
                        fill="currentColor"
                        opacity="0.3"
                    />
                </svg>
                <span class="ribbon-label">{{ t("fileList.column") }}</span>
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";

const { t } = useI18n();
const store = useFileStore();

const emit = defineEmits<{ action: [action: string] }>();

function switchToTree() {
    store.setViewMode("tree");
    store.collapseAllTree();
}

const hasSelection = computed(() => store.selectedFiles.size > 0);
const selectedCount = computed(() => store.selectedFiles.size);
</script>

<style scoped>
.ribbon {
    display: flex;
    align-items: center;
    padding: 2px 8px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    gap: 2px;
    height: 48px;
    -webkit-app-region: no-drag;
    overflow-x: auto;
}

.ribbon-group {
    display: flex;
    align-items: center;
    gap: 2px;
}

.ribbon-sep {
    width: 1px;
    height: 28px;
    background: var(--border);
    margin: 0 4px;
    flex-shrink: 0;
}

.ribbon-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1px;
    padding: 2px 10px;
    height: 42px;
    min-width: 52px;
    border-radius: 6px;
    border: none;
    background: transparent;
    color: var(--text-primary);
    cursor: pointer;
    transition: background 0.1s;
    font-size: var(--font-size-xs);
    white-space: nowrap;
    position: relative;
}

.ribbon-btn:hover {
    background: var(--bg-hover);
}

.ribbon-btn:active {
    background: var(--bg-selected);
}

.ribbon-btn:disabled {
    opacity: 0.35;
    cursor: default;
}

.ribbon-btn:disabled:hover {
    background: transparent;
}

.ribbon-btn svg {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
}

.ribbon-btn.active {
    background: var(--bg-selected);
    color: var(--accent);
}

.ribbon-label {
    font-size: 11px;
    color: var(--text-secondary);
    line-height: 1;
}
</style>
