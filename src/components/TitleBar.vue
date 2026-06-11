<template>
    <div class="titlebar" data-tauri-drag-region>
        <div class="titlebar-left">
            <svg class="app-icon" viewBox="0 0 20 20" fill="none">
                <path
                    d="M3 6.5A1.5 1.5 0 014.5 5h3.3a1.5 1.5 0 011.2.6l1 1.4h5.5A1.5 1.5 0 0117 8.5v6a1.5 1.5 0 01-1.5 1.5h-11A1.5 1.5 0 013 14.5V5.5z"
                    fill="var(--accent)"
                    opacity="0.7"
                />
                <path
                    d="M3 7.5A1.5 1.5 0 014.5 6h3.3a1.5 1.5 0 011.2.6l1 1.4h5.5A1.5 1.5 0 0117 9.5v6a1.5 1.5 0 01-1.5 1.5h-11A1.5 1.5 0 013 14.5V5.5z"
                    fill="var(--accent)"
                />
            </svg>
            <span class="app-title">{{ t("app.title") }}</span>
        </div>
        <div class="titlebar-center">
            <span class="titlebar-path" v-if="store.currentPath">{{
                store.currentPath
            }}</span>
        </div>
        <div class="titlebar-right" data-tauri-drag-region="">
            <button
                class="titlebar-btn"
                @click="minimize"
                :title="t('titlebar.minimize')"
            >
                <svg viewBox="0 0 12 12">
                    <path
                        d="M2 6h8"
                        stroke="currentColor"
                        stroke-width="1.2"
                        stroke-linecap="round"
                    />
                </svg>
            </button>
            <button
                class="titlebar-btn"
                @click="toggleMaximize"
                :title="
                    isMaximized ? t('titlebar.restore') : t('titlebar.maximize')
                "
            >
                <svg v-if="isMaximized" viewBox="0 0 12 12">
                    <rect
                        x="1.5"
                        y="3.5"
                        width="7"
                        height="7"
                        rx="0.8"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="1.1"
                    />
                    <path
                        d="M3.5 3.5V2.5a1 1 0 011-1h5a1 1 0 011 1v5a1 1 0 01-1 1h-1"
                        stroke="currentColor"
                        stroke-width="1.1"
                        fill="none"
                    />
                </svg>
                <svg v-else viewBox="0 0 12 12">
                    <rect
                        x="1.5"
                        y="1.5"
                        width="9"
                        height="9"
                        rx="1"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="1.2"
                    />
                </svg>
            </button>
            <button
                class="titlebar-btn titlebar-close"
                @click="close"
                :title="t('titlebar.close')"
            >
                <svg viewBox="0 0 12 12">
                    <path
                        d="M2 2l8 8M10 2l-8 8"
                        stroke="currentColor"
                        stroke-width="1.2"
                        stroke-linecap="round"
                    />
                </svg>
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import { getCurrentWindow } from "@tauri-apps/api/window";

const { t } = useI18n();
const store = useFileStore();

const appWindow = getCurrentWindow();
const isMaximized = ref(false);

function minimize() {
    appWindow.minimize();
}
async function toggleMaximize() {
    await appWindow.toggleMaximize();
    isMaximized.value = await appWindow.isMaximized();
}
function close() {
    appWindow.close();
}

onMounted(async () => {
    isMaximized.value = await appWindow.isMaximized();
    const unlisten = await appWindow.onResized(async () => {
        isMaximized.value = await appWindow.isMaximized();
    });
    onUnmounted(() => {
        unlisten();
    });
});
</script>

<style scoped>
.titlebar {
    height: 32px;
    min-height: 32px;
    background: var(--bg-tertiary);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 6px;
    flex-shrink: 0;
    -webkit-app-region: drag;
    border-bottom: 1px solid var(--border);
    z-index: 100;
}

.titlebar-left {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 4px;
    min-width: 0;
}

.app-icon {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
}

.app-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    white-space: nowrap;
    letter-spacing: 0.3px;
}

.titlebar-center {
    flex: 1;
    text-align: center;
    overflow: hidden;
    padding: 0 16px;
}

.titlebar-path {
    font-size: 11px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.titlebar-right {
    display: flex;
    gap: 1px;
    -webkit-app-region: no-drag;
    align-items: center;
}

.titlebar-btn {
    width: 38px;
    height: 28px;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    color: var(--text-secondary);
    transition:
        background 0.1s,
        color 0.1s;
}

.titlebar-btn svg {
    width: 12px;
    height: 12px;
}

.titlebar-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
}

.titlebar-close:hover {
    background: var(--danger);
    color: white;
}
</style>
