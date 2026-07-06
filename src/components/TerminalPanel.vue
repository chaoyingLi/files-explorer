<template>
    <div
        v-if="visible"
        class="terminal-panel"
        :class="{ 'terminal-maximized': isMaximized }"
        @contextmenu.prevent.stop
    >
        <div class="terminal-resize-handle" @mousedown="onResizeStart">
            <div class="resize-hint" />
        </div>
        <div class="terminal-header" @dblclick="toggleMaximize">
            <span class="terminal-title">
                <span class="shell-indicator">▸</span>
                <span class="shell-name">{{ shellName }}</span>
                <span class="shell-sep">｜</span>
                <span class="terminal-cwd">{{ cwd }}</span>
            </span>
            <div class="terminal-actions">
                <span
                    v-if="!terminalReady"
                    class="terminal-dot starting"
                    :title="$t('terminal.starting')"
                />
                <span
                    v-else-if="exited"
                    class="terminal-dot exited"
                    :title="$t('terminal.exited')"
                />
                <span v-else class="terminal-dot running" title="运行中" />
                <span class="terminal-font-btns">
                    <button
                        class="terminal-btn"
                        title="缩小 (Ctrl+-)"
                        @click="zoomOut"
                    >
                        −
                    </button>
                    <span class="font-size-label">{{ fontSize }}px</span>
                    <button
                        class="terminal-btn"
                        title="放大 (Ctrl+=)"
                        @click="zoomIn"
                    >
                        +
                    </button>
                </span>
                <button
                    class="terminal-btn"
                    :title="
                        isMaximized
                            ? $t('terminal.restore')
                            : $t('terminal.maximize')
                    "
                    @click="toggleMaximize"
                >
                    <svg
                        v-if="isMaximized"
                        viewBox="0 0 16 16"
                        width="14"
                        height="14"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="1.2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        <rect x="1" y="3" width="10" height="9" rx="1" />
                        <rect x="5" y="5" width="10" height="9" rx="1" />
                    </svg>
                    <svg
                        v-else
                        viewBox="0 0 16 16"
                        width="14"
                        height="14"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="1.2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        <rect x="2" y="3" width="11" height="10" rx="1" />
                        <path d="M13 3L8 8" />
                        <path d="M13 3v3M13 3h-3" />
                    </svg>
                </button>
                <button
                    class="terminal-btn"
                    :title="$t('terminal.restart')"
                    @click="restartTerminal"
                >
                    <svg
                        viewBox="0 0 16 16"
                        width="14"
                        height="14"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="1.2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        <path d="M2 8a6 6 0 0110.5-2.5" />
                        <polyline points="12,1 13.5,5 9.5,5" />
                    </svg>
                </button>
                <button
                    class="terminal-btn"
                    :title="$t('terminal.close') + ' (Ctrl+`)'"
                    @click="$emit('close')"
                >
                    <svg
                        viewBox="0 0 16 16"
                        width="14"
                        height="14"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="1.4"
                        stroke-linecap="round"
                    >
                        <line x1="4" y1="4" x2="12" y2="12" />
                        <line x1="12" y1="4" x2="4" y2="12" />
                    </svg>
                </button>
            </div>
        </div>
        <div class="terminal-body-wrapper">
            <div v-if="!terminalReady && !exited" class="terminal-overlay">
                <div class="loading-spinner" />
                <span>{{ $t("terminal.starting") }}</span>
            </div>
            <div v-else-if="exited" class="terminal-overlay">
                <span>{{ $t("terminal.exitedHint") }}</span>
                <button class="overlay-btn" @click="restartTerminal">
                    {{ $t("terminal.restart") }}
                </button>
            </div>
            <div ref="terminalContainer" class="terminal-body" />
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, watch, onUnmounted, nextTick, computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import { useSettingsStore, type ThemeMode } from "@/stores/settingsStore";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";

const { t } = useI18n();
const props = defineProps<{ visible: boolean; height: number }>();
const emit = defineEmits<{
    close: [];
    "update:height": [h: number];
    "update:maximized": [v: boolean];
}>();

