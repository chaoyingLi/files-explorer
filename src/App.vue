<template>
    <div
        class="app-container"
        @click="onGlobalClick"
        @contextmenu.prevent="onGlobalContextMenu"
    >
        <TitleBar />
        <Toolbar
            @open-settings="showSettings = true"
            @navigate-back="handleToolbarBack"
            @navigate-forward="handleToolbarForward"
            @navigate-up="handleToolbarUp"
            @refresh="handleToolbarRefresh"
            @navigate-address="handleToolbarAddress"
        />
        <RibbonToolbar @action="handleContextAction" />
        <div class="main-content">
            <Sidebar
                @navigate="handleSidebarNavigate"
                @navigate-home="handleSidebarHome"
                @context-menu="handleSidebarContext"
            />
            <div class="panes-area">
                <PaneNode
                    :node="tabStore.rootLayout"
                    :focused-pane-id="focusedPaneId"
                    @focus="onPaneFocusEvent"
                    @tab-click="
                        (pid: string, tid: string) => onTabClickEvent(pid, tid)
                    "
                    @tab-close="
                        (pid: string, tid: string) => onTabCloseEvent(pid, tid)
                    "
                    @tab-new="(pid: string) => onNewTabEvent(pid)"
                    @pane-close="
                        (pid: string) => {
                            tabStore.closePane(pid);
                            const fp = tabStore.getFocusedPane();
                            if (fp) {
                                focusedPaneId = fp.id;
                                const t = fp.tabs.find(
                                    (x: Tab) => x.id === fp.activeTabId,
                                );
                                if (t) {
                                    loadFileStateFromTab(t);
                                    if (!t.path) store.loadDrives();
                                }
                            }
                        }
                    "
                    @tab-drop="
                        (pid: string, tid: string, e: DragEvent) =>
                            onTabDropEvent(pid, tid, e)
                    "
                    @navigate="
                        (pid: string, path: string) =>
                            navigatePaneEvent(pid, path)
                    "
                />
            </div>
        </div>
        <DeleteConfirmDialog
            v-if="store.showDeleteConfirm"
            :count="store.deleteTargetCount"
            :permanently="store.deletePermanently"
            @confirm="handleDeleteConfirm"
            @cancel="store.cancelDelete()"
        />
        <StatusBar />
        <ContextMenu
            v-if="showContextMenu"
            :x="contextMenuPos.x"
            :y="contextMenuPos.y"
            :items="contextMenuItems"
            @close="showContextMenu = false"
            @action="handleContextAction"
        />
        <NewItemDialog
            v-if="showNewDialog"
            :type="newDialogType"
            @close="showNewDialog = false"
            @confirm="handleNewItem"
        />
        <RenameDialog
            v-if="showRenameDialog"
            :oldName="renameTarget"
            @close="showRenameDialog = false"
            @confirm="handleRename"
        />
        <SettingsDialog v-if="showSettings" @close="showSettings = false" />
        <div
            v-if="toastMessage"
            class="toast"
            :class="{ 'toast-error': toastIsError }"
        >
            <span v-if="toastIsError" class="toast-icon">⚠</span>
            {{ toastMessage }}
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import { useSettingsStore } from "@/stores/settingsStore";
import { useTabStore, type Tab, type LayoutPane } from "@/stores/tabStore";
import * as tauri from "@/utils/tauri";
import type { ContextMenuOption } from "@/types";
import TitleBar from "@/components/TitleBar.vue";
import DeleteConfirmDialog from "@/components/Dialogs/DeleteConfirmDialog.vue";
import Toolbar from "@/components/Toolbar.vue";
import Sidebar from "@/components/Sidebar.vue";
import FileList from "@/components/FileList.vue";
import Breadcrumb from "@/components/Breadcrumb.vue";
import ContextMenu from "@/components/ContextMenu.vue";
import StatusBar from "@/components/StatusBar.vue";
import NewItemDialog from "@/components/Dialogs/NewItemDialog.vue";
import RenameDialog from "@/components/Dialogs/RenameDialog.vue";
import SettingsDialog from "@/components/Dialogs/SettingsDialog.vue";
import RibbonToolbar from "@/components/RibbonToolbar.vue";
import PaneNode from "@/components/PaneNode.vue";

const { t } = useI18n();
const store = useFileStore();
useSettingsStore();
const tabStore = useTabStore();

