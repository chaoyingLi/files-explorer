<template>
    <div
        v-if="visible"
        class="terminal-panel"
        :class="{ 'terminal-maximized': isMaximized }"
        :style="isMaximized ? maximizeStyle : { height: height + 'px' }"
        @click="showTermMenu = false"
        @contextmenu.prevent.stop="onTermContextMenu"
    >
        <div class="terminal-resize-handle" @mousedown="onResizeStart">
            <div class="resize-hint" />
        </div>

        <!-- Tab bar + Header (merged) -->
        <div class="terminal-tab-bar">
            <div class="tab-bar-left">
                <button
                    v-for="tab in tabs"
                    :key="tab.id"
                    class="terminal-tab"
                    :class="{ active: tab.id === activeTabId }"
                    @click="switchTab(tab.id)"
                    @dblclick="toggleMaximize"
                >
                    <svg
                        class="tab-icon"
                        viewBox="0 0 16 16"
                        width="14"
                        height="14"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="1.3"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        <rect x="1.5" y="2.5" width="13" height="11" rx="1.2" />
                        <path d="M4.5 6l2.5 2-2.5 2M8 10h3.5" />
                    </svg>
                    <span>{{ tabDisplayName(tab) }}</span>
                    <span class="tab-close" @click.stop="closeTab(tab.id)"
                        >&times;</span
                    >
                </button>
                <button
                    class="tab-add"
                    :title="$t('terminal.newTerminal')"
                    @click="addTab"
                >
                    +
                </button>
            </div>

            <span class="tab-bar-cwd" @dblclick="toggleMaximize">{{
                activeTab?.cwd || ""
            }}</span>

            <div class="tab-bar-actions">
                <span
                    v-if="!activeTab?.terminalReady && !activeTab?.exited"
                    class="terminal-dot starting"
                    :title="$t('terminal.starting')"
                />
                <span
                    v-else-if="activeTab?.exited"
                    class="terminal-dot exited"
                    :title="$t('terminal.exited')"
                />
                <span v-else class="terminal-dot running" title="运行中" />
                <span class="terminal-font-btns">
                    <button
                        class="terminal-btn"
                        title="缩小 (Ctrl+-\)"
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
                    <span v-if="isMaximized">⧉</span>
                    <span v-else>⛶</span>
                </button>
                <button
                    class="terminal-btn"
                    :title="$t('terminal.restart')"
                    @click="restartTerminal(activeTabId!)"
                >
                    <span>⟳</span>
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
            <div
                v-for="tab in tabs"
                :key="tab.id"
                v-show="tab.id === activeTabId"
                class="terminal-body-outer"
            >
                <div
                    v-if="!tab.terminalReady && !tab.exited"
                    class="terminal-overlay"
                >
                    <div class="loading-spinner" />
                    <span>{{ $t("terminal.starting") }}</span>
                </div>
                <div v-else-if="tab.exited" class="terminal-overlay">
                    <span>{{ $t("terminal.exitedHint") }}</span>
                    <button
                        class="overlay-btn"
                        @click="restartTerminal(tab.id)"
                    >
                        {{ $t("terminal.restart") }}
                    </button>
                </div>
                <div v-else-if="tab.spawnError" class="terminal-overlay">
                    <span>{{ $t("terminal.spawnFailed") }}</span>
                    <button
                        class="overlay-btn"
                        @click="restartTerminal(tab.id)"
                    >
                        {{ $t("terminal.restart") }}
                    </button>
                </div>
                <div
                    :ref="(el) => setTabContainer(tab.id, el)"
                    class="terminal-body"
                />
            </div>
        </div>

        <ContextMenu
            v-if="showTermMenu"
            :x="termMenuPos.x"
            :y="termMenuPos.y"
            :items="termMenuItems"
            @close="showTermMenu = false"
            @action="onTermMenuAction"
        />
    </div>
</template>

