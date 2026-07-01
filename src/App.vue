<template>
    <PreviewWindow v-if="isPreviewWindow" />
    <div
        v-else
        class="app-container"
        @click="ctx.closeContextMenu()"
        @contextmenu.prevent="onGlobalContextMenu"
    >
        <TitleBar @open-settings="showSettings = true" />
        <Toolbar
            @navigate-back="nav.toolbarBack"
            @navigate-forward="nav.toolbarForward"
            @navigate-up="nav.toolbarUp"
            @refresh="nav.toolbarRefresh"
            @navigate-address="nav.toolbarAddress"
            @search-submit="(q: string) => search.submitSearch(q)"
        />
        <RibbonToolbar @action="(a: string) => actions.executeAction(a)" />
        <div class="main-content">
            <Sidebar
                :width="sidebarWidth"
                @navigate="nav.sidebarNavigate"
                @navigate-home="nav.sidebarHome"
                @context-menu="handleSidebarContext"
                @resize-start="onSidebarResizeStart"
            />
            <div class="panes-area">
                <PaneNode
                    :node="tabStore.rootLayout"
                    :focused-pane-id="focusedPaneId"
                    @focus="nav.onPaneFocus"
                    @tab-click="
                        (pid: string, tid: string) =>
                            nav.onTabClick(pid, tid, search.cleanupSearch)
                    "
                    @tab-close="
                        (pid: string, tid: string) =>
                            nav.onTabClose(pid, tid, search.cleanupSearch)
                    "
                    @tab-new="(pid: string) => nav.onNewTab(pid)"
                    @pane-close="(pid: string) => onPaneClose(pid)"
                    @tab-drop="
                        (pid: string, tid: string, e: DragEvent) =>
                            dnd.onTabDrop(pid, tid, e)
                    "
                    @navigate="
                        (pid: string, path: string) =>
                            nav.navigatePane(pid, path)
                    "
                    @file-drop="dnd.handleFileDrop"
                />
            </div>
            <PropertiesPanel
                v-if="showProperties"
                :visible="showProperties"
                :width="propsWidth"
                @close="showProperties = false"
                @resize-start="onPropsResizeStart"
            />
        </div>
        <DeleteConfirmDialog
            v-if="del.showDeleteConfirm"
            :count="del.deleteTargetCount"
            :permanently="del.deletePermanently"
            @confirm="actions.handleDeleteConfirm"
            @cancel="del.cancelDelete()"
        />
        <StatusBar @toggle-properties="toggleProperties" />
        <ContextMenu
            v-if="ctx.showContextMenu.value"
            :x="ctx.contextMenuPos.value.x"
            :y="ctx.contextMenuPos.value.y"
            :items="contextMenuItems"
            @close="ctx.closeContextMenu()"
            @action="handleContextAction"
        />
        <NewItemDialog
            v-if="actions.showNewDialog.value"
            :type="actions.newDialogType.value"
            @close="actions.showNewDialog.value = false"
            @confirm="actions.handleNewItem"
        />
        <RenameDialog
            v-if="actions.showRenameDialog.value"
            :oldName="actions.renameTarget.value"
            @close="actions.showRenameDialog.value = false"
            @confirm="actions.handleRename"
        />
        <SettingsDialog v-if="showSettings" @close="showSettings = false" />
        <div v-if="toast.messages.value.length > 0" class="toast-container">
            <div
                v-for="msg in toast.messages.value"
                :key="msg.id"
                class="toast"
                :class="{ 'toast-error': msg.isError }"
            >
                <span v-if="msg.isError" class="toast-icon">⚠</span>
                {{ msg.text }}
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, onErrorCaptured } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import { useSelectionStore } from "@/stores/selectionStore";
import { useDeleteStore } from "@/stores/deleteStore";
import { useSettingsStore } from "@/stores/settingsStore";
import { useTabStore, type Tab } from "@/stores/tabStore";
import { useNavigationStore } from "@/stores/navigationStore";
import { useViewStore } from "@/stores/viewStore";
import * as tauri from "@/utils/tauri";
import {
    saveSession,
    loadSession,
    serializeLayout,
    deserializeLayout,
    type SessionSnapshot,
} from "@/utils/session";