const store = useFileStore();
const settings = useSettingsStore();
const terminalContainer = ref<HTMLDivElement>();
const terminalReady = ref(false);
const exited = ref(false);
const isMaximized = ref(false);
const fontSize = ref(13);
const _savedHeight = ref(250);

let term: Terminal | null = null;
let fit: FitAddon | null = null;
let unlistens: (() => void)[] = [];

const shellPath = ref("");
const shellName = computed(() => {
    if (!shellPath.value) return "zsh";
    const parts = shellPath.value.replace(/\\/g, "/").split("/");
    return parts[parts.length - 1] || shellPath.value;
});
async function loadShellName() {
    try {
        shellPath.value = await invoke<string>("get_default_shell");
    } catch {
        shellPath.value = "";
    }
}

const cwd = computed(() => {
    if (!store.currentPath) return "";
    const p = store.currentPath;
    if (p === "/") return "/";
    const parts = p.replace(/\\/g, "/").split("/");
    return parts[parts.length - 1] || p;
});

function toggleMaximize() {
    if (!isMaximized.value) _savedHeight.value = props.height;
    isMaximized.value = !isMaximized.value;
    emit("update:maximized", isMaximized.value);
    nextTick(() => resizeTerminal());
}

function zoomIn() {
    fontSize.value = Math.min(24, fontSize.value + 1);
    applyFontSize();
}
function zoomOut() {
    fontSize.value = Math.max(8, fontSize.value - 1);
    applyFontSize();
}
function resetZoom() {
    fontSize.value = 13;
    applyFontSize();
}
function applyFontSize() {
    if (term) {
        term.options.fontSize = fontSize.value;
        nextTick(() => resizeTerminal());
    }
}

// ── Full ANSI colour palettes per theme ──
interface TermTheme {
    background: string;
    foreground: string;
    cursor: string;
    selectionBackground: string;
    black: string;
    red: string;
    green: string;
    yellow: string;
    blue: string;
    magenta: string;
    cyan: string;
    white: string;
    brightBlack: string;
    brightRed: string;
    brightGreen: string;
    brightYellow: string;
    brightBlue: string;
    brightMagenta: string;
    brightCyan: string;
    brightWhite: string;
}

