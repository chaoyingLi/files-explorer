<template>
    <div
        ref="listEl"
        class="file-list"
        :class="{ 'drop-active': isDragOver && !!currentPath }"
        :style="colVars"
        @click.self="store.clearSelection()"
        @contextmenu.prevent="onContextMenu"
    >
        <div v-if="isDragOver && currentPath" class="drop-indicator">
            <svg viewBox="0 0 16 16" class="drop-icon">
                <path
                    d="M8 2v10M3 7l5 5 5-5"
                    stroke="currentColor"
                    stroke-width="1.5"
                    fill="none"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                />
            </svg>
            <span>{{ t("fileList.dropToMove") }}</span>
        </div>

        <!-- Column header (details/list only) -->
        <div
            v-if="
                currentPath &&
                (store.viewMode === 'details' || store.viewMode === 'list')
            "
            class="file-list-header"
        >
            <div
                class="col-name"
                :style="{ width: colWidths.name + 'px' }"
                @click="sortBy('name')"
            >
                {{ t("fileList.name")
                }}{{ sortField === "name" ? (sortAsc ? " ▲" : " ▼") : "" }}
                <div
                    class="col-handle"
                    @mousedown.stop="onResizeStart('name', $event)"
                ></div>
            </div>
            <div v-if="isSearchTab" class="col-path-header">
                {{ t("fileList.path") }}
            </div>
            <div
                class="col-date"
                v-if="store.viewMode === 'details'"
                :style="{ width: colWidths.date + 'px' }"
                @click="sortBy('modified')"
            >
                {{ t("fileList.dateModified")
                }}{{ sortField === "modified" ? (sortAsc ? " ▲" : " ▼") : "" }}
                <div
                    class="col-handle"
                    @mousedown.stop="onResizeStart('date', $event)"
                ></div>
            </div>
            <div
                class="col-created"
                v-if="store.viewMode === 'details'"
                :style="{ width: colWidths.created + 'px' }"
                @click="sortBy('created')"
            >
                {{ t("fileList.dateCreated")
                }}{{ sortField === "created" ? (sortAsc ? " ▲" : " ▼") : "" }}
                <div
                    class="col-handle"
                    @mousedown.stop="onResizeStart('created', $event)"
                ></div>
            </div>
            <div
                class="col-type"
                v-if="store.viewMode === 'details'"
                :style="{ width: colWidths.type + 'px' }"
            >
                {{ t("fileList.type") }}
                <div
                    class="col-handle"
                    @mousedown.stop="onResizeStart('type', $event)"
                ></div>
            </div>
            <div
                class="col-size"
                v-if="store.viewMode === 'details'"
                :style="{ width: colWidths.size + 'px' }"
                @click="sortBy('size')"
            >
                {{ t("fileList.size")
                }}{{ sortField === "size" ? (sortAsc ? " ▲" : " ▼") : "" }}
            </div>
        </div>

        <!-- This PC view -->
        <ThisPcView v-if="!currentPath && !store.loading" />

        <!-- Loading state -->
        <div v-if="store.loading" class="loading-state">
            <div class="loading-spinner"></div>
            <span>{{ t("fileList.loading") }}</span>
        </div>

        <!-- Directory content -->
        <div
            v-if="currentPath && !store.loading"
            class="file-items"
            :class="'view-' + store.viewMode"
            :style="colVars"
            @dragover.prevent="onDragOver"
            @dragleave="onDragLeave"
            @drop="onDrop"
        >
            <div
                v-if="displayFiles.length === 0 && !store.loading"
                class="empty-state"
            >
                {{ t("fileList.emptyFolder") }}
            </div>

            <!-- Details & List views -->
            <DetailsListView
                v-if="store.viewMode === 'details' || store.viewMode === 'list'"
                :files="displayFiles"
                :compact="store.viewMode === 'list'"
                :show-path="isSearchTab"
                @file-click="onFileClick"
                @file-dbl-click="onFileDblClick"
                @file-context-menu="onFileContextMenu"
            />

            <!-- Tree view -->
            <TreeView
                v-else-if="store.viewMode === 'tree'"
                :items="treeVisible"
                @file-context-menu="onFileContextMenu"
            />

            <!-- Grid view -->
            <GridView
                v-else
                :files="displayFiles"
                @file-click="onFileClick"
                @file-dbl-click="onFileDblClick"
                @file-context-menu="onFileContextMenu"
            />
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import { useTabStore } from "@/stores/tabStore";
import type { FileEntry } from "@/types";
import * as tauri from "@/utils/tauri";