const showContextMenu = ref(false);
const contextMenuPos = ref({ x: 0, y: 0 });
const sidebarContextPath = ref("");
const showNewDialog = ref(false);
const newDialogType = ref("folder");
const showRenameDialog = ref(false);
const showSettings = ref(false);
const focusedPaneId = ref("");
const renameTarget = ref("");
const toastMessage = ref("");
const toastIsError = ref(false);
let toastTimer: ReturnType<typeof setTimeout> | null = null;

const contextMenuItems = computed<ContextMenuOption[]>(() => {
    const hasSelection = store.selectedFiles.size > 0;
    const singleSelection = store.selectedFiles.size === 1;

    const items: ContextMenuOption[] = [];

    if (!store.currentPath) {
        items.push({ label: t("contextMenu.open"), action: "open" });
        return items;
    }

    items.push(
        {
            label: t("contextMenu.newFolder"),
            action: "newFolder",
            shortcut: t("shortcuts.ctrlShiftN"),
        },
        {
            label: t("contextMenu.newFile"),
            action: "newFile",
            shortcut: t("shortcuts.ctrlN"),
        },
        { label: "", action: "", separator: true },
    );

    if (hasSelection) {
        items.push(
            { label: t("contextMenu.open"), action: "open" },
            { label: "", action: "", separator: true },
            {
                label: t("contextMenu.cut"),
                action: "cut",
                shortcut: t("shortcuts.ctrlX"),
            },
            {
                label: t("contextMenu.copy"),
                action: "copy",
                shortcut: t("shortcuts.ctrlC"),
            },
            { label: "", action: "", separator: true },
            {
                label: t("contextMenu.delete"),
                action: "delete",
                shortcut: t("shortcuts.del"),
                children: [
                    {
                        label: t("dialogs.delete"),
                        action: "delete",
                    },
                    {
                        label: t("dialogs.deletePermanent"),
                        action: "deletePermanent",
                    },
                ],
            },
            { label: "", action: "", separator: true },
        );

        if (singleSelection) {
            items.push({
                label: t("contextMenu.rename"),
                action: "rename",
                shortcut: t("shortcuts.f2"),
            });
        }
    }

    // Open in Terminal + Split
    if (store.currentPath) {
        items.push(
            { label: "", action: "", separator: true },
            {
                label: t("contextMenu.openInTerminal"),
                action: "openInTerminal",
            },
            {
                label: t("split.label"),
                action: "split",
                children: [
                    { label: t("split.left"), action: "splitLeft" },
                    { label: t("split.right"), action: "splitRight" },
                    { label: t("split.up"), action: "splitUp" },
                    { label: t("split.down"), action: "splitDown" },
                ],
            },
        );
    }

    items.push(
        {
            label: t("contextMenu.paste"),
            action: "paste",
            shortcut: t("shortcuts.ctrlV"),
        },
        { label: "", action: "", separator: true },
        {
            label: t("contextMenu.selectAll"),
            action: "selectAll",
            shortcut: t("shortcuts.ctrlA"),
        },
        { label: "", action: "", separator: true },
        {
            label: t("contextMenu.refresh"),
            action: "refresh",
            shortcut: t("shortcuts.f5"),
        },
    );

    return items;
});

function showToast(msg: string, isError = false) {
    toastMessage.value = msg;
    toastIsError.value = isError;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(
        () => {
            toastMessage.value = "";
        },
        isError ? 4000 : 2000,
    );
}

function onGlobalClick() {
    showContextMenu.value = false;
}

function onGlobalContextMenu(e: MouseEvent) {
    contextMenuPos.value = { x: e.clientX, y: e.clientY };
    showContextMenu.value = true;
}