const TERM_THEMES: Record<ThemeMode, TermTheme> = {
    dark: {
        background: "#1e1e2e",
        foreground: "#cdd6f4",
        cursor: "#f5e0dc",
        selectionBackground: "#45475a",
        black: "#45475a",
        red: "#f38ba8",
        green: "#a6e3a1",
        yellow: "#f9e2af",
        blue: "#89b4fa",
        magenta: "#f5c2e7",
        cyan: "#94e2d5",
        white: "#bac2de",
        brightBlack: "#585b70",
        brightRed: "#f8b0c0",
        brightGreen: "#c0edbc",
        brightYellow: "#fceac0",
        brightBlue: "#a4c8fb",
        brightMagenta: "#f8d3ed",
        brightCyan: "#b0eae0",
        brightWhite: "#c6d0e8",
    },
    light: {
        background: "#eff1f5",
        foreground: "#4c4f69",
        cursor: "#dc8a78",
        selectionBackground: "#ccd0da",
        black: "#5c5f77",
        red: "#d20f39",
        green: "#40a02b",
        yellow: "#df8e1d",
        blue: "#1e66f5",
        magenta: "#ea76cb",
        cyan: "#179299",
        white: "#acb0be",
        brightBlack: "#6c6f85",
        brightRed: "#e64553",
        brightGreen: "#54b838",
        brightYellow: "#e69d3a",
        brightBlue: "#3d7af7",
        brightMagenta: "#ee8dd4",
        brightCyan: "#22a6ad",
        brightWhite: "#bcc0cc",
    },
    nord: {
        background: "#2e3440",
        foreground: "#d8dee9",
        cursor: "#81a1c1",
        selectionBackground: "#434c5e",
        black: "#3b4252",
        red: "#bf616a",
        green: "#a3be8c",
        yellow: "#ebcb8b",
        blue: "#81a1c1",
        magenta: "#b48ead",
        cyan: "#88c0d0",
        white: "#e5e9f0",
        brightBlack: "#4c566a",
        brightRed: "#cc737b",
        brightGreen: "#b4c9a0",
        brightYellow: "#efd49c",
        brightBlue: "#94b0cc",
        brightMagenta: "#c19fba",
        brightCyan: "#9acbd8",
        brightWhite: "#eceff4",
    },
    "tokyo-night": {
        background: "#1a1b26",
        foreground: "#c0caf5",
        cursor: "#7aa2f7",
        selectionBackground: "#33467c",
        black: "#32344a",
        red: "#f7768e",
        green: "#9ece6a",
        yellow: "#e0af68",
        blue: "#7aa2f7",
        magenta: "#ad8ee6",
        cyan: "#449dab",
        white: "#787c99",
        brightBlack: "#444b6a",
        brightRed: "#ff8da1",
        brightGreen: "#b9f27c",
        brightYellow: "#ffc07a",
        brightBlue: "#8fb4ff",
        brightMagenta: "#c0a4f0",
        brightCyan: "#3db5c7",
        brightWhite: "#acb0d0",
    },
    "one-dark-pro": {
        background: "#282c34",
        foreground: "#abb2bf",
        cursor: "#528bff",
        selectionBackground: "#3e4452",
        black: "#3f4451",
        red: "#e05561",
        green: "#8cc265",
        yellow: "#d18f52",
        blue: "#4aa5f0",
        magenta: "#c162de",
        cyan: "#42b3c2",
        white: "#d7dae0",
        brightBlack: "#4f5666",
        brightRed: "#ff6b77",
        brightGreen: "#a5e075",
        brightYellow: "#f0a45d",
        brightBlue: "#61baff",
        brightMagenta: "#de75ff",
        brightCyan: "#5bc8d6",
        brightWhite: "#e6e6e6",
    },
    dracula: {
        background: "#282a36",
        foreground: "#f8f8f2",
        cursor: "#bd93f9",
        selectionBackground: "#44475a",
        black: "#21222c",
        red: "#ff5555",
        green: "#50fa7b",
        yellow: "#f1fa8c",
        blue: "#bd93f9",
        magenta: "#ff79c6",
        cyan: "#8be9fd",
        white: "#f8f8f2",
        brightBlack: "#6272a4",
        brightRed: "#ff6e6e",
        brightGreen: "#69ff94",
        brightYellow: "#ffffa5",
        brightBlue: "#d6acff",
        brightMagenta: "#ff92df",
        brightCyan: "#a4ffff",
        brightWhite: "#ffffff",
    },
    "solarized-light": {
        background: "#fdf6e3",
        foreground: "#586e75",
        cursor: "#268bd2",
        selectionBackground: "#eee8d5",
        black: "#002b36",
        red: "#dc322f",
        green: "#859900",
        yellow: "#b58900",
        blue: "#268bd2",
        magenta: "#6c71c4",
        cyan: "#2aa198",
        white: "#93a1a1",
        brightBlack: "#073642",
        brightRed: "#e04542",
        brightGreen: "#97ad08",
        brightYellow: "#c89b08",
        brightBlue: "#3d98dc",
        brightMagenta: "#7e83ce",
        brightCyan: "#32b3ab",
        brightWhite: "#fdf6e3",
    },
};

function currentTermTheme() {
    return TERM_THEMES[settings.theme] || TERM_THEMES.dark;
}
function applyTermTheme() {
    if (!term) return;
    const th = currentTermTheme();
    term.options.theme = th as any;
}
watch(
    () => settings.theme,
    () => applyTermTheme(),
);

function encodeBase64(bytes: Uint8Array): string {
    const CHUNK = 0x8000;
    let bin = "";
    for (let i = 0; i < bytes.length; i += CHUNK)
        bin += String.fromCharCode(...bytes.subarray(i, i + CHUNK));
    return btoa(bin);
}