import ThisPcView from "@/components/ThisPcView.vue";
import DetailsListView from "@/components/DetailsListView.vue";
import TreeView, { type TreeViewItem } from "@/components/TreeView.vue";
import GridView from "@/components/GridView.vue";

const { t } = useI18n();
const store = useFileStore();
const tabStore = useTabStore();

const props = defineProps<{ paneId?: string }>();

const emit = defineEmits<{
    contextMenu: [e: MouseEvent];
    fileContextMenu: [file: FileEntry, e: MouseEvent];
    fileDrop: [targetDir: string, paths: string[], ctrlKey: boolean];
}>();

// ── Tab-aware computed properties ──
const activeTabData = computed(() => {
    if (!props.paneId) return null;
    const pane = tabStore.findPaneById(props.paneId);
    if (!pane) return null;
    return pane.tabs.find((t) => t.id === pane.activeTabId);
});

const isSearchTab = computed(() => activeTabData.value?.isSearch ?? false);

const currentFiles = computed({
    get: () => (activeTabData.value ? activeTabData.value.files : store.files),
    set: (v) => {
        if (activeTabData.value) activeTabData.value.files = v;
        else store.files = v;
    },
});
const currentPath = computed({
    get: () =>
        activeTabData.value ? activeTabData.value.path : store.currentPath,
    set: (v) => {
        if (activeTabData.value) activeTabData.value.path = v;
        else store.currentPath = v;
    },
});

// ── Sort state ──
const sortField = ref<"name" | "modified" | "created" | "size">("name");
const sortAsc = ref(true);
const listEl = ref<HTMLElement | null>(null);

const displayFiles = computed(() => {
    const source = currentFiles.value;
    const sorted = [...source].sort((a, b) => {
        let cmp = 0;
        if (a.is_dir && !b.is_dir) return -1;
        if (!a.is_dir && b.is_dir) return 1;
        switch (sortField.value) {
            case "name":
                cmp = a.name.localeCompare(b.name, undefined, {
                    sensitivity: "base",
                });
                break;
            case "modified":
                cmp = b.modified - a.modified;
                break;
            case "created":
                cmp = b.created - a.created;
                break;
            case "size":
                cmp = b.size - a.size;
                break;
        }
        return sortAsc.value ? cmp : -cmp;
    });
    return sorted;
});

function sortBy(field: "name" | "modified" | "created" | "size") {
    if (sortField.value === field) {
        sortAsc.value = !sortAsc.value;
    } else {
        sortField.value = field;
        sortAsc.value = true;
    }
}

// ── Column resize ──
const colWidths = reactive<Record<string, number>>(loadCW());
const colVars = computed(() => ({
    "--col-name": colWidths.name + "px",
    "--col-date": colWidths.date + "px",
    "--col-created": colWidths.created + "px",
    "--col-type": colWidths.type + "px",
    "--col-size": colWidths.size + "px",
}));

function loadCW() {
    try {
        const r = localStorage.getItem("cols");
        if (r)
            return {
                name: 280,
                date: 140,
                created: 140,
                type: 100,
                size: 90,
                ...JSON.parse(r),
            };
    } catch {}
    return { name: 280, date: 140, created: 140, type: 100, size: 90 };
}

function saveCW() {
    localStorage.setItem("cols", JSON.stringify(colWidths));
}

let _col: string | null = null,
    _sx = 0,
    _sw = 0;

function onResizeStart(col: string, e: MouseEvent) {
    _col = col;
    _sx = e.clientX;
    _sw = colWidths[col] || 100;
    addEventListener("mousemove", onResizeMove);
    addEventListener("mouseup", onResizeEnd);
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
    e.preventDefault();
}

function onResizeMove(e: MouseEvent) {
    if (!_col) return;
    colWidths[_col] = Math.max(60, _sw + e.clientX - _sx);
}

function onResizeEnd() {
    removeEventListener("mousemove", onResizeMove);
    removeEventListener("mouseup", onResizeEnd);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    _col = null;
    saveCW();
}

// ── Tree view computed ──
const treeVisible = computed<TreeViewItem[]>(() => {
    const result: TreeViewItem[] = [];
    function walk(items: FileEntry[], depth: number) {
        for (const item of items) {
            const expanded = item.is_dir && store.isTreeExpanded(item.path);
            const hasChildren = item.is_dir;
            result.push({ file: item, depth, expanded, hasChildren });
            if (expanded) {
                const children = store.getTreeChildren(item.path);
                if (children) {
                    walk(children, depth + 1);
                }
            }
        }
    }
    const source = currentFiles.value;
    const sorted = [...source].sort((a, b) => {
        if (a.is_dir && !b.is_dir) return -1;
        if (!a.is_dir && b.is_dir) return 1;
        return a.name.localeCompare(b.name, undefined, { sensitivity: "base" });
    });
    walk(sorted, 0);
    return result;
});