<script setup lang="ts">
import { ref, watch, onUnmounted, nextTick, computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import {
    useSettingsStore,
    type ThemeMode,
    FONT_FAMILY_MAP,
} from "@/stores/settingsStore";
import type { ContextMenuOption } from "@/types";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import ContextMenu from "./ContextMenu.vue";

interface TerminalTab {
    id: number;
    shellName: string;
    cwd: string;
    terminalReady: boolean;
    exited: boolean;
    spawnError: boolean;
    term: Terminal | null;
    fit: FitAddon | null;
}

const { t } = useI18n();
const props = defineProps<{ visible: boolean; height: number }>();
const emit = defineEmits<{
    close: [];
    "update:height": [h: number];
    "update:maximized": [v: boolean];
}>();

const store = useFileStore();
const settings = useSettingsStore();

const tabs = ref<TerminalTab[]>([]);
const activeTabId = ref<number | null>(null);
const nextTabId = ref(1);
const isMaximized = ref(false);
const fontSize = ref<number>(settings.termFontSize);
const _savedHeight = ref(250);
const fontSizeChanged = ref(false);
const maximizeStyle = ref<Record<string, string>>({});

// Container elements map (id → HTMLDivElement)
const tabContainers = new Map<number, HTMLDivElement>();
function setTabContainer(id: number, el: any) {
    if (el) tabContainers.set(id, el as HTMLDivElement);
    else tabContainers.delete(id);
}

const activeTab = computed(
    () => tabs.value.find((t) => t.id === activeTabId.value) || null,
);

function tabDisplayName(tab: TerminalTab): string {
    const sameName = tabs.value.filter((t) => t.shellName === tab.shellName);
    if (sameName.length > 1) {
        const idx = sameName.indexOf(tab) + 1;
        return `${tab.shellName} #${idx}`;
    }
    return tab.shellName;
}

// ── Tab management ──

async function addTab() {
    const id = nextTabId.value++;
    const cwd = store.currentPath || "/";
    const parts = cwd.replace(/\\/g, "/").split("/");
    const dirName = parts[parts.length - 1] || cwd;
    // Load shell name
    let shellName = "";
    try {
        shellName = await invoke<string>("get_default_shell");
    } catch {
        shellName = "";
    }
    const sh =
        shellName.replace(/\\/g, "/").split("/").pop() || shellName || "sh";

    const tab: TerminalTab = {
        id,
        shellName: sh,
        cwd: dirName,
        terminalReady: false,
        exited: false,
        spawnError: false,
        term: null,
        fit: null,
    };
    tabs.value.push(tab);
    activeTabId.value = id;

    await nextTick();
    initTerminal(tab);
    await spawnTerminal(tab);
    await nextTick();
    resizeTerminal(id);
}

function closeTab(id: number) {
    const tab = tabs.value.find((t) => t.id === id);
    if (!tab) return;
    invoke("terminal_kill", { id }).catch(() => {});
    tab.term?.dispose();
    tab.term = null;
    tab.fit = null;
    tabContainers.delete(id);
    tabs.value = tabs.value.filter((t) => t.id !== id);

    if (tabs.value.length === 0) {
        emit("close");
    } else if (activeTabId.value === id) {
        // Switch to last tab
        activeTabId.value = tabs.value[tabs.value.length - 1].id;
        nextTick(() => resizeTerminal(activeTabId.value!));
    }
}

function switchTab(id: number) {
    if (activeTabId.value === id) return;
    activeTabId.value = id;
    nextTick(() => resizeTerminal(id));
}

async function restartTerminal(id: number) {
    const tab = tabs.value.find((t) => t.id === id);
    if (!tab) return;
    tab.term?.reset();
    tab.terminalReady = false;
    tab.exited = false;
    // Sync cwd from current path
    const p = store.currentPath || "/";
    const parts = p.replace(/\\/g, "/").split("/");
    tab.cwd = parts[parts.length - 1] || p;
    await invoke("terminal_kill", { id }).catch(() => {});
    await spawnTerminal(tab);
}

// ── Terminal lifecycle per tab ──

function initTerminal(tab: TerminalTab) {
    const container = tabContainers.get(tab.id);
    if (!container) return;
    const th = currentTermTheme();
    const term = new Terminal({
        cursorBlink: true,
        fontSize: settings.termFontSize,
        fontFamily: FONT_FAMILY_MAP[settings.termFontFamily],
        theme: th as any,
        allowProposedApi: true,
    });
    const fit = new FitAddon();
    term.loadAddon(fit);
    term.open(container);
    fit.fit();
    // Middle-click paste
    container.addEventListener("auxclick", (e: MouseEvent) => {
        if (e.button === 1) {
            e.preventDefault();
            navigator.clipboard.readText().then((text) => {
                if (text)
                    invoke("terminal_write", {
                        id: tab.id,
                        data: encodeBase64(new TextEncoder().encode(text)),
                    }).catch(() => {});
            });
        }
    });
    // Keyboard handler
    term.attachCustomKeyEventHandler((e: KeyboardEvent): boolean => {
        if (e.type !== "keydown") return true;
        const ctrl = e.ctrlKey || e.metaKey;
        if (ctrl && e.key === "c") {
            if (term.hasSelection()) return true;
            e.preventDefault();
            invoke("terminal_write", { id: tab.id, data: btoa("\x03") }).catch(
                () => {},
            );
            return false;
        }
        if (ctrl && e.key === "v") {
            e.preventDefault();
            navigator.clipboard.readText().then((text) => {
                if (text)
                    invoke("terminal_write", {
                        id: tab.id,
                        data: encodeBase64(new TextEncoder().encode(text)),
                    }).catch(() => {});
            });
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
    // Data output
    term.onData((data) =>
        invoke("terminal_write", {
            id: tab.id,
            data: encodeBase64(new TextEncoder().encode(data)),
        }).catch(() => {}),
    );

    tab.term = term;
    tab.fit = fit;
}

async function spawnTerminal(tab: TerminalTab) {
    tab.spawnError = false;
    try {
        const timeout = setTimeout(() => {
            const t = tabs.value.find((t) => t.id === tab.id);
            if (t && !t.terminalReady && !t.exited) {
                t.spawnError = true;
            }
        }, 10000);
        await invoke("terminal_spawn", {
            id: tab.id,
            cwd: store.currentPath || "/",
            termType: settings.termEmulation,
        });
        clearTimeout(timeout);
    } catch (e) {
        const t = tabs.value.find((t) => t.id === tab.id);
        if (t) t.spawnError = true;
        console.error(e);
    }
}

async function resizeTerminal(id: number) {
    const tab = tabs.value.find((t) => t.id === id);
    if (tab?.fit) {
        try {
            tab.fit.fit();
        } catch {}
    }
    if (tab?.term) {
        invoke("terminal_resize", {
            id,
            rows: tab.term.rows,
            cols: tab.term.cols,
        }).catch(() => {});
    }
}

// ── Global event routing ──

let globalUnlistens: (() => void)[] = [];

async function setupGlobalListeners() {
    type TermPayload = { id: number; data?: string };

    const u1 = await listen<TermPayload>("terminal-output", (event) => {
        const { id, data } = event.payload;
        const tab = tabs.value.find((t) => t.id === id);
        if (tab?.term && data) {
            const b = atob(data);
            const arr = new Uint8Array(b.length);
            for (let i = 0; i < b.length; i++) arr[i] = b.charCodeAt(i);
            tab.term.write(arr);
        }
    });
    const u2 = await listen<TermPayload>("terminal-ready", (event) => {
        const { id } = event.payload;
        const tab = tabs.value.find((t) => t.id === id);
        if (tab) {
            tab.terminalReady = true;
            tab.exited = false;
            tab.spawnError = false;
        }
    });
    const u3 = await listen<TermPayload>("terminal-exit", (event) => {
        const { id } = event.payload;
        const tab = tabs.value.find((t) => t.id === id);
        if (tab) {
            tab.terminalReady = false;
            tab.exited = true;
        }
    });
    globalUnlistens = [u1, u2, u3];
}

// ── Theme ──

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
        brightGreen: "#b8e6b3",
        brightYellow: "#fae3b0",
        brightBlue: "#9fc6f8",
        brightMagenta: "#f5cde0",
        brightCyan: "#a6e8da",
        brightWhite: "#d0d6e6",
    },
    light: {
        background: "#eff1f5",
        foreground: "#4c4f69",
        cursor: "#dc8a78",
        selectionBackground: "#acb0be",
        black: "#5c5f77",
        red: "#d20f39",
        green: "#40a02b",
        yellow: "#df8e1d",
        blue: "#1e66f5",
        magenta: "#ea76cb",
        cyan: "#179299",
        white: "#acb0be",
        brightBlack: "#6c6f85",
        brightRed: "#d20f39",
        brightGreen: "#40a02b",
        brightYellow: "#df8e1d",
        brightBlue: "#1e66f5",
        brightMagenta: "#ea76cb",
        brightCyan: "#179299",
        brightWhite: "#bcc0cc",
    },
    nord: {
        background: "#2e3440",
        foreground: "#d8dee9",
        cursor: "#88c0d0",
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
        brightRed: "#bf616a",
        brightGreen: "#a3be8c",
        brightYellow: "#ebcb8b",
        brightBlue: "#81a1c1",
        brightMagenta: "#b48ead",
        brightCyan: "#8fbcbb",
        brightWhite: "#eceff4",
    },
    "tokyo-night": {
        background: "#1a1b26",
        foreground: "#a9b1d6",
        cursor: "#c0caf5",
        selectionBackground: "#364a82",
        black: "#414868",
        red: "#f7768e",
        green: "#73daca",
        yellow: "#e0af68",
        blue: "#7aa2f7",
        magenta: "#bb9af7",
        cyan: "#7dcfff",
        white: "#a9b1d6",
        brightBlack: "#565f89",
        brightRed: "#f7768e",
        brightGreen: "#73daca",
        brightYellow: "#e0af68",
        brightBlue: "#7aa2f7",
        brightMagenta: "#bb9af7",
        brightCyan: "#7dcfff",
        brightWhite: "#c0caf5",
    },
    "one-dark-pro": {
        background: "#282c34",
        foreground: "#abb2bf",
        cursor: "#528bff",
        selectionBackground: "#3e4451",
        black: "#5c6370",
        red: "#e06c75",
        green: "#98c379",
        yellow: "#d19a66",
        blue: "#61afef",
        magenta: "#c678dd",
        cyan: "#56b6c2",
        white: "#abb2bf",
        brightBlack: "#5c6370",
        brightRed: "#e06c75",
        brightGreen: "#98c379",
        brightYellow: "#d19a66",
        brightBlue: "#61afef",
        brightMagenta: "#c678dd",
        brightCyan: "#56b6c2",
        brightWhite: "#c8ccd4",
    },
    dracula: {
        background: "#282a36",
        foreground: "#f8f8f2",
        cursor: "#f8f8f2",
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
        brightRed: "#ff5555",
        brightGreen: "#50fa7b",
        brightYellow: "#f1fa8c",
        brightBlue: "#bd93f9",
        brightMagenta: "#ff79c6",
        brightCyan: "#8be9fd",
        brightWhite: "#ffffff",
    },
    "solarized-light": {
        background: "#fdf6e3",
        foreground: "#657b83",
        cursor: "#586e75",
        selectionBackground: "#eee8d5",
        black: "#073642",
        red: "#dc322f",
        green: "#859900",
        yellow: "#b58900",
        blue: "#268bd2",
        magenta: "#d33682",
        cyan: "#2aa198",
        white: "#eee8d5",
        brightBlack: "#002b36",
        brightRed: "#cb4b16",
        brightGreen: "#586e75",
        brightYellow: "#657b83",
        brightBlue: "#839496",
        brightMagenta: "#6c71c4",
        brightCyan: "#93a1a1",
        brightWhite: "#fdf6e3",
    },
};

function currentTermTheme() {
    return TERM_THEMES[settings.theme] || TERM_THEMES.dark;
}

watch(
    () => settings.theme,
    () => {
        for (const tab of tabs.value) {
            if (tab.term) {
                tab.term.options.theme = currentTermTheme() as any;
            }
        }
    },
);

watch(
    () => settings.termFontSize,
    (v) => {
        for (const tab of tabs.value) {
            if (tab.term) {
                tab.term.options.fontSize = v;
            }
        }
        nextTick(() => {
            for (const tab of tabs.value) {
                if (tab.fit)
                    try {
                        tab.fit.fit();
                    } catch {}
            }
        });
    },
);

watch(
    () => settings.termFontFamily,
    (v) => {
        const ff = FONT_FAMILY_MAP[v];
        for (const tab of tabs.value) {
            if (tab.term) {
                tab.term.options.fontFamily = ff;
            }
        }
        nextTick(() => {
            for (const tab of tabs.value) {
                if (tab.fit)
                    try {
                        tab.fit.fit();
                    } catch {}
            }
        });
    },
);

// Sync fontSize label with settings
watch(
    () => settings.termFontSize,
    (v) => {
        fontSize.value = v;
    },
);

// ── Encoding helper ──

function encodeBase64(bytes: Uint8Array): string {
    const CHUNK = 0x8000;
    let bin = "";
    for (let i = 0; i < bytes.length; i += CHUNK)
        bin += String.fromCharCode(...bytes.subarray(i, i + CHUNK));
    return btoa(bin);
}

// ── Zoom ──

function zoomIn() {
    fontSize.value = Math.min(24, fontSize.value + 1) as any;
    settings.setTermFontSize(fontSize.value as any);
    applyFontSize();
}
function zoomOut() {
    fontSize.value = Math.max(8, fontSize.value - 1) as any;
    settings.setTermFontSize(fontSize.value as any);
    applyFontSize();
}
function resetZoom() {
    fontSize.value = 13 as any;
    settings.setTermFontSize(13);
    applyFontSize();
}

function applyFontSize() {
    fontSizeChanged.value = true;
    for (const tab of tabs.value) {
        if (tab.term) {
            tab.term.options.fontSize = fontSize.value;
        }
    }
    nextTick(() => {
        for (const tab of tabs.value) {
            if (tab.fit) {
                try {
                    tab.fit.fit();
                } catch {}
            }
        }
    });
}

// ── Maximize ──

function toggleMaximize() {
    if (!isMaximized.value) {
        _savedHeight.value = props.height;
        const titlebar = document.querySelector(".titlebar");
        const statusbar = document.querySelector(".status-bar");
        const top = titlebar ? titlebar.getBoundingClientRect().bottom : 0;
        const bottom = statusbar
            ? window.innerHeight - statusbar.getBoundingClientRect().top
            : 0;
        maximizeStyle.value = {
            position: "fixed",
            top: top + "px",
            left: "0",
            right: "0",
            bottom: bottom + "px",
            zIndex: "999",
            height: "auto",
        };
    } else {
        maximizeStyle.value = {};
    }
    isMaximized.value = !isMaximized.value;
    emit("update:maximized", isMaximized.value);
    nextTick(() => {
        for (const tab of tabs.value) {
            resizeTerminal(tab.id);
        }
    });
}

// ── Resize handle ──

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
    if (activeTab.value?.fit) {
        try {
            activeTab.value.fit.fit();
        } catch {}
    }
}

function onResizeEnd() {
    document.removeEventListener("mousemove", onResizeMove);
    document.removeEventListener("mouseup", onResizeEnd);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    if (activeTabId.value !== null) {
        resizeTerminal(activeTabId.value);
    }
}

// ── Context menu ──

const showTermMenu = ref(false);
const termMenuPos = ref({ x: 0, y: 0 });

const termMenuItems = computed<ContextMenuOption[]>(() => [
    {
        label: t("terminal.newTerminal"),
        action: "newTerminal",
        shortcut: "Ctrl+Shift+T",
    },
    { label: "", action: "", separator: true },
    { label: t("terminal.copy"), action: "copy", shortcut: "Ctrl+Shift+C" },
    { label: t("terminal.paste"), action: "paste", shortcut: "Ctrl+Shift+V" },
    {
        label: t("terminal.selectAll"),
        action: "selectAll",
        shortcut: "Ctrl+Shift+A",
    },
    { label: "", action: "", separator: true },
    { label: t("terminal.clear"), action: "clear", shortcut: "Ctrl+Shift+K" },
    { label: "", action: "", separator: true },
    {
        label: t("terminal.closeTab"),
        action: "closeTab",
        shortcut: "Ctrl+Shift+W",
    },
]);

function onTermContextMenu(e: MouseEvent) {
    termMenuPos.value = { x: e.clientX, y: e.clientY };
    showTermMenu.value = true;
}

function onTermMenuAction(action: string) {
    showTermMenu.value = false;
    switch (action) {
        case "newTerminal":
            addTab();
            break;
        case "copy":
            if (activeTab.value?.term?.hasSelection()) {
                navigator.clipboard
                    .writeText(activeTab.value.term.getSelection())
                    .catch(() => {});
            }
            break;
        case "paste":
            navigator.clipboard.readText().then((text) => {
                if (text && activeTab.value)
                    invoke("terminal_write", {
                        id: activeTab.value.id,
                        data: encodeBase64(new TextEncoder().encode(text)),
                    }).catch(() => {});
            });
            break;
        case "selectAll":
            activeTab.value?.term?.selectAll();
            break;
        case "clear":
            activeTab.value?.term?.clear();
            break;
        case "closeTab":
            if (activeTabId.value !== null) closeTab(activeTabId.value);
            break;
    }
}

// ── Lifecycle ──

watch(
    () => props.visible,
    async (v) => {
        if (v) {
            await setupGlobalListeners();
            // Auto-create first tab
            if (tabs.value.length === 0) {
                await addTab();
            }
        } else {
            for (const u of globalUnlistens) u();
            globalUnlistens = [];
            invoke("terminal_kill_all").catch(() => {});
            for (const tab of tabs.value) {
                tab.term?.dispose();
            }
            tabs.value = [];
            activeTabId.value = null;
            isMaximized.value = false;
            emit("update:maximized", false);
        }
    },
);

// 终端不跟随目录导航，每个终端独立，cd 到哪里就保持在哪里

onMounted(() => {});
onUnmounted(() => {
    for (const u of globalUnlistens) u();
    globalUnlistens = [];
    invoke("terminal_kill_all").catch(() => {});
    for (const tab of tabs.value) {
        tab.term?.dispose();
    }
    tabs.value = [];
    activeTabId.value = null;
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

/* ── Tab bar (merged with header) ── */
.terminal-tab-bar {
    display: flex;
    align-items: center;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    padding: 0 4px;
    flex-shrink: 0;
    height: 32px;
    gap: 4px;
}
.tab-bar-left {
    display: flex;
    align-items: center;
    overflow-x: auto;
    gap: 2px;
    flex-shrink: 0;
}
.tab-bar-left::-webkit-scrollbar {
    height: 0;
}
.terminal-tab {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 10px;
    font-size: var(--font-size-sm);
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--text-muted);
    cursor: pointer;
    white-space: nowrap;
    transition: all 0.12s;
    flex-shrink: 0;
}
.terminal-tab:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
}
.terminal-tab.active {
    color: var(--text-primary);
    border-bottom-color: var(--accent);
    background: var(--bg-primary);
}
.tab-close {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    border-radius: 3px;
    font-size: 14px;
    line-height: 1;
    color: var(--text-muted);
    transition: all 0.1s;
}
.tab-close:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
}
.tab-icon {
    flex-shrink: 0;
    opacity: 0.7;
}
.terminal-tab.active .tab-icon {
    opacity: 1;
    color: var(--accent);
}
.tab-add {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    font-size: 18px;
    cursor: pointer;
    border-radius: 4px;
    flex-shrink: 0;
    margin-left: 2px;
    transition: all 0.12s;
}
.tab-add:hover {
    background: var(--bg-hover);
    color: var(--accent);
}