function setupKeyboardHandler() {
    if (!term) return;
    term.attachCustomKeyEventHandler((e: KeyboardEvent): boolean => {
        if (e.type !== "keydown") return true;
        const ctrl = e.ctrlKey || e.metaKey;
        if (ctrl && e.key === "c") {
            if (term?.hasSelection()) return true;
            e.preventDefault();
            invoke("terminal_write", { data: btoa("\x03") }).catch(() => {});
            return false;
        }
        if (ctrl && e.key === "v") {
            e.preventDefault();
            navigator.clipboard.readText().then((text) =>
                invoke("terminal_write", {
                    data: encodeBase64(new TextEncoder().encode(text)),
                }).catch(() => {}),
            );
            return false;
        }
        if (ctrl && (e.key === "=" || e.key === "+")) {
            e.preventDefault();
            zoomIn();
            return false;
        }
        if (ctrl && e.key === "-") {
            e.preventDefault();
            zoomOut();
            return false;
        }
        if (ctrl && e.key === "0") {
            e.preventDefault();
            resetZoom();
            return false;
        }
        if (e.key === "Escape" && isMaximized.value) {
            e.preventDefault();
            toggleMaximize();
            return false;
        }
        return true;
    });
}

let _ro: ResizeObserver | null = null;

function initTerminal() {
    if (!terminalContainer.value) return;
    const th = currentTermTheme();
    term = new Terminal({
        cursorBlink: true,
        fontSize: fontSize.value,
        fontFamily: 'Menlo, "DejaVu Sans Mono", Consolas, monospace',
        theme: th as any,
        allowProposedApi: true,
    });
    fit = new FitAddon();
    term.loadAddon(fit);
    term.open(terminalContainer.value);
    fit.fit();
    setupKeyboardHandler();
    term.onData((data) =>
        invoke("terminal_write", {
            data: encodeBase64(new TextEncoder().encode(data)),
        }).catch(() => {}),
    );
}

async function spawnTerminal() {
    try {
        await invoke("terminal_spawn", { cwd: store.currentPath || "/" });
    } catch (e) {
        console.error(e);
    }
}

async function restartTerminal() {
    if (term) term.reset();
    exited.value = false;
    terminalReady.value = false;
    await invoke("terminal_kill").catch(() => {});
    await spawnTerminal();
}

async function setupListeners() {
    const u1 = await listen<string>("terminal-output", (event) => {
        if (term) {
            const b = atob(event.payload);
            const arr = new Uint8Array(b.length);
            for (let i = 0; i < b.length; i++) arr[i] = b.charCodeAt(i);
            term.write(arr);
        }
    });
    const u2 = await listen("terminal-ready", () => {
        terminalReady.value = true;
        exited.value = false;
    });
    const u3 = await listen("terminal-exit", () => {
        terminalReady.value = false;
        exited.value = true;
    });
    unlistens = [u1, u2, u3];
}
function cleanupListeners() {
    for (const u of unlistens) u();
    unlistens = [];
}

function resizeTerminal() {
    if (fit) {
        try {
            fit.fit();
        } catch {}
    }
    if (term)
        invoke("terminal_resize", { rows: term.rows, cols: term.cols }).catch(
            () => {},
        );
}

watch(
    () => props.visible,
    async (v) => {
        if (v) {
            await nextTick();
            initTerminal();
            await setupListeners();
            await spawnTerminal();
            await nextTick();
            resizeTerminal();
        } else {
            cleanupListeners();
            await invoke("terminal_kill").catch(() => {});
            if (_ro) {
                _ro.disconnect();
                _ro = null;
            }
            if (term) {
                term.dispose();
                term = null;
            }
            terminalReady.value = false;
            exited.value = false;
            isMaximized.value = false;
            emit("update:maximized", false);
        }
    },
);

watch(
    () => store.currentPath,
    async (newPath, oldPath) => {
        if (!props.visible || !newPath || newPath === oldPath) return;
        await restartTerminal();
    },
);

let _resizeStartY = 0,
    _resizeStartH = 0;