// ── File click handlers ──
function onFileClick(file: FileEntry, e: MouseEvent) {
    const multi = e.ctrlKey || e.metaKey;
    store.selectFile(file, multi);
}

async function onFileDblClick(file: FileEntry, _e: MouseEvent) {
    await store.openSelectedFile(file);
}

function onContextMenu(e: MouseEvent) {
    emit("contextMenu", e);
}

function onFileContextMenu(file: FileEntry, e: MouseEvent) {
    store.selectFile(file, e.ctrlKey || e.metaKey);
    emit("fileContextMenu", file, e);
}

// ── HTML5 Drag-and-Drop ──
const isDragOver = ref(false);

function onDragOver(e: DragEvent) {
    if (!currentPath.value) return;
    e.preventDefault();
    e.dataTransfer!.dropEffect = e.ctrlKey || e.metaKey ? "copy" : "move";
    isDragOver.value = true;
}

function onDragLeave() {
    isDragOver.value = false;
}

function onDrop(e: DragEvent) {
    e.preventDefault();
    isDragOver.value = false;
    const dir = currentPath.value;
    if (!dir || !e.dataTransfer) return;

    let paths: string[] = [];

    // Internal drag (from our FileItem via custom MIME type)
    const internal = e.dataTransfer.getData(
        "application/x-files-explorer-paths",
    );
    if (internal) {
        try {
            paths = JSON.parse(internal);
        } catch {}
    }
    // External drag — read file URIs
    if (paths.length === 0) {
        const uriText = e.dataTransfer.getData("text/uri-list");
        if (uriText) {
            paths = uriText
                .split("\r\n")
                .filter(Boolean)
                .map((u) => {
                    let p = u.replace(/^file:\/+/i, "");
                    try {
                        p = decodeURIComponent(p);
                    } catch {}
                    return p.replace(/\//g, "\\");
                });
        }
    }

    if (paths.length === 0) return;

    const ctrl = e.ctrlKey || e.metaKey;
    tauri
        .moveFiles(paths, dir, ctrl)
        .then(async () => {
            currentFiles.value = await tauri.listDirectory(dir);
        })
        .catch((err) => console.error("Drop failed:", err));
}
</script>

<style scoped>
.file-list {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--bg-primary);
    position: relative;
}

.file-list.drop-active {
    background: rgba(137, 180, 250, 0.04);
    outline: 2px dashed var(--accent);
    outline-offset: -2px;
}

.drop-indicator {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    z-index: 50;
    color: var(--accent);
    font-size: 14px;
    font-weight: 500;
    pointer-events: none;
    background: rgba(137, 180, 250, 0.06);
    backdrop-filter: blur(2px);
}

.drop-icon {
    width: 32px;
    height: 32px;
    opacity: 0.8;
}

/* ── Column header ── */
.file-list-header {
    display: flex;
    align-items: center;
    padding: 2px 8px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    font-size: 12px;
    color: var(--text-muted);
    font-weight: 500;
    min-height: 28px;
    white-space: nowrap;
    flex-shrink: 0;
}
.file-list-header > div:not(.col-handle) {
    cursor: pointer;
    padding: 4px 6px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
    overflow: hidden;
    position: relative;
}
.file-list-header > div:not(.col-handle):hover {
    background: var(--bg-hover);
}
.col-handle {
    position: absolute;
    right: 0;
    top: 0;
    width: 5px;
    height: 100%;
    cursor: col-resize;
    z-index: 1;
    border-radius: 2px;
    transition: background 0.15s;
}
.col-handle:hover {
    background: var(--accent);
}
.col-name {
    width: var(--col-name, 280px);
    min-width: 120px;
    flex-shrink: 0;
}
.col-date {
    width: var(--col-date, 140px);
    min-width: 80px;
    flex-shrink: 0;
}
.col-created {
    width: var(--col-created, 140px);
    min-width: 80px;
    flex-shrink: 0;
}
.col-type {
    width: var(--col-type, 100px);
    min-width: 60px;
    flex-shrink: 0;
}
.col-size {
    width: var(--col-size, 90px);
    min-width: 60px;
    flex-shrink: 0;
    text-align: right;
}
.col-path-header {
    width: 260px;
    flex-shrink: 0;
    cursor: default !important;
}

/* ── File items wrapper ── */
.file-items {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
}

/* ── Loading & empty states ── */
.loading-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--text-muted);
    font-size: 14px;
}

.empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 40px;
    color: var(--text-muted);
    font-size: 14px;
}
</style>
