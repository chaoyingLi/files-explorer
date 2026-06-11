<template>
    <div
        v-if="node.type === 'pane'"
        class="pane-leaf"
        :class="{ focused: isFocused }"
        @mousedown="$emit('focus', node.id)"
    >
        <div class="pane-tabs">
            <div
                v-for="tab in node.tabs"
                :key="tab.id"
                class="pane-tab"
                :class="{ active: node.activeTabId === tab.id }"
                @click="$emit('tabClick', node.id, tab.id)"
                @auxclick.prevent="$emit('tabClose', node.id, tab.id)"
                @dragover.prevent="tabStore.onTabDragEnter(tab.id)"
                @dragleave="tabStore.onTabDragLeave(tab.id)"
                @drop.prevent="$emit('tabDrop', node.id, tab.id, $event)"
            >
                <span class="pane-tab-title">{{
                    tab.title || t("sidebar.thisPc")
                }}</span>
                <button
                    v-if="node.tabs.length > 1"
                    class="pane-tab-close"
                    @click.stop="$emit('tabClose', node.id, tab.id)"
                >
                    <svg viewBox="0 0 10 10">
                        <path
                            d="M2 2l6 6M8 2l-6 6"
                            stroke="currentColor"
                            stroke-width="1.2"
                            stroke-linecap="round"
                        />
                    </svg>
                </button>
            </div>
            <button
                class="pane-tab-new"
                @click="$emit('tabNew', node.id)"
                title="New Tab"
            >
                +
            </button>
            <button
                class="pane-close-btn"
                @click.stop="$emit('paneClose', node.id)"
                title="Close Pane"
            >
                <svg viewBox="0 0 10 10">
                    <path
                        d="M2 2l6 6M8 2l-6 6"
                        stroke="currentColor"
                        stroke-width="1.2"
                        stroke-linecap="round"
                    />
                </svg>
            </button>
        </div>
        <!-- Search tab: show search header -->
        <div v-if="activeTab?.isSearch" class="search-header">
            <svg class="search-header-icon" viewBox="0 0 18 18" fill="none">
                <circle
                    cx="7.5"
                    cy="7.5"
                    r="5"
                    stroke="currentColor"
                    stroke-width="1.5"
                />
                <path
                    d="M11 11l4 4"
                    stroke="currentColor"
                    stroke-width="1.5"
                    stroke-linecap="round"
                />
            </svg>
            <span class="search-header-query">{{
                activeTab?.searchQuery
            }}</span>
            <span v-if="activeTab?.searchDone" class="search-header-count">
                {{ activeTab?.searchTotal }} {{ t("fileList.items") }}
            </span>
            <span v-else class="search-header-searching">
                {{ t("fileList.searching") }}
                <span class="search-spinner"></span>
            </span>
        </div>
        <!-- Normal tab: show breadcrumb -->
        <Breadcrumb
            v-else
            :path="activeTab?.path ?? ''"
            @navigate="(p: string) => $emit('navigate', node.id, p)"
        />
        <FileList
            :pane-id="node.id"
            @file-drop="
                (dir: string, paths: string[], ctrl: boolean) =>
                    $emit('fileDrop', node.id, dir, paths, ctrl)
            "
        />
    </div>
    <div v-else class="pane-split" :class="'split-' + node.direction">
        <PaneNode
            v-for="child in node.children"
            :key="child.id"
            :node="child"
            :focused-pane-id="focusedPaneId"
            @focus="(id: string) => $emit('focus', id)"
            @tabClick="
                (pid: string, tid: string) => $emit('tabClick', pid, tid)
            "
            @tabClose="
                (pid: string, tid: string) => $emit('tabClose', pid, tid)
            "
            @tabNew="(pid: string) => $emit('tabNew', pid)"
            @tabDrop="
                (pid: string, tid: string, e: DragEvent) =>
                    $emit('tabDrop', pid, tid, e)
            "
            @paneClose="(pid: string) => $emit('paneClose', pid)"
            @navigate="(pid: string, p: string) => $emit('navigate', pid, p)"
            @fileDrop="
                (pid: string, dir: string, paths: string[], ctrl: boolean) =>
                    $emit('fileDrop', pid, dir, paths, ctrl)
            "
        />
    </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import {
    useTabStore,
    type LayoutNode,
    type LayoutPane,
} from "@/stores/tabStore";
import Breadcrumb from "@/components/Breadcrumb.vue";
import FileList from "@/components/FileList.vue";

