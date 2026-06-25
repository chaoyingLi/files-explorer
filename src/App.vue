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
            @search-submit="(q, c) => search.submitSearch(q, c)"
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
            <PropertiesPanel
                v-if="showProperties"
                :visible="showProperties"
                @close="showProperties = false"
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
import PropertiesPanel from "@/components/PropertiesPanel.vue";

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
const showSettings = ref(false);
const showProperties = ref(false);

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
            | "left"
            | "right"
            | "up"
            | "down";
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
