<template>
    <div class="status-bar" @contextmenu.prevent.stop>
        <div class="status-left">
            <button
                v-if="terminalMaximized"
                class="status-indicator"
                :title="$t('terminal.restore')"
                @click="$emit('restoreTerminal')"
            >
                <span class="indicator-dot" />
                {{ $t("terminal.maximized") }}
            </button>
            <span v-else>{{ statusText }}</span>
        </div>
        <div class="status-right">
            <button
                class="status-btn"
                :title="$t('statusBar.terminal') + ' (Ctrl+`)'"
                @click="$emit('toggleTerminal')"
            >
                <svg
                    viewBox="0 0 16 16"
                    width="12"
                    height="12"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.3"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <rect x="1.5" y="2.5" width="13" height="11" rx="1.2" />
                    <path d="M4.5 6l2.5 2-2.5 2M8 10h3.5" />
                </svg>
            </button>
            <button
                class="status-btn"
                :title="$t('properties.title')"
                @click="$emit('toggleProperties')"
            >
                <svg
                    viewBox="0 0 16 16"
                    width="12"
                    height="12"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.2"
                >
                    <rect x="2" y="2" width="8" height="12" rx="1" />
                    <line
                        x1="11.5"
                        y1="2"
                        x2="11.5"
                        y2="14"
                        stroke-linecap="round"
                    />
                </svg>
            </button>
            <span v-if="store.loading" class="loading-indicator">
                <div
                    class="loading-spinner"
                    style="width: 14px; height: 14px; border-width: 1.5px"
                ></div>
            </span>
            <span v-else>{{ itemCount }}</span>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import { displayPath } from "@/utils/platform";

const props = defineProps<{
    terminalMaximized?: boolean;
}>();

defineEmits<{
    toggleProperties: [];
    toggleTerminal: [];
    restoreTerminal: [];
}>();

const { t } = useI18n();
const store = useFileStore();

const statusText = computed(() => {
    if (!store.currentPath) return t("statusBar.thisPc");
    return displayPath(store.currentPath);
});

const itemCount = computed(() => {
    if (!store.currentPath) {
        return `${store.drives.length} ${t("statusBar.drives")}`;
    }
    return t("statusBar.items", { count: store.files.length });
});
</script>

<style scoped>
.status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 2px 12px;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border);
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    height: 24px;
    flex-shrink: 0;
}

.status-left {
    display: flex;
    align-items: center;
    gap: 8px;
    overflow: hidden;
}

.status-left span {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.status-right {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
}

.loading-indicator {
    display: flex;
    align-items: center;
}

.status-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: 3px;
    font-size: var(--font-size-lg);
    line-height: 1;
}

.status-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
}

.status-indicator {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 2px 8px;
    border: 1px solid var(--accent);
    border-radius: 3px;
    background: transparent;
    color: var(--accent);
    font-size: var(--font-size-sm);
    cursor: pointer;
}
.status-indicator:hover {
    background: var(--bg-hover);
}

.indicator-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
    flex-shrink: 0;
}
</style>