/* ── Cwd in tab bar ── */
.tab-bar-cwd {
    flex: 1;
    text-align: center;
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    padding: 0 8px;
    cursor: default;
    user-select: none;
}

/* ── Actions in tab bar ── */
.tab-bar-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
}

/* ── Shared header/action elements ── */
.terminal-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    display: inline-block;
}
.terminal-dot.running {
    background: #a6e3a1;
}
.terminal-dot.starting {
    background: #f9e2af;
    animation: blink 0.8s infinite;
}
.terminal-dot.exited {
    background: #f38ba8;
}
@keyframes blink {
    50% {
        opacity: 0.3;
    }
}
.terminal-font-btns {
    display: flex;
    align-items: center;
    gap: 4px;
}
.font-size-label {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    min-width: 24px;
    text-align: center;
}
.terminal-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: 4px;
    font-size: 16px;
    transition: all 0.12s;
}
.terminal-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
}

/* ── Terminal body ── */
.terminal-body-wrapper {
    flex: 1;
    position: relative;
    overflow: hidden;
}
.terminal-body-outer {
    position: absolute;
    inset: 0;
}
.terminal-body {
    width: 100%;
    height: 100%;
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
    font-size: var(--font-size-sm);
    z-index: 2;
}
.overlay-btn {
    padding: 6px 16px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg-secondary);
    color: var(--text-primary);
    cursor: pointer;
    font-size: var(--font-size-sm);
    transition: all 0.12s;
}
.overlay-btn:hover {
    background: var(--bg-hover);
}
.loading-spinner {
    width: 24px;
    height: 24px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
}
@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}

.terminal-body :deep(.xterm) {
    height: 100%;
}
.terminal-body :deep(.xterm-viewport) {
    scrollbar-width: thin;
}
</style>
