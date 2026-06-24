<template>
    <div class="titlebar" data-tauri-drag-region @dblclick="toggleMaximize">
        <!-- macOS traffic lights (left) -->
        <div v-if="isMac" class="titlebar-left mac-traffic-lights">
            <button
                class="traffic-btn traffic-close"
                @click="close"
                :title="t('titlebar.close')"
            >
                <svg viewBox="0 0 12 12">
                    <path
                        d="M3 3l6 6M9 3l-6 6"
                        stroke="currentColor"
                        stroke-width="1.2"
                        stroke-linecap="round"
                    />
                </svg>
            </button>
            <button
                class="traffic-btn traffic-minimize"
                @click="minimize"
                :title="t('titlebar.minimize')"
            >
                <svg viewBox="0 0 12 12">
                    <path
                        d="M3 6h6"
                        stroke="currentColor"
                        stroke-width="1.3"
                        stroke-linecap="round"
                    />
                </svg>
            </button>
            <button
                class="traffic-btn traffic-maximize"
                @click="toggleMaximize"
                :title="
                    isMaximized ? t('titlebar.restore') : t('titlebar.maximize')
                "
            >
                <svg v-if="isMaximized" viewBox="0 0 12 12">
                    <rect
                        x="1.5"
                        y="3"
                        width="7"
                        height="7"
                        rx="0.8"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="1.1"
                    />
                    <path
                        d="M3.5 3V2.5a.8.8 0 01.8-.8h5a.8.8 0 01.8.8v5a.8.8 0 01-.8.8H8"
                        stroke="currentColor"
                        stroke-width="1.1"
                        fill="none"
                    />
                </svg>
                <svg v-else viewBox="0 0 12 12">
                    <path
                        d="M3 4.5h6v-1.8L4.8 3H3v1.5zM3 7.5h6v1.8L4.8 9H3V7.5z"
                        fill="currentColor"
                    />
                </svg>
            </button>
        </div>
        <!-- Non-mac: app icon + title (left) -->
        <div v-else class="titlebar-left">
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
        <div v-if="isMac" class="titlebar-right" data-tauri-drag-region="" />
        <div v-else class="titlebar-right" data-tauri-drag-region="">
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
import { getCurrentWindow, currentMonitor } from "@tauri-apps/api/window";
import { PhysicalPosition, PhysicalSize } from "@tauri-apps/api/dpi";

const { t } = useI18n();
const store = useFileStore();
const isMac = /Mac|iPod|iPhone|iPad/.test(navigator.platform);

const appWindow = getCurrentWindow();
const isMaximized = ref(false);
let savedPos: PhysicalPosition | null = null;
let savedSize: PhysicalSize | null = null;
let ignoreNextResize = false;

function minimize() {
    appWindow.minimize();
}

async function toggleMaximize() {
    try {
        if (isMaximized.value) {
            await appWindow.unmaximize();
        } else {
            await appWindow.maximize();
        }
        isMaximized.value = !isMaximized.value;
    } catch {
        // Fallback: manual maximize/restore
        await manualToggle();
    }
}

async function manualToggle() {
    if (isMaximized.value) {
        if (savedPos && savedSize) {
            ignoreNextResize = true;
            await appWindow.setPosition(savedPos);
            await appWindow.setSize(savedSize);
        }
        isMaximized.value = false;
    } else {
        savedPos = await appWindow.outerPosition();
        savedSize = await appWindow.outerSize();
        const mon = await currentMonitor();
        if (mon) {
            ignoreNextResize = true;
            if (isMac) {
                await appWindow.setPosition(
                    new PhysicalPosition(mon.position.x, mon.position.y + 25),
                );
                await appWindow.setSize(
                    new PhysicalSize(mon.size.width, mon.size.height - 85),
                );
            } else {
                await appWindow.setPosition(mon.position);
                await appWindow.setSize(mon.size);
            }
        }
        isMaximized.value = true;
    }
}

function close() {
    appWindow.close();
}

onMounted(async () => {
    // Sync state with native maximize
    try {
        isMaximized.value = await appWindow.isMaximized();
    } catch {
        /* ignore */
    }
    const unlisten = await appWindow.onResized(async () => {
        if (ignoreNextResize) {
            ignoreNextResize = false;
            return;
        }
        // If user resizes manually, check if still maximized
        try {
            isMaximized.value = await appWindow.isMaximized();
        } catch {
            /* ignore */
        }
    });
    onUnmounted(() => unlisten());
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
    padding: 0 8px;
    flex-shrink: 0;
    -webkit-app-region: drag;
    border-bottom: 1px solid var(--border);
    z-index: 100;
}

.mac-traffic-lights {
    gap: 8px;
    padding-left: 4px;
}
.traffic-btn {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    -webkit-app-region: no-drag;
    transition: opacity 0.15s;
}
.traffic-btn svg {
    width: 8px;
    height: 8px;
    opacity: 0;
    transition: opacity 0.15s;
}
.traffic-btn:hover svg {
    opacity: 1;
}
.traffic-close {
    background: #ed6a5e;
}
.traffic-close svg {
    color: #7b1e14;
}
.traffic-minimize {
    background: #f5bf4f;
}
.traffic-minimize svg {
    color: #8e630f;
}
.traffic-maximize {
    background: #61c454;
}
.traffic-maximize svg {
    color: #1d6e14;
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