async function handleContextAction(action: string) {
    showContextMenu.value = false;

    switch (action) {
        case "newFolder":
            newDialogType.value = "folder";
            showNewDialog.value = true;
            break;
        case "newFile":
            newDialogType.value = "file";
            showNewDialog.value = true;
            break;
        case "open":
            if (store.selectedFiles.size > 0) {
                const first = [...store.selectedFiles][0];
                const file = store.files.find((f) => f.path === first);
                if (file) await store.openSelectedFile(file);
            }
            break;
        case "cut":
            await store.cutSelected();
            showToast(t("toast.cut"));
            break;
        case "copy":
            await store.copySelected();
            showToast(t("toast.copied"));
            break;
        case "paste":
            try {
                await store.paste();
                showToast(t("toast.pasted"));
            } catch (e: any) {
                showToast(t("toast.error") + ": " + e);
            }
            break;
        case "delete":
            store.requestDelete(false);
            break;
        case "deletePermanent":
            store.requestDelete(true);
            break;
        case "rename":
            if (store.selectedFiles.size === 1) {
                const path = [...store.selectedFiles][0];
                const file = store.files.find((f) => f.path === path);
                if (file) {
                    renameTarget.value = file.name;
                    showRenameDialog.value = true;
                }
            }
            break;
        case "selectAll":
            store.selectAll();
            break;
        case "refresh":
            await store.refresh();
            break;
        case "openInTerminal":
            try {
                const target =
                    store.selectedFiles.size === 1
                        ? [...store.selectedFiles][0]
                        : store.currentPath;
                await tauri.openInTerminal(target);
            } catch (e: any) {
                showToast(t("toast.error") + ": " + e);
            }
            break;
        case "settings":
            showSettings.value = true;
            break;
        case "splitLeft":
        case "splitRight":
        case "splitUp":
        case "splitDown":
            // Determine the target path: sidebar context > right-click selection > current path
            let splitPath = "";
            let splitTitle = "";
            if (sidebarContextPath.value) {
                splitPath = sidebarContextPath.value;
                const parts = splitPath
                    .replace(/\\/g, "/")
                    .split("/")
                    .filter(Boolean);
                splitTitle = parts[parts.length - 1] || splitPath;
                sidebarContextPath.value = "";
            } else if (store.selectedFiles.size === 1) {
                const sel = [...store.selectedFiles][0];
                const sf = store.files.find((f: any) => f.path === sel);
                if (sf?.is_dir) {
                    splitPath = sel;
                    splitTitle = sf.name;
                }
            }
            if (!splitPath) {
                splitPath = store.currentPath;
                splitTitle = store.currentDirectoryName;
            }
            if (splitPath) {
                const dir = action.replace("split", "").toLowerCase() as
                    | "left"
                    | "right"
                    | "up"
                    | "down";
                const fp = tabStore.getFocusedPane();
                if (fp) {
                    const ot = tabStore.getFocusedTab();
                    if (ot) saveFileStateToTab(ot);
                    tabStore.splitPane(fp.id, splitPath, splitTitle, dir);
                    await store.navigateTo(splitPath, false);
                    // Sync the loaded files to the new pane's tab
                    const nt = tabStore.getFocusedTab();
                    if (nt) saveFileStateToTab(nt);
                }
            }
            break;
    }
}

function saveFileStateToTab(tab: Tab) {
    tab.files = store.files;
    tab.path = store.currentPath;
    tab.title = store.currentDirectoryName || t("sidebar.thisPc");
    tab.selectedFiles = [...store.selectedFiles];
    tab.treeExpanded = store.getTreeExpandedArray();
}
function loadFileStateFromTab(tab: Tab) {
    store.files = tab.files || [];
    store.selectedFiles = new Set(tab.selectedFiles || []);
    store.currentPath = tab.path || "";
    store.setTreeExpanded(tab.treeExpanded || []);
}

function onPaneFocusEvent(paneId: string) {
    const ot = tabStore.getFocusedTab();
    if (ot) saveFileStateToTab(ot);
    tabStore.focusPane(paneId);
    focusedPaneId.value = paneId;
    const p = tabStore.findPaneById(paneId);
    if (p) {
        const t = p.tabs.find((x: Tab) => x.id === p.activeTabId);
        if (t) {
            loadFileStateFromTab(t);
            if (!t.path) store.loadDrives();
        }
    }
}
function onTabClickEvent(paneId: string, tabId: string) {
    const ot = tabStore.getFocusedTab();
    if (ot) saveFileStateToTab(ot);
    tabStore.switchTab(paneId, tabId);
    tabStore.focusPane(paneId);
    focusedPaneId.value = paneId;
    const p = tabStore.findPaneById(paneId);
    if (p) {
        const t = p.tabs.find((x: Tab) => x.id === tabId);
        if (t) {
            loadFileStateFromTab(t);
            if (!t.path) store.loadDrives();
        }
    }
}
function onTabCloseEvent(paneId: string, tabId: string) {
    tabStore.closeTab(paneId, tabId);
    const p = tabStore.findPaneById(paneId);
    if (p) {
        const t = p.tabs.find((x: Tab) => x.id === p.activeTabId);
        if (t) {
            loadFileStateFromTab(t);
            if (!t.path) store.loadDrives();
        }
    }
}
async function onNewTabEvent(paneId: string) {
    // Save current tab state before switching
    const ot = tabStore.getFocusedTab();
    if (ot) saveFileStateToTab(ot);

    // Focus the target pane and create the new tab
    tabStore.focusPane(paneId);
    focusedPaneId.value = paneId;
    tabStore.addTab(
        paneId,
        store.currentPath || "",
        store.currentDirectoryName || t("sidebar.thisPc"),
    );

    // Load the new tab's saved state into the store
    const fp = tabStore.getFocusedPane();
    if (fp) {
        const nt = fp.tabs.find((t: Tab) => t.id === fp.activeTabId);
        if (nt) {
            loadFileStateFromTab(nt);
            if (nt.path) {
                await store.navigateTo(nt.path, false);
                const tab = tabStore.getFocusedTab();
                if (tab) saveFileStateToTab(tab);
            } else {
                await store.loadDrives();
            }
        }
    }
}
async function navigatePaneEvent(paneId: string, path: string) {
    const ot = tabStore.getFocusedTab();
    if (ot) saveFileStateToTab(ot);
    tabStore.focusPane(paneId);
    focusedPaneId.value = paneId;
    if (!path) {
        store.currentPath = "";
        store.files = [];
        store.selectedFiles = new Set();
        await store.loadDrives();
    } else await store.navigateTo(path, false);
    // After navigation completes, sync back to the active tab
    const nt = tabStore.getFocusedTab();
    if (nt) saveFileStateToTab(nt);
}

