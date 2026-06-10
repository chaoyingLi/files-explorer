<template>
    <div
        class="file-list"
        @click.self="store.clearSelection()"
        @contextmenu.prevent="onContextMenu"
    >
        <div class="file-list-header">
            <div class="col-name" @click="sortBy('name')">
                {{ t("fileList.name") }}
                <span v-if="sortField === 'name'" class="sort-arrow">{{
                    sortAsc ? "▲" : "▼"
                }}</span>
            </div>
            <div v-if="isSearchTab" class="col-path-header">
                {{ t("fileList.path") }}
            </div>
            <div class="col-date" @click="sortBy('modified')">
                {{ t("fileList.dateModified") }}
                <span v-if="sortField === 'modified'" class="sort-arrow">{{
                    sortAsc ? "▲" : "▼"
                }}</span>
            </div>
            <div class="col-created" @click="sortBy('created')">
                {{ t("fileList.dateCreated") }}
                <span v-if="sortField === 'created'" class="sort-arrow">{{
                    sortAsc ? "▲" : "▼"
                }}</span>
            </div>
            <div class="col-type">{{ t("fileList.type") }}</div>
            <div class="col-size" @click="sortBy('size')">
                {{ t("fileList.size") }}
                <span v-if="sortField === 'size'" class="sort-arrow">{{
                    sortAsc ? "▲" : "▼"
                }}</span>
            </div>
        </div>

        <div v-if="!currentPath && !store.loading" class="this-pc">
            <div class="this-pc-header">
                {{ t("fileList.devicesAndDrives") }}
            </div>
            <div class="drives-grid">
                <div
                    v-for="drive in store.drives"
                    :key="drive.mount_point"
                    class="drive-card"
                    @dblclick="store.openDrive(drive)"
                >
                    <div class="drive-card-top">
                        <svg class="drive-card-icon" viewBox="0 0 48 48">
                            <rect
                                x="6"
                                y="10"
                                width="36"
                                height="28"
                                rx="4"
                                fill="#6C7086"
                            />
                            <rect
                                x="10"
                                y="14"
                                width="28"
                                height="20"
                                rx="2"
                                fill="#45475A"
                            />
                            <circle cx="18" cy="24" r="4" fill="#F5C542" />
                            <circle cx="18" cy="24" r="2" fill="#F9E2AF" />
                        </svg>
                        <div class="drive-card-info">
                            <div class="drive-card-name">
                                <span v-if="drive.label" class="drive-label">{{
                                    drive.label
                                }}</span>
                                <span class="drive-letter">{{
                                    drive.name
                                }}</span>
                            </div>
                            <div
                                v-if="drive.total_space > 0"
                                class="drive-space"
                            >
                                <div class="drive-progress">
                                    <div
                                        class="drive-progress-bar"
                                        :style="{
                                            width: usePercent(drive) + '%',
                                        }"
                                    ></div>
                                </div>
                                <div class="drive-space-text">
                                    {{ formatSize(drive.available_space) }} free
                                    of {{ formatSize(drive.total_space) }}
                                </div>
                            </div>
                            <div v-else class="drive-space-text">
                                {{ drive.file_system }}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            <div class="this-pc-header" style="margin-top: 24px">
                {{ t("fileList.folders") }}
            </div>
            <div class="drives-grid">
                <div
                    v-for="item in quickAccessFolders"
                    :key="item.path"
                    class="drive-card"
                    @dblclick="store.navigateTo(item.path)"
                >
                    <svg
                        class="drive-card-icon folder-icon"
                        viewBox="0 0 48 48"
                    >
                        <path
                            d="M4 12a3 3 0 013-3h10.6a3 3 0 012.4 1.2l3.2 4.2a2 2 0 001.6.8H41a3 3 0 013 3v18a3 3 0 01-3 3H7a3 3 0 01-3-3V12z"
                            fill="#DEB949"
                        />
                        <path
                            d="M4 15a3 3 0 013-3h10.6a3 3 0 012.4 1.2l3.2 4.2a2 2 0 001.6.8H41a3 3 0 013 3v16a3 3 0 01-3 3H7a3 3 0 01-3-3V12z"
                            fill="#F5C542"
                        />
                    </svg>
                    <div class="drive-card-label">{{ item.name }}</div>
                </div>
            </div>
        </div>

        <div v-if="store.loading" class="loading-state">
            <div class="loading-spinner"></div>
            <span>{{ t("fileList.loading") }}</span>
        </div>

        <div
            v-if="currentPath && !store.loading"
            class="file-items"
            :class="'view-' + store.viewMode"
        >
            <div
                v-if="displayFiles.length === 0 && !store.loading"
                class="empty-state"
            >
                {{ t("fileList.emptyFolder") }}
            </div>
            <!-- Details & List views -->
            <template
                v-if="store.viewMode === 'details' || store.viewMode === 'list'"
            >
                <FileItem
                    v-for="file in displayFiles"
                    :key="file.path"
                    :file="file"
                    :compact="store.viewMode === 'list'"
                    :selected="store.selectedFiles.has(file.path)"
                    :is-cut="store.isFileCut(file.path)"
                    :show-path="isSearchTab"
                    @click="onFileClick(file, $event)"
                    @dblclick="store.openSelectedFile(file)"
                    @contextmenu="onFileContextMenu(file, $event)"
                />
            </template>
            <!-- Tree view -->
            <div
                v-else-if="store.viewMode === 'tree'"
                class="file-items view-tree"
            >
                <div
                    v-for="item in treeVisible"
                    :key="item.file.path"
                    class="tree-item"
                    :class="{
                        selected: store.selectedFiles.has(item.file.path),
                        'tree-dir': item.file.is_dir,
                    }"
                    :style="{ paddingLeft: 12 + item.depth * 18 + 'px' }"
                    @click="onTreeItemClick(item, $event)"
                    @dblclick="onTreeItemDoubleClick(item)"
                    @contextmenu.prevent="onFileContextMenu(item.file, $event)"
                >
                    <!-- Expand/collapse arrow -->
                    <span
                        class="tree-arrow"
                        :class="{
                            expanded: item.expanded,
                            invisible: !item.hasChildren,
                        }"
                        @click.stop="onTreeArrowClick(item)"
                    >
                        <svg viewBox="0 0 10 10" width="10" height="10">
                            <path
                                d="M3.5 2l3.5 3-3.5 3"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="1.3"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                            />
                        </svg>
                    </span>
                    <!-- Icon (Win11 Fluent style, category-colored) -->
                    <span
                        class="tree-icon-wrap"
                        :class="
                            item.file.is_dir ? '' : treeColorClass(item.file)
                        "
                    >
                        <!-- Folder icon -->
                        <svg
                            v-if="item.file.is_dir"
                            class="tree-icon"
                            viewBox="0 0 18 18"
                        >
                            <path
                                d="M2 5.5c0-.83.67-1.5 1.5-1.5h2.5a1.5 1.5 0 011.1.5l.7.85a.5.5 0 00.38.18H14c.83 0 1.5.67 1.5 1.5v4.5a1.5 1.5 0 01-1.5 1.5h-11A1.5 1.5 0 012 12V5.5z"
                                fill="var(--folder-back)"
                            />
                            <path
                                d="M2 6.5c0-.83.67-1.5 1.5-1.5h2.5a1.5 1.5 0 011.1.5l.7.85a.5.5 0 00.38.18H14c.83 0 1.5.67 1.5 1.5v4a1.5 1.5 0 01-1.5 1.5h-11A1.5 1.5 0 012 12V6.5z"
                                fill="var(--file-icon-primary)"
                            />
                        </svg>
                        <!-- File icon (Win11 Fluent style) -->
                        <svg v-else class="tree-icon" viewBox="0 0 18 18">
                            <path
                                d="M4.5 2h5.1l3.9 3.9V14a1.5 1.5 0 01-1.5 1.5h-7.5A1.5 1.5 0 013 14V3.5A1.5 1.5 0 014.5 2z"
                                fill="var(--file-icon-primary)"
                            />
                            <path
                                d="M9.6 2v2.65c0 .47.38.85.85.85H13"
                                fill="var(--file-icon-secondary)"
                            />
                            <rect
                                x="6"
                                y="10"
                                width="6"
                                height="1"
                                rx="0.5"
                                fill="var(--file-icon-lines)"
                                opacity="0.4"
                            />
                            <rect
                                x="6"
                                y="12"
                                width="4"
                                height="1"
                                rx="0.5"
                                fill="var(--file-icon-lines)"
                                opacity="0.4"
                            />
                        </svg>
                    </span>
                    <!-- Name -->
                    <span class="tree-name">{{ item.file.name }}</span>
                    <!-- Meta (date + size) on the right for files -->
                    <span v-if="!item.file.is_dir" class="tree-size">{{
                        formatSize(item.file.size)
                    }}</span>
                </div>
            </div>
            <!-- Grid view -->
            <div v-else class="grid-items">
                <div
                    v-for="file in displayFiles"
                    :key="file.path"
                    class="grid-card"
                    :class="{ selected: store.selectedFiles.has(file.path) }"
                    @click="onFileClick(file, $event)"
                    @dblclick="store.openSelectedFile(file)"
                    @contextmenu.prevent="onFileContextMenu(file, $event)"
                >
                    <div class="grid-icon" :class="gridColorClass(file)">
                        <!-- Win11-style Folder (48x48) -->
                        <svg
                            v-if="file.is_dir"
                            viewBox="0 0 48 48"
                            class="grid-folder"
                        >
                            <defs>
                                <linearGradient
                                    id="gf-grad"
                                    x1="0"
                                    y1="0"
                                    x2="0"
                                    y2="1"
                                >
                                    <stop
                                        offset="0%"
                                        stop-color="var(--folder-main)"
                                        stop-opacity="0.9"
                                    />
                                    <stop
                                        offset="100%"
                                        stop-color="var(--folder-shade)"
                                        stop-opacity="0.95"
                                    />
                                </linearGradient>
                                <linearGradient
                                    id="gf-tab"
                                    x1="0"
                                    y1="0"
                                    x2="0"
                                    y2="1"
                                >
                                    <stop
                                        offset="0%"
                                        stop-color="var(--folder-shade)"
                                        stop-opacity="0.6"
                                    />
                                    <stop
                                        offset="100%"
                                        stop-color="var(--folder-shade)"
                                        stop-opacity="0.4"
                                    />
                                </linearGradient>
                            </defs>
                            <path
                                d="M5 15a3.5 3.5 0 013.5-3.5h8.08c.97 0 1.88.47 2.44 1.26l2.34 3.24H39.5A3.5 3.5 0 0143 19.5v14a3.5 3.5 0 01-3.5 3.5H8.5A3.5 3.5 0 015 33.5V15z"
                                fill="url(#gf-tab)"
                            />
                            <path
                                d="M5 17.5A3.5 3.5 0 018.5 14h8.08c.97 0 1.88.47 2.44 1.26l2.34 3.24H39.5A3.5 3.5 0 0143 22v11.5a3.5 3.5 0 01-3.5 3.5H8.5A3.5 3.5 0 015 33.5V17.5z"
                                fill="url(#gf-grad)"
                            />
                        </svg>
                        <!-- Win11-style Document (48x48) -->
                        <svg v-else viewBox="0 0 48 48" class="grid-doc">
                            <defs>
                                <linearGradient
                                    id="gd-grad"
                                    x1="0"
                                    y1="0"
                                    x2="0"
                                    y2="1"
                                >
                                    <stop
                                        offset="0%"
                                        stop-color="var(--doc-main)"
                                        stop-opacity="0.9"
                                    />
                                    <stop
                                        offset="100%"
                                        stop-color="var(--doc-shade)"
                                        stop-opacity="0.85"
                                    />
                                </linearGradient>
                            </defs>
                            <path
                                d="M12 6h18.38l9.62 9.62V38a4 4 0 01-4 4H12a4 4 0 01-4-4V10a4 4 0 014-4z"
                                fill="url(#gd-grad)"
                            />
                            <path
                                d="M30.38 6v6.62c0 .55.45 1 1 1H38"
                                fill="var(--doc-highlight)"
                                opacity="0.5"
                            />
                        </svg>
                    </div>
                    <div class="grid-name">{{ file.name }}</div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import { useTabStore } from "@/stores/tabStore";