import TitleBar from "@/components/TitleBar.vue";
import Toolbar from "@/components/Toolbar.vue";
import RibbonToolbar from "@/components/RibbonToolbar.vue";
import Sidebar from "@/components/Sidebar.vue";
import PaneNode from "@/components/PaneNode.vue";
import DeleteConfirmDialog from "@/components/Dialogs/DeleteConfirmDialog.vue";
import StatusBar from "@/components/StatusBar.vue";
import ContextMenu from "@/components/ContextMenu.vue";
import NewItemDialog from "@/components/Dialogs/NewItemDialog.vue";
import RenameDialog from "@/components/Dialogs/RenameDialog.vue";
import SettingsDialog from "@/components/Dialogs/SettingsDialog.vue";
import PropertiesPanel from "@/components/PropertiesPanel.vue";
import PreviewWindow from "@/components/PreviewWindow.vue";

import { useToast } from "@/composables/useToast";
import { useContextMenu } from "@/composables/useContextMenu";
import { useFileActions } from "@/composables/useFileActions";
import { usePanelNavigation } from "@/composables/usePanelNavigation";
import { useKeyboardShortcuts } from "@/composables/useKeyboardShortcuts";
import { useSearchService } from "@/composables/useSearchService";
import { useDragDrop } from "@/composables/useDragDrop";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

const { t } = useI18n();
const store = useFileStore();
const sel = useSelectionStore();
const del = useDeleteStore();
useSettingsStore();
const tabStore = useTabStore();
const navStore = useNavigationStore();
const view = useViewStore();
const showSettings = ref(false);
const showProperties = ref(true);

// ── Window resize state (captured by onMounted closures) ──
let _cachedWinWidth = 1200;
let _cachedWinHeight = 800;
let _unmaximizedWidth = 1200;
let _unmaximizedHeight = 800;

// ── Detect preview window mode ──
const isPreviewWindow = computed(() =>
    new URLSearchParams(window.location.search).has("preview"),
);

// ── Sidebar & Properties panel resize ──
const sidebarWidth = ref(loadPanelWidth("sidebar", 220));
const propsWidth = ref(loadPanelWidth("props", 260));

function loadPanelWidth(key: string, fallback: number): number {
    try {
        const raw = localStorage.getItem("app-panel-" + key);
        if (raw) {
            const v = JSON.parse(raw);
            if (typeof v === "number" && v >= 100) return v;
        }
    } catch {}
    return fallback;
}
function savePanelWidth(key: string, w: number) {
    localStorage.setItem("app-panel-" + key, JSON.stringify(w));
}

let _resizePanelKey = "";
let _resizeStartX = 0;
let _resizeStartW = 0;

function onSidebarResizeStart(e: MouseEvent) {
    _resizePanelKey = "sidebar";
    _resizeStartX = e.clientX;
    _resizeStartW = sidebarWidth.value;
    addEventListener("mousemove", onPanelResizeMove);
    addEventListener("mouseup", onPanelResizeEnd);
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
    e.preventDefault();
}

function onPropsResizeStart(e: MouseEvent) {
    _resizePanelKey = "props";
    _resizeStartX = e.clientX;
    _resizeStartW = propsWidth.value;
    addEventListener("mousemove", onPanelResizeMove);
    addEventListener("mouseup", onPanelResizeEnd);
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
    e.preventDefault();
}

function onPanelResizeMove(e: MouseEvent) {
    if (_resizePanelKey === "sidebar") {
        sidebarWidth.value = Math.max(
            160,
            _resizeStartW + e.clientX - _resizeStartX,
        );
    } else if (_resizePanelKey === "props") {
        propsWidth.value = Math.max(
            180,
            _resizeStartW - (e.clientX - _resizeStartX),
        );
    }
}