// Sidebar navigation: navigate the focused pane
async function handleSidebarNavigate(path: string) {
    const fp = tabStore.getFocusedPane();
    if (!fp) return;
    await navigatePaneEvent(fp.id, path);
}

async function handleSidebarHome() {
    const fp = tabStore.getFocusedPane();
    if (!fp) return;
    await navigatePaneEvent(fp.id, "");
}

// Sidebar right-click: navigate to item and show context menu
function handleSidebarContext(path: string, event: MouseEvent) {
    sidebarContextPath.value = path || "";
    showContextMenu.value = true;
    contextMenuPos.value = { x: event.clientX, y: event.clientY };
}

// Toolbar navigation through pane system
async function handleToolbarBack() {
    await store.navigateBack();
}
async function handleToolbarForward() {
    await store.navigateForward();
}
async function handleToolbarUp() {
    await store.navigateUp();
}
async function handleToolbarRefresh() {
    await store.refresh();
}
async function handleToolbarAddress(path: string) {
    const fp = tabStore.getFocusedPane();
    if (!fp) return;
    await navigatePaneEvent(fp.id, path);
}

// Auto-save: sync store changes to focused tab
watch(
    () => store.currentPath,
    () => {
        const tab = tabStore.getFocusedTab();
        if (tab) saveFileStateToTab(tab);
    },
);
async function onTabDropEvent(paneId: string, tabId: string, e: DragEvent) {
    e.preventDefault();
    tabStore.onTabDragLeave(tabId);
    const raw = e.dataTransfer?.getData("text/plain");
    if (!raw) return;
    const paths = raw.split("\n").filter(Boolean);
    const tp = tabStore.findPaneById(paneId);
    if (!tp) return;
    const tt =
        tp.tabs.find((t: Tab) => t.id === tabId) ||
        tp.tabs.find((t: Tab) => t.id === tp.activeTabId);
    if (!tt?.path) return;
    for (const sp of paths) {
        const nm = sp.replace(/\\/g, "/").split("/").pop() || "";
        const dp =
            tt.path +
            (tt.path.endsWith("\\") || tt.path.endsWith("/") ? "" : "\\") +
            nm;
        if (sp !== dp) {
            try {
                await tauri.renameItem(sp, dp);
            } catch (_) {}
        }
    }
    store.navigateTo(tt.path, false);
}

async function handleNewItem(name: string, type: string) {
    if (type === "folder") {
        await store.createNewFolder(name);
    } else {
        await store.createNewFile(name);
    }
    showNewDialog.value = false;
}

async function handleDeleteConfirm(permanently: boolean) {
    store.deletePermanently = permanently;
    try {
        const result = await store.confirmDelete();
        if (result.failed > 0) {
            showToast(`${result.message}`, true);
        } else {
            showToast(t("toast.deleted"));
        }
    } catch (e: any) {
        showToast(t("toast.error") + ": " + e, true);
    }
}

async function handleRename(newName: string) {
    if (store.selectedFiles.size === 1) {
        const oldPath = [...store.selectedFiles][0];
        await store.renameFile(oldPath, newName);
    }
    showRenameDialog.value = false;
}