import type { FileEntry, DiskInfo } from "@/types";
import FileItem from "@/components/FileItem.vue";

const { t } = useI18n();
const store = useFileStore();
const tabStore = useTabStore();

const props = defineProps<{ paneId?: string }>();

const emit = defineEmits<{
    contextMenu: [e: MouseEvent];
    fileContextMenu: [file: FileEntry, e: MouseEvent];
}>();

// Get pane's active tab data - NEVER fall back to store when paneId is set
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

const sortField = ref<"name" | "modified" | "created" | "size">("name");
const sortAsc = ref(true);

const quickAccessFolders = computed(() => {
    const items: { name: string; path: string }[] = [];
    const dirs = store.specialDirs;
    if (!dirs) return items;

    items.push({ name: t("sidebar.desktop"), path: dirs.desktop });
    items.push({ name: t("sidebar.downloads"), path: dirs.downloads });
    items.push({ name: t("sidebar.documents"), path: dirs.documents });
    items.push({ name: t("sidebar.pictures"), path: dirs.pictures });
    items.push({ name: t("sidebar.music"), path: dirs.music });
    items.push({ name: t("sidebar.videos"), path: dirs.videos });
    return items;
});

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

// ── Tree view: builds tree items from tab-aware currentFiles ──
const treeVisible = computed(() => {
    const result: {
        file: FileEntry;
        depth: number;
        expanded: boolean;
        hasChildren: boolean;
    }[] = [];
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

function sortBy(field: "name" | "modified" | "created" | "size") {
    if (sortField.value === field) {
        sortAsc.value = !sortAsc.value;
    } else {
        sortField.value = field;
        sortAsc.value = true;
    }
}

function onFileClick(file: FileEntry, e: MouseEvent) {
    const multi = e.ctrlKey || e.metaKey;
    store.selectFile(file, multi);
}

function onContextMenu(e: MouseEvent) {
    emit("contextMenu", e);
}

function onFileContextMenu(file: FileEntry, e: MouseEvent) {
    store.selectFile(file, e.ctrlKey || e.metaKey);
    emit("fileContextMenu", file, e);
}

// ── Tree view handlers ──
function onTreeArrowClick(item: any) {
    if (item.file.is_dir) {
        store.toggleTreeExpand(item.file.path);
    }
}
function onTreeItemClick(item: any, e: MouseEvent) {
    const multi = e.ctrlKey || e.metaKey;
    store.selectFile(item.file, multi);
}
async function onTreeItemDoubleClick(item: any) {
    if (item.file.is_dir) {
        // Toggle expand on double-click
        await store.toggleTreeExpand(item.file.path);
    } else {
        await store.openSelectedFile(item.file);
    }
}
function formatSize(bytes: number): string {
    if (bytes === 0) return "0 B";
    const units = ["B", "KB", "MB", "GB", "TB"];
    let i = 0;
    let size = bytes;
    while (size >= 1024 && i < units.length - 1) {
        size /= 1024;
        i++;
    }
    return `${size.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

function usePercent(drive: DiskInfo): number {
    if (drive.total_space === 0) return 0;
    return Math.round((drive.used_space / drive.total_space) * 100);
}

function treeColorClass(file: FileEntry): string {
    if (file.is_dir) return "";
    const ext = file.extension.toLowerCase();
    if (
        ["js", "ts", "vue", "py", "rs", "go", "java", "c", "cpp", "h"].includes(
            ext,
        )
    )
        return "tree-color-code";
    if (
        ["png", "jpg", "jpeg", "gif", "svg", "webp", "bmp", "ico"].includes(ext)
    )
        return "tree-color-image";
    if (["mp3", "wav", "flac", "ogg", "aac"].includes(ext))
        return "tree-color-audio";
    if (["mp4", "avi", "mkv", "mov", "wmv"].includes(ext))
        return "tree-color-video";
    if (["zip", "rar", "7z", "tar", "gz", "xz"].includes(ext))
        return "tree-color-archive";
    if (["pdf"].includes(ext)) return "tree-color-pdf";
    if (["exe", "dll", "msi"].includes(ext)) return "tree-color-app";
    if (["html", "css", "scss", "less"].includes(ext)) return "tree-color-web";
    return "tree-color-default";
}

function gridColorClass(file: FileEntry): string {
    if (file.is_dir) return "grid-folder-color";
    const ext = file.extension.toLowerCase();
    if (
        ["js", "ts", "vue", "py", "rs", "go", "java", "c", "cpp", "h"].includes(
            ext,
        )
    )
        return "grid-color-code";
    if (["png", "jpg", "jpeg", "gif", "svg", "webp"].includes(ext))
        return "grid-color-image";
    if (["mp3", "wav", "flac"].includes(ext)) return "grid-color-audio";
    if (["mp4", "avi", "mkv"].includes(ext)) return "grid-color-video";
    if (["zip", "rar", "7z", "tar", "gz"].includes(ext))
        return "grid-color-archive";
    if (["pdf"].includes(ext)) return "grid-color-pdf";
    if (["exe", "dll", "msi"].includes(ext)) return "grid-color-app";
    if (["html", "css", "scss", "less"].includes(ext)) return "grid-color-web";
    return "grid-color-default";
}
</script>

<style scoped>
.file-list {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--bg-primary);
}

.file-list-header {
    display: flex;
    align-items: center;
    padding: 4px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    font-size: 12px;
    color: var(--text-muted);
    font-weight: 500;
    min-height: 28px;
}

.file-list-header > div {
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    gap: 4px;
}

.file-list-header > div:hover {
    background: var(--bg-hover);
}

.col-path-header {
    width: 260px;
    flex-shrink: 0;
    cursor: default !important;
}

.col-name {
    flex: 1;
}
.col-date {
    width: 160px;
    flex-shrink: 0;
}

.col-created {
    width: 160px;
    flex-shrink: 0;
}
.col-type {
    width: 100px;
    flex-shrink: 0;
    cursor: default !important;
}
.col-type:hover {
    background: transparent !important;
}
.col-size {
    width: 100px;
    flex-shrink: 0;
    justify-content: flex-end;
}
.sort-arrow {
    font-size: 10px;
}

.this-pc {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
}

.this-pc-header {
    font-size: 14px;
    font-weight: 600;
    margin-bottom: 12px;
    color: var(--text-primary);
}

.drives-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
}

.drive-card {
    display: flex;
    flex-direction: column;
    padding: 14px 16px;
    border-radius: 10px;
    cursor: pointer;
    transition: background 0.1s;
    border: 1px solid var(--border);
    width: 200px;
    flex-shrink: 0;
}

.drive-card:hover {
    background: var(--bg-hover);
}

.drive-card-top {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
}

.drive-card-icon {
    width: 44px;
    height: 44px;
    flex-shrink: 0;
    filter: drop-shadow(0 1px 3px rgba(0, 0, 0, 0.15));
}

.drive-card-info {
    width: 100%;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 5px;
}

.drive-card-name {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1px;
}

.drive-label {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    text-align: center;
    word-break: break-word;
}

.drive-letter {
    font-size: 11px;
    color: var(--text-muted);
}

.drive-space {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.drive-progress {
    height: 4px;
    background: var(--bg-hover);
    border-radius: 2px;
    overflow: hidden;
}

.drive-progress-bar {
    height: 100%;
    background: var(--accent);
    border-radius: 2px;
    transition: width 0.3s ease;
    min-width: 2px;
}

.drive-space-text {
    font-size: 10px;
    color: var(--text-muted);
    text-align: center;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.file-items {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
}

.file-items.view-list .file-item {
    min-height: 26px;
    padding: 1px 12px;
}

.file-items.view-list .col-date,
.file-items.view-list .col-type,
.file-items.view-list .col-size {
    display: none;
}

/* Grid view */
.grid-items {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
    gap: 4px;
    padding: 8px;
    align-content: start;
}

.grid-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 10px 6px;
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.1s;
    text-align: center;
    min-height: 0;
}

.grid-card:hover {
    background: var(--bg-hover);
}

.grid-card.selected {
    background: var(--bg-selected);
}

.grid-icon {
    width: 48px;
    height: 48px;
    margin-bottom: 6px;
    flex-shrink: 0;
}

.grid-icon svg {
    width: 100%;
    height: 100%;
    filter: drop-shadow(0 2px 3px rgba(0, 0, 0, 0.15));
}

.grid-name {
    font-size: 12px;
    line-height: 1.3;
    word-break: break-word;
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    max-width: 100%;
}

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

.search-results-header {
    padding: 8px 16px;
    font-size: 13px;
    color: var(--text-muted);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
}

/* ── Grid view icon colors (Win11 style, theme-aware) ── */
/* Default (folder) colors - set on root for the .grid-icon */
.grid-icon {
    --folder-main: #f5c542;
    --folder-shade: #e8b825;
    --doc-main: #7890b0;
    --doc-shade: #5a7295;
    --doc-highlight: #9ab0cc;
}

/* Grid file category colors - dark theme */
[data-theme="dark"] .grid-color-code {
    --doc-main: #a6e3a1;
    --doc-shade: #7bc47a;
    --doc-highlight: #c8f0c4;
}
[data-theme="dark"] .grid-color-image {
    --doc-main: #cba6f7;
    --doc-shade: #b485e8;
    --doc-highlight: #dec0fa;
}
[data-theme="dark"] .grid-color-audio {
    --doc-main: #f9e2af;
    --doc-shade: #e8c77a;
    --doc-highlight: #fcf0d0;
}
[data-theme="dark"] .grid-color-video {
    --doc-main: #f38ba8;
    --doc-shade: #e46d8e;
    --doc-highlight: #f8b0c4;
}
[data-theme="dark"] .grid-color-archive {
    --doc-main: #f5c542;
    --doc-shade: #dba42e;
    --doc-highlight: #f9e2af;
}
[data-theme="dark"] .grid-color-pdf {
    --doc-main: #f38ba8;
    --doc-shade: #e46d8e;
    --doc-highlight: #f8b0c4;
}
[data-theme="dark"] .grid-color-app {
    --doc-main: #89b4fa;
    --doc-shade: #5f9cf0;
    --doc-highlight: #b8d4fc;
}
[data-theme="dark"] .grid-color-web {
    --doc-main: #fab387;
    --doc-shade: #e8955e;
    --doc-highlight: #fccca8;
}
[data-theme="dark"] .grid-color-default {
    --doc-main: #7890b0;
    --doc-shade: #5a7295;
    --doc-highlight: #9ab0cc;
}

/* Grid file category colors - light theme */
[data-theme="light"] .grid-icon {
    --folder-main: #df8e1d;
    --folder-shade: #c47a15;
    --doc-main: #8c8fa0;
    --doc-shade: #70748c;
    --doc-highlight: #a8acc0;
}
[data-theme="light"] .grid-color-code {
    --doc-main: #40a02b;
    --doc-shade: #2e801e;
    --doc-highlight: #6cc05a;
}
[data-theme="light"] .grid-color-image {
    --doc-main: #8839ef;
    --doc-shade: #7020d5;
    --doc-highlight: #a868f8;
}
[data-theme="light"] .grid-color-audio {
    --doc-main: #df8e1d;
    --doc-shade: #c47a15;
    --doc-highlight: #f0b050;
}
[data-theme="light"] .grid-color-video {
    --doc-main: #d20f39;
    --doc-shade: #b0082a;
    --doc-highlight: #e84560;
}
[data-theme="light"] .grid-color-archive {
    --doc-main: #df8e1d;
    --doc-shade: #c47a15;
    --doc-highlight: #f0b050;
}
[data-theme="light"] .grid-color-pdf {
    --doc-main: #d20f39;
    --doc-shade: #b0082a;
    --doc-highlight: #e84560;
}
[data-theme="light"] .grid-color-app {
    --doc-main: #1e66f5;
    --doc-shade: #0d4fd8;
    --doc-highlight: #5090f8;
}
[data-theme="light"] .grid-color-web {
    --doc-main: #fe640b;
    --doc-shade: #d95208;
    --doc-highlight: #fe8a48;
}
[data-theme="light"] .grid-color-default {
    --doc-main: #8c8fa0;
    --doc-shade: #70748c;
    --doc-highlight: #a8acc0;
}

/* ── Tree view ── */
.view-tree {
    padding: 2px 0;
}

.tree-item {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 1px 8px 1px 12px;
    cursor: pointer;
    font-size: var(--font-size-base, 13px);
    min-height: 24px;
    border-radius: 4px;
    margin: 0 4px;
    transition: background 0.05s;
    white-space: nowrap;
}

.tree-item:hover {
    background: var(--bg-hover);
}

.tree-item.selected {
    background: var(--bg-selected);
}

.tree-arrow {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 3px;
    color: var(--text-muted);
    transition: transform 0.12s;
}

.tree-arrow:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
}

.tree-arrow.expanded {
    transform: rotate(90deg);
}

.tree-arrow.invisible {
    visibility: hidden;
}

.tree-icon-wrap {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
}

.tree-icon {
    width: 16px;
    height: 16px;
}

.tree-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-left: 4px;
}

.tree-size {
    flex-shrink: 0;
    font-size: var(--font-size-sm, 11px);
    color: var(--text-muted);
    margin-left: auto;
    padding-left: 12px;
}

.tree-dir .tree-name {
    font-weight: 500;
}

/* ── Tree view category colors (dark theme) ── */
[data-theme="dark"] .tree-color-code {
    --file-icon-primary: #a6e3a1;
    --file-icon-secondary: #7bc47a;
}
[data-theme="dark"] .tree-color-image {
    --file-icon-primary: #cba6f7;
    --file-icon-secondary: #b485e8;
}
[data-theme="dark"] .tree-color-audio {
    --file-icon-primary: #f9e2af;
    --file-icon-secondary: #e8c77a;
}
[data-theme="dark"] .tree-color-video {
    --file-icon-primary: #f38ba8;
    --file-icon-secondary: #e46d8e;
}
[data-theme="dark"] .tree-color-archive {
    --file-icon-primary: #f5c542;
    --file-icon-secondary: #dba42e;
}
[data-theme="dark"] .tree-color-pdf {
    --file-icon-primary: #f38ba8;
    --file-icon-secondary: #e46d8e;
}
[data-theme="dark"] .tree-color-app {
    --file-icon-primary: #89b4fa;
    --file-icon-secondary: #5f9cf0;
}
[data-theme="dark"] .tree-color-web {
    --file-icon-primary: #fab387;
    --file-icon-secondary: #e8955e;
}
[data-theme="dark"] .tree-color-default {
    --file-icon-primary: #7890b0;
    --file-icon-secondary: #5a7295;
}

/* ── Tree view category colors (light theme) ── */
[data-theme="light"] .tree-color-code {
    --file-icon-primary: #40a02b;
    --file-icon-secondary: #2e801e;
}
[data-theme="light"] .tree-color-image {
    --file-icon-primary: #8839ef;
    --file-icon-secondary: #7020d5;
}
[data-theme="light"] .tree-color-audio {
    --file-icon-primary: #df8e1d;
    --file-icon-secondary: #c47a15;
}
[data-theme="light"] .tree-color-video {
    --file-icon-primary: #d20f39;
    --file-icon-secondary: #b0082a;
}
[data-theme="light"] .tree-color-archive {
    --file-icon-primary: #df8e1d;
    --file-icon-secondary: #c47a15;
}
[data-theme="light"] .tree-color-pdf {
    --file-icon-primary: #d20f39;
    --file-icon-secondary: #b0082a;
}
[data-theme="light"] .tree-color-app {
    --file-icon-primary: #1e66f5;
    --file-icon-secondary: #0d4fd8;
}
[data-theme="light"] .tree-color-web {
    --file-icon-primary: #fe640b;
    --file-icon-secondary: #d95208;
}
[data-theme="light"] .tree-color-default {
    --file-icon-primary: #8c8fa0;
    --file-icon-secondary: #70748c;
}
</style>