function onResizeStart(e: MouseEvent) {
    if (isMaximized.value) return;
    _resizeStartY = e.clientY;
    _resizeStartH = props.height;
    document.addEventListener("mousemove", onResizeMove);
    document.addEventListener("mouseup", onResizeEnd);
    document.body.style.cursor = "row-resize";
    document.body.style.userSelect = "none";
}
function onResizeMove(e: MouseEvent) {
    emit(
        "update:height",
        Math.max(100, Math.min(600, _resizeStartH + _resizeStartY - e.clientY)),
    );
    resizeTerminal();
}
function onResizeEnd() {
    document.removeEventListener("mousemove", onResizeMove);
    document.removeEventListener("mouseup", onResizeEnd);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    if (fit) {
        try {
            fit.fit();
        } catch {}
    }
}

watch(terminalContainer, (el) => {
    if (el && term) {
        _ro = new ResizeObserver(() => resizeTerminal());
        _ro.observe(el);
    }
});
onMounted(() => loadShellName());
onUnmounted(() => {
    cleanupListeners();
    if (_ro) {
        _ro.disconnect();
        _ro = null;
    }
    if (term) term.dispose();
});
</script>

<style scoped>
.terminal-panel {
    display: flex;
    flex-direction: column;
    background: var(--bg-primary);
    border-top: 1px solid var(--border);
    overflow: hidden;
    transition: all 0.15s ease;
}
.terminal-panel.terminal-maximized {
    position: fixed;
    inset: 0;
    z-index: 1000;
    border-top: none;
}
.terminal-resize-handle {
    height: 5px;
    cursor: row-resize;
    background: transparent;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s;
}
.terminal-maximized .terminal-resize-handle {
    display: none;
}
.resize-hint {
    width: 28px;
    height: 2px;
    background: var(--border);
    border-radius: 1px;
    opacity: 0.4;
    transition:
        opacity 0.15s,
        width 0.15s;
}
.terminal-resize-handle:hover .resize-hint {
    opacity: 1;
    width: 40px;
    background: var(--accent);
}
.terminal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 3px 10px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    min-height: 32px;
    user-select: none;
}
.terminal-title {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 12px;
    min-width: 0;
}
.shell-indicator {
    color: var(--accent);
    font-size: 10px;
    flex-shrink: 0;
}
.shell-name {
    color: var(--text-primary);
    font-weight: 600;
    flex-shrink: 0;
}
.shell-sep {
    color: var(--border);
    flex-shrink: 0;
}
.terminal-cwd {
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.terminal-actions {
    display: flex;
    align-items: center;
    gap: 3px;
    flex-shrink: 0;
}
.terminal-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    margin-right: 4px;
    flex-shrink: 0;
}
.terminal-dot.running {
    background: #a6e3a1;
}
.terminal-dot.starting {
    background: #f9e2af;
}
.terminal-dot.exited {
    background: #f38ba8;
}
.terminal-font-btns {
    display: flex;
    align-items: center;
    gap: 1px;
    margin-right: 4px;
    padding: 0 4px;
    border-left: 1px solid var(--border);
}
.font-size-label {
    font-size: 10px;
    color: var(--text-muted);
    min-width: 24px;
    text-align: center;
}
.terminal-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: 4px;
    font-size: 14px;
    line-height: 1;
}
.terminal-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
}
.terminal-body-wrapper {
    flex: 1;
    position: relative;
    min-height: 0;
}
.terminal-body {
    width: 100%;
    height: 100%;
    padding: 6px 10px;
}
.terminal-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    background: var(--bg-primary);
    color: var(--text-muted);
    font-size: 13px;
    z-index: 2;
}
.overlay-btn {
    padding: 4px 14px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-secondary);
    color: var(--text-primary);
    cursor: pointer;
    font-size: 12px;
}
.overlay-btn:hover {
    background: var(--bg-hover);
}
.loading-spinner {
    width: 20px;
    height: 20px;
    border: 2px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
}
@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}
.terminal-body ::-webkit-scrollbar {
    width: 6px;
}
.terminal-body ::-webkit-scrollbar-track {
    background: transparent;
}
.terminal-body ::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: 3px;
}
.terminal-body ::-webkit-scrollbar-thumb:hover {
    background: var(--text-muted);
}
</style>