function onPanelResizeEnd() {
    removeEventListener("mousemove", onPanelResizeMove);
    removeEventListener("mouseup", onPanelResizeEnd);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    if (_resizePanelKey === "sidebar") {
        savePanelWidth("sidebar", sidebarWidth.value);
    } else if (_resizePanelKey === "props") {
        savePanelWidth("props", propsWidth.value);
    }
    _resizePanelKey = "";
}

const toast = useToast();
const ctx = useContextMenu();
const toggleProperties = () => {
    showProperties.value = !showProperties.value;
};
const actions = useFileActions(toast, toggleProperties);
const nav = usePanelNavigation(
    actions.saveFileStateToTab,
    actions.loadFileStateFromTab,
    t,
);
const search = useSearchService(actions.saveFileStateToTab, toast.show);
const dnd = useDragDrop(toast.show, t);

const focusedPaneId = computed(() => nav.focusedPaneId.value);
const contextMenuItems = computed(() => ctx.contextMenuItems.value);

// Global error boundary
onErrorCaptured((err, _instance, info) => {
    console.error("[FilesExplorer Error]", info, err);
    // Show user-friendly error toast
    toast.show(t("toast.error") + ": " + String(err).slice(0, 120), true);
    return false; // prevent propagation
});

useKeyboardShortcuts({
    onTabClose: (pid, tid) => nav.onTabClose(pid, tid, search.cleanupSearch),
    onTabClick: nav.onTabClick,
    showToast: toast.show,
    openNewDialog(type) {
        actions.newDialogType.value = type;
        actions.showNewDialog.value = true;
    },
    openRenameDialog(target) {
        actions.renameTarget.value = target;
        actions.showRenameDialog.value = true;
    },
    openSettings: () => {
        showSettings.value = true;
    },
    openProperties: () => {
        showProperties.value = !showProperties.value;
    },
});

function onGlobalContextMenu(e: MouseEvent) {
    // 不拦截来自属性面板的右键事件（属性面板有自己的右键菜单）
    const target = e.target as HTMLElement | null;
    if (target?.closest(".preview-panel")) return;
    ctx.openContextMenu(e.clientX, e.clientY);
}

function handleSidebarContext(path: string, event: MouseEvent) {
    ctx.sidebarContextPath.value = path || "";
    ctx.openContextMenu(event.clientX, event.clientY);
}

async function handleContextAction(action: string) {
    ctx.closeContextMenu();
    if (action === "openInPreviewWindow") {
        const path =
            sel.selectedFiles.size === 1 ? [...sel.selectedFiles][0] : null;
        if (path) {
            const { WebviewWindow } =
                await import("@tauri-apps/api/webviewWindow");
            const label = `preview-${path.replace(/[^a-zA-Z0-9]/g, "_")}`;
            try {
                const existing = await WebviewWindow.getByLabel(label);
                if (existing) {
                    await existing.setFocus();
                    return;
                }
            } catch {}
            const { width, height } = await tauri.getAdaptivePreviewSize();
            new WebviewWindow(label, {
                url: `/?preview=${encodeURIComponent(path)}`,
                title: path.split("/").pop() || path,
                width,
                height,
                minWidth: 640,
                minHeight: 400,
                decorations: false,
                resizable: true,
                center: true,
            });
        }
        return;
    }
    if (action === "showInExplorer") {
        const path =
            sel.selectedFiles.size === 1
                ? [...sel.selectedFiles][0]
                : store.currentPath;
        if (path) {
            try {
                await tauri.showInExplorer(path);
                toast.show("showInExplorer: " + path);
            } catch (e: any) {
                toast.show(t("toast.error") + ": " + e, true);
            }
        }
        return;
    }
    if (action.startsWith("split")) {
        const dir = action.replace("split", "").toLowerCase() as
            "left" | "right" | "up" | "down";
        let splitPath = ctx.sidebarContextPath.value;
        let splitTitle = "";
        if (!splitPath && sel.selectedFiles.size === 1) {
            const sf = store.files.find(
                (f) => f.path === [...sel.selectedFiles][0],
            );
            if (sf?.is_dir) {
                splitPath = [...sel.selectedFiles][0];
                splitTitle = sf.name;
            }
        }
        if (!splitPath) {
            splitPath = store.currentPath;
            splitTitle = store.currentDirectoryName;
        }
        ctx.sidebarContextPath.value = "";
        if (splitPath) {
            const fp = tabStore.getFocusedPane();
            if (fp) {
                const ot = tabStore.getFocusedTab();
                if (ot) actions.saveFileStateToTab(ot);
                tabStore.splitPane(fp.id, splitPath, splitTitle, dir);
                await store.navigateTo(splitPath, false);
                const nt = tabStore.getFocusedTab();
                if (nt) actions.saveFileStateToTab(nt);
            }
        }
    } else {
        await actions.executeAction(action);
    }
}

