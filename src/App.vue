<template>
    <div
        class="app-container"
        @click="ctx.closeContextMenu()"
        @contextmenu.prevent="onGlobalContextMenu"
    >
        <TitleBar />
        <Toolbar
            @open-settings="showSettings = true"
            @navigate-back="nav.toolbarBack"
            @navigate-forward="nav.toolbarForward"
            @navigate-up="nav.toolbarUp"
            @refresh="nav.toolbarRefresh"
            @navigate-address="nav.toolbarAddress"
            @search-submit="search.submitSearch"
        />
        <RibbonToolbar @action="(a: string) => actions.executeAction(a)" />
        <div class="main-content">
            <Sidebar
                @navigate="nav.sidebarNavigate"
                @navigate-home="nav.sidebarHome"
                @context-menu="handleSidebarContext"
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
        </div>
        <DeleteConfirmDialog
            v-if="store.showDeleteConfirm"
            :count="store.deleteTargetCount"
            :permanently="store.deletePermanently"
            @confirm="actions.handleDeleteConfirm"
            @cancel="store.cancelDelete()"
        />
        <StatusBar />
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
        <div
            v-if="toast.message.value"
            class="toast"
            :class="{ 'toast-error': toast.isError.value }"
        >
            <span v-if="toast.isError.value" class="toast-icon">⚠</span>
            {{ toast.message.value }}
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import { useSettingsStore } from "@/stores/settingsStore";
import { useTabStore, type Tab } from "@/stores/tabStore";
import * as tauri from "@/utils/tauri";

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
useSettingsStore();
const tabStore = useTabStore();
const showSettings = ref(false);

const toast = useToast();
const ctx = useContextMenu();
const actions = useFileActions(toast);
const nav = usePanelNavigation(
    actions.saveFileStateToTab,
    actions.loadFileStateFromTab,
    t,
);
const search = useSearchService(actions.saveFileStateToTab, toast.show);
const dnd = useDragDrop(toast.show, t);

// Unwrap refs for template prop binding (vue-tsc strictness)
const focusedPaneId = computed(() => nav.focusedPaneId.value);
const contextMenuItems = computed(() => ctx.contextMenuItems.value);

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
});

function onGlobalContextMenu(e: MouseEvent) {
    ctx.openContextMenu(e.clientX, e.clientY);
}

function handleSidebarContext(path: string, event: MouseEvent) {
    ctx.sidebarContextPath.value = path || "";
    ctx.openContextMenu(event.clientX, event.clientY);
}

async function handleContextAction(action: string) {
    ctx.closeContextMenu();
    if (action === "showInExplorer") {
        const path =
            store.selectedFiles.size === 1
                ? [...store.selectedFiles][0]
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
            | "left"
            | "right"
            | "up"
            | "down";
        let splitPath = ctx.sidebarContextPath.value;
        let splitTitle = "";
        if (!splitPath && store.selectedFiles.size === 1) {
            const sel = [...store.selectedFiles][0];
            const sf = store.files.find((f) => f.path === sel);
            if (sf?.is_dir) {
                splitPath = sel;
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
    // Native OS file drop (Explorer / other apps drag files into window)
    let _ndu: (() => void) | null = null;
    try {
        const win = getCurrentWebviewWindow();
        _ndu = await win.onDragDropEvent(async (event: any) => {
            const e = event.payload;
            if (e.type !== "drop") return;
            // Extract absolute file paths
            const paths: string[] = (e.paths || [])
                .map((p: any) => (typeof p === "string" ? p : p.path || ""))
                .filter(Boolean);
            if (paths.length === 0) return;
            // Move files to focused pane's current directory
            const dir = store.currentPath;
            if (!dir) return;
            try {
                await tauri.moveFiles(paths, dir, false);
                await store.refresh();
                toast.show(`Imported ${paths.length} file(s)`);
            } catch (err: any) {
                toast.show(`Import failed: ${err}`, true);
            }
        });
    } catch {
        /* ignore if API not available */
    }

    // Init panels
    const allPanes = tabStore.getAllPanes();
    if (allPanes.length > 0) {
        const firstPane = allPanes[0];
        tabStore.focusPane(firstPane.id);
        nav.focusedPaneId.value = firstPane.id;
        const activeTab = firstPane.tabs.find(
            (t: Tab) => t.id === firstPane.activeTabId,
        );
        if (activeTab) actions.loadFileStateFromTab(activeTab);
    }
    await store.loadDrives();
    await store.checkUndoStatus();
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
.toast {
    position: fixed;
    bottom: 36px;
    left: 50%;
    transform: translateX(-50%);
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 8px 20px;
    font-size: 13px;
    color: var(--text-primary);
    box-shadow: 0 4px 16px var(--shadow);
    z-index: 2000;
    display: flex;
    align-items: center;
    gap: 8px;
}
.toast-error {
    border-color: var(--danger);
    color: var(--danger);
}
.toast-icon {
    font-size: 14px;
}
</style>