function onKeydown(e: KeyboardEvent) {
    if (e.target instanceof HTMLInputElement) return;
    const ctrl = e.ctrlKey || e.metaKey;

    // ── RICH SHORTCUTS ──
    if (e.key === "Enter" && store.selectedFiles.size > 0) {
        e.preventDefault();
        const first = [...store.selectedFiles][0];
        const file = store.files.find((f) => f.path === first);
        if (file) store.openSelectedFile(file);
    } else if (e.key === "Escape" && store.isCutPending) {
        e.preventDefault();
        store.cancelCut();
        showToast("Cut cancelled");
    } else if (ctrl && e.key === "z") {
        e.preventDefault();
        store
            .performUndo()
            .then((msg) => showToast(msg))
            .catch((e) => showToast("Undo: " + e, true));
    } else if (ctrl && e.key === "w") {
        e.preventDefault();
        const fp = tabStore.getFocusedPane();
        if (fp) onTabCloseEvent(fp.id, fp.activeTabId);
    } else if (ctrl && e.key === "Tab" && !e.shiftKey) {
        e.preventDefault();
        const fp = tabStore.getFocusedPane();
        if (fp && fp.tabs.length > 1) {
            const idx = fp.tabs.findIndex((t) => t.id === fp.activeTabId);
            const next = (idx + 1) % fp.tabs.length;
            onTabClickEvent(fp.id, fp.tabs[next].id);
        }
    } else if (ctrl && e.shiftKey && e.key === "Tab") {
        e.preventDefault();
        const fp = tabStore.getFocusedPane();
        if (fp && fp.tabs.length > 1) {
            const idx = fp.tabs.findIndex((t) => t.id === fp.activeTabId);
            const prev = (idx - 1 + fp.tabs.length) % fp.tabs.length;
            onTabClickEvent(fp.id, fp.tabs[prev].id);
        }
    } else if (ctrl && e.key === "n") {
        e.preventDefault();
        newDialogType.value = "file";
        showNewDialog.value = true;
    } else if (ctrl && e.shiftKey && e.key === "N") {
        e.preventDefault();
        newDialogType.value = "folder";
        showNewDialog.value = true;
    } else if (ctrl && e.key === "c") {
        e.preventDefault();
        store.copySelected();
        showToast(t("toast.copied"));
    } else if (ctrl && e.key === "x") {
        e.preventDefault();
        store.cutSelected();
        showToast(t("toast.cut"));
    } else if (ctrl && e.key === "v") {
        e.preventDefault();
        store
            .paste()
            .then(() => showToast(t("toast.pasted")))
            .catch((e) => showToast(t("toast.error") + ": " + e));
    } else if (ctrl && e.key === "a") {
        e.preventDefault();
        store.selectAll();
    } else if (e.key === "Delete") {
        e.preventDefault();
        store.requestDelete(false);
    } else if (e.key === "Delete" && e.shiftKey) {
        e.preventDefault();
        store.requestDelete(true);
    } else if (e.key === "F2" && store.selectedFiles.size === 1) {
        e.preventDefault();
        const path = [...store.selectedFiles][0];
        const file = store.files.find((f) => f.path === path);
        if (file) {
            renameTarget.value = file.name;
            showRenameDialog.value = true;
        }
    } else if (e.key === "F5") {
        e.preventDefault();
        store.refresh();
    } else if (ctrl && e.key === ",") {
        e.preventDefault();
        showSettings.value = true;
    } else if (e.key === "Backspace" && !ctrl) {
        e.preventDefault();
        store.navigateUp();
    }
}

onMounted(async () => {
    document.addEventListener("keydown", onKeydown);

    // Initialize first pane focus
    const allPanes = tabStore.getAllPanes();
    if (allPanes.length > 0) {
        const firstPane = allPanes[0];
        tabStore.focusPane(firstPane.id);
        focusedPaneId.value = firstPane.id;
        const activeTab = firstPane.tabs.find(
            (t: Tab) => t.id === firstPane.activeTabId,
        );
        if (activeTab) {
            loadFileStateFromTab(activeTab);
        }
    }

    await store.loadDrives();
    await store.checkUndoStatus();
});

onUnmounted(() => {
    document.removeEventListener("keydown", onKeydown);
});
</script>

<style scoped>
.app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
}

.main-content {
    display: flex;
    flex: 1;
    overflow: hidden;
}

.file-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
}

.panes-area {
    flex: 1;
    display: flex;
    overflow: hidden;
}
</style>