function onPaneClose(pid: string) {
    tabStore.closePane(pid);
    const fp = tabStore.getFocusedPane();
    if (fp) {
        nav.focusedPaneId.value = fp.id;
        const t = fp.tabs.find((x: Tab) => x.id === fp.activeTabId);
        if (t) {
            actions.loadFileStateFromTab(t);
            if (!t.path) store.loadDrives();
        }
    }
}

onMounted(async () => {
    // ── Register close handler EARLY — before any async operations ──
    let _closeUnlisten: (() => void) | null = null;
    try {
        const win = getCurrentWebviewWindow();
        _closeUnlisten = await win.onCloseRequested((event) => {
            _saveSessionNow();
            const s = useSettingsStore();
            if (s.showTray && !s.quitOnClose) {
                event.preventDefault();
                win.hide();
            }
        });
    } catch {
        /* Tauri API may not be available */
    }
    // ── Restore window size (three-tier fallback) ──
    let _restoredPath = "";
    let _sessionData: SessionSnapshot | null = null;
    try {
        const win = getCurrentWebviewWindow();
        let winWidth = 0;
        let winHeight = 0;

        // Tier 1: User-adjusted size (saved on every non-maximized resize)
        try {
            const winSizeRaw = localStorage.getItem("app-win-size");
            if (winSizeRaw) {
                const parsed = JSON.parse(winSizeRaw);
                if (parsed.width > 0 && parsed.height > 0) {
                    winWidth = parsed.width;
                    winHeight = parsed.height;
                }
            }
        } catch {
            /* ignore */
        }

        // Tier 2: Session snapshot window size
        if (!winWidth || !winHeight) {
            _sessionData = loadSession();
            if (_sessionData?.window?.width && _sessionData?.window?.height) {
                winWidth = _sessionData.window.width;
                winHeight = _sessionData.window.height;
            }
        }

        // Tier 3: Screen-adaptive default (65% × 75% of monitor, min 1024×680)
        if (!winWidth || !winHeight) {
            try {
                const { currentMonitor } =
                    await import("@tauri-apps/api/window");
                const monitor = await currentMonitor();
                if (monitor) {
                    const mw = monitor.size.width;
                    const mh = monitor.size.height;
                    winWidth = Math.max(1024, Math.round(mw * 0.65));
                    winHeight = Math.max(680, Math.round(mh * 0.75));
                    // Cap at 90% of screen
                    winWidth = Math.min(winWidth, Math.round(mw * 0.9));
                    winHeight = Math.min(winHeight, Math.round(mh * 0.9));
                }
            } catch {
                /* ignore */
            }
        }

        // Final fallback
        if (!winWidth || !winHeight) {
            winWidth = 1200;
            winHeight = 800;
        }

        // Also update unmaximized cache for future saves
        _unmaximizedWidth = winWidth;
        _unmaximizedHeight = winHeight;
        const { PhysicalSize, PhysicalPosition } =
            await import("@tauri-apps/api/dpi");
        await win.setSize(new PhysicalSize(winWidth, winHeight));
        // Center on current monitor
        try {
            const { currentMonitor } = await import("@tauri-apps/api/window");
            const monitor = await currentMonitor();
            if (monitor) {
                const { width: mw, height: mh } = monitor.size;
                const { x: mx, y: my } = monitor.position;
                const cx = Math.round(mx + (mw - winWidth) / 2);
                const cy = Math.round(my + (mh - winHeight) / 2);
                await win.setPosition(
                    new PhysicalPosition(Math.max(0, cx), Math.max(0, cy)),
                );
            }
        } catch {
            /* ignore if position API not available */
        }

        // ── Restore session state (view mode, layout, etc.) ──
        if (!_sessionData) _sessionData = loadSession();
        if (_sessionData) {
            // Restore view mode
            view.setViewMode(_sessionData.viewMode);
            // Restore properties panel
            showProperties.value = _sessionData.propertiesOpen;
            // Restore layout
            if (_sessionData.layout) {
                const restored = deserializeLayout(_sessionData.layout);
                tabStore.setRootLayout(restored);
                // Focus the pane from the index path
                const allPanes = tabStore.getAllPanes();
                const fpIdx = _sessionData.focusPaneIndexPath?.[0] ?? 0;
                const focusPane =
                    allPanes[Math.min(fpIdx, allPanes.length - 1)];
                if (focusPane) {
                    tabStore.focusPane(focusPane.id);
                    nav.focusedPaneId.value = focusPane.id;
                }
                // Restore navigation history
                navStore.restore(
                    _sessionData.navigationHistory || [],
                    _sessionData.navigationIndex ?? -1,
                );
                // Navigate all panes
                for (const pane of allPanes) {
                    const activeTab = pane.tabs.find(
                        (t: Tab) => t.id === pane.activeTabId,
                    );
                    if (!activeTab || !activeTab.path) continue;
                    tabStore.focusPane(pane.id);
                    nav.focusedPaneId.value = pane.id;
                    await store.navigateTo(activeTab.path, false);
                    actions.saveFileStateToTab(activeTab);
                    if (!_restoredPath) {
                        _restoredPath = activeTab.path;
                    }
                }
                // Restore the correct focus
                if (focusPane) {
                    tabStore.focusPane(focusPane.id);
                    nav.focusedPaneId.value = focusPane.id;
                    const ft = focusPane.tabs.find(
                        (t: Tab) => t.id === focusPane.activeTabId,
                    );
                    if (ft) actions.loadFileStateFromTab(ft);
                }
            }
        }
    } catch (e) {
        console.warn("Session restore failed, using defaults:", e);
    }

    // ── Listen for drag-drop (external file imports) ──
    let _ndu: (() => void) | null = null;
    try {
        const win = getCurrentWebviewWindow();
        _ndu = await win.onDragDropEvent(async (event: any) => {
            const e = event.payload;
            if (e.type !== "drop") return;
            const paths: string[] = (e.paths || [])
                .map((p: any) => (typeof p === "string" ? p : p.path || ""))
                .filter(Boolean);
            if (paths.length === 0) return;
            const dir = store.currentPath;
            if (!dir) return;
            try {
                await tauri.moveFiles(paths, dir, false);
                await store.refresh();
                toast.show(t("toast.imported", { count: paths.length }));
            } catch (err: any) {
                toast.show(t("toast.error") + ": " + err, true);
            }
        });
    } catch {
        /* ignore if API not available */
    }

    onUnmounted(() => {
        _ndu?.();
    });

    // ── Initialization (only if no session restored) ──
    const allPanes = tabStore.getAllPanes();
    if (allPanes.length > 0) {
        const firstPane = allPanes[0];
        if (!_sessionData || !_restoredPath) {
            tabStore.focusPane(firstPane.id);
            nav.focusedPaneId.value = firstPane.id;
            const activeTab = firstPane.tabs.find(
                (t: Tab) => t.id === firstPane.activeTabId,
            );
            if (activeTab) actions.loadFileStateFromTab(activeTab);
        }
    }
    // Load drives for sidebar
    await store.loadDrives();
    store.startDrivePolling();
    await store.checkUndoStatus();

    // ── Track window resize ──
    let _resizeTimer: ReturnType<typeof setTimeout> | null = null;
    const _onResize = async () => {
        if (_resizeTimer) clearTimeout(_resizeTimer);
        _resizeTimer = setTimeout(async () => {
            try {
                const win = getCurrentWebviewWindow();
                const maximized = await win.isMaximized();
                const size = await win.innerSize();
                // Always cache current size for exit save
                _cachedWinWidth = size.width;
                _cachedWinHeight = size.height;
                // When NOT maximized, persist the unmaximized size
                if (!maximized) {
                    _unmaximizedWidth = size.width;
                    _unmaximizedHeight = size.height;
                    localStorage.setItem(
                        "app-win-size",
                        JSON.stringify({
                            width: size.width,
                            height: size.height,
                        }),
                    );
                }
            } catch {
                /* ignore */
            }
        }, 500);
    };
    window.addEventListener("resize", _onResize);

    // ── Helper: save a session snapshot synchronously ──
    function _saveSessionNow() {
        try {
            const { layout, focusPaneIndexPath } = serializeLayout(
                tabStore.rootLayout,
                focusedPaneId.value || "",
            );
            const snapshot: SessionSnapshot = {
                version: 1,
                savedAt: Date.now(),
                window: {
                    width: _cachedWinWidth,
                    height: _cachedWinHeight,
                },
                viewMode: view.viewMode,
                propertiesOpen: showProperties.value,
                layout,
                navigationHistory: navStore.history.filter(
                    (h): h is string => typeof h === "string",
                ),
                navigationIndex: Math.max(
                    -1,
                    navStore.history.findIndex(
                        (h, i) =>
                            i <= navStore.historyIndex && typeof h === "string",
                    ),
                ),
                focusPaneIndexPath,
            };
            saveSession(snapshot);
            // Also persist unmaximized window size for next cold start
            localStorage.setItem(
                "app-win-size",
                JSON.stringify({
                    width: _unmaximizedWidth,
                    height: _unmaximizedHeight,
                }),
            );
        } catch (e) {
            console.error("Failed to save session:", e);
        }
    }

    // Save on beforeunload (standard web API)
    window.addEventListener("beforeunload", _saveSessionNow);

    // Also save on Tauri window close (more reliable in Tauri)
    // ── Tray event listeners ──
    let _trayNavUnlisten: (() => void) | null = null;
    let _traySettingsUnlisten: (() => void) | null = null;
    try {
        const { listen } = await import("@tauri-apps/api/event");
        _trayNavUnlisten = await listen<string>("tray-navigate", (event) => {
            const path = event.payload;
            if (path) {
                const fp = tabStore.getFocusedPane();
                if (fp) nav.navigatePane(fp.id, path);
            }
        });
        _traySettingsUnlisten = await listen("tray-open-settings", () => {
            showSettings.value = true;
        });
    } catch {
        /* ignore */
    }

    onUnmounted(() => {
        window.removeEventListener("resize", _onResize);
        window.removeEventListener("beforeunload", _saveSessionNow);
        _closeUnlisten?.();
        _trayNavUnlisten?.();
        _traySettingsUnlisten?.();
        store.stopDrivePolling();
    });

    // Signal splashscreen: frontend ready
    try {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("set_complete", { task: "frontend" });
    } catch {
        /* ignore */
    }
});
</script>

<style scoped>
.app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
}
.main-content {
    flex: 1;
    display: flex;
    overflow: hidden;
}
.panes-area {
    flex: 1;
    overflow: hidden;
    display: flex;
}
.toast-container {
    position: fixed;
    bottom: 36px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    flex-direction: column-reverse;
    gap: 8px;
    z-index: 2000;
}
.toast {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 8px 20px;
    font-size: 13px;
    color: var(--text-primary);
    box-shadow: 0 4px 16px var(--shadow);
    display: flex;
    align-items: center;
    gap: 8px;
    white-space: nowrap;
}
.toast-error {
    border-color: var(--danger);
    color: var(--danger);
}
.toast-icon {
    font-size: 14px;
}
</style>