const { t } = useI18n();

const props = defineProps<{ node: LayoutNode; focusedPaneId: string }>();
defineEmits<{
    focus: [paneId: string];
    tabClick: [paneId: string, tabId: string];
    tabClose: [paneId: string, tabId: string];
    tabNew: [paneId: string];
    paneClose: [paneId: string];
    tabDrop: [paneId: string, tabId: string, e: DragEvent];
    navigate: [paneId: string, path: string];
    fileDrop: [paneId: string, dir: string, paths: string[], ctrl: boolean];
}>();

const tabStore = useTabStore();
const activeTab = computed(() => {
    if (props.node.type !== "pane") return null;
    const pane = props.node as LayoutPane;
    return pane.tabs.find((t) => t.id === pane.activeTabId);
});
const isFocused = computed(
    () => props.node.type === "pane" && props.node.id === props.focusedPaneId,
);
</script>

<script lang="ts">
export default { name: "PaneNode" };
</script>

<style scoped>
.pane-leaf {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
    min-height: 0;
    border: 1px solid transparent;
}
.pane-leaf.focused {
    border-color: var(--accent);
}
.pane-split {
    flex: 1;
    display: flex;
    overflow: hidden;
    min-width: 0;
    min-height: 0;
}
.pane-split.split-horizontal {
    flex-direction: row;
}
.pane-split.split-vertical {
    flex-direction: column;
}
.pane-tabs {
    display: flex;
    align-items: center;
    height: 32px;
    padding: 0 4px;
    gap: 1px;
    background: var(--bg-tertiary);
    overflow-x: auto;
    flex-shrink: 0;
}
.pane-tab {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 3px 10px;
    height: 26px;
    border-radius: 6px 6px 0 0;
    cursor: pointer;
    font-size: 12px;
    background: transparent;
    color: var(--text-muted);
    white-space: nowrap;
    border: 1px solid transparent;
    border-bottom: none;
    max-width: 160px;
}
.pane-tab:hover {
    background: var(--bg-hover);
}
.pane-tab.active {
    background: var(--bg-primary);
    color: var(--text-primary);
    border-color: var(--border);
}
.pane-tab-title {
    overflow: hidden;
    text-overflow: ellipsis;
}
.pane-tab-close {
    width: 14px;
    height: 14px;
    padding: 0;
    opacity: 0;
    border-radius: 3px;
    flex-shrink: 0;
}
.pane-tab:hover .pane-tab-close {
    opacity: 0.5;
}
.pane-tab-close:hover {
    opacity: 1 !important;
    background: var(--bg-hover);
}
.pane-tab-new {
    width: 24px;
    height: 24px;
    font-size: 14px;
    line-height: 1;
    padding: 0;
    border-radius: 5px;
    flex-shrink: 0;
    color: var(--text-muted);
}
.pane-tab-new:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
}
.pane-close-btn {
    width: 20px;
    height: 20px;
    padding: 0;
    border-radius: 4px;
    flex-shrink: 0;
    color: var(--text-muted);
    margin-left: auto;
}
.pane-close-btn:hover {
    background: var(--bg-hover);
    color: var(--danger);
}

/* ── Search tab header ── */
.search-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 12px;
    background: var(--bg-primary);
    border-bottom: 1px solid var(--border);
    min-height: 32px;
    font-size: 13px;
}

.search-header-icon {
    width: 16px;
    height: 16px;
    color: var(--accent);
    flex-shrink: 0;
}

.search-header-query {
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.search-header-count {
    color: var(--text-muted);
    font-size: 12px;
    margin-left: auto;
    flex-shrink: 0;
}

.search-header-searching {
    color: var(--text-muted);
    font-size: 12px;
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
}

.search-spinner {
    width: 12px;
    height: 12px;
    border: 2px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: search-spin 0.6s linear infinite;
}

@keyframes search-spin {
    to {
        transform: rotate(360deg);
    }
}
</style>
