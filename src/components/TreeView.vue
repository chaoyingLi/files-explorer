<template>
    <div class="file-items view-tree">
        <div
            v-for="item in items"
            :key="item.file.path"
            class="tree-item"
            :class="{
                selected: isSelected(item.file.path),
                'tree-dir': item.file.is_dir,
            }"
            :style="{ paddingLeft: 12 + item.depth * 18 + 'px' }"
            @click="onItemClick(item, $event)"
            @dblclick="onItemDoubleClick(item)"
            @contextmenu.prevent="onContextMenu(item.file, $event)"
        >
            <!-- Expand/collapse arrow -->
            <span
                class="tree-arrow"
                :class="{
                    expanded: item.expanded,
                    invisible: !item.hasChildren,
                }"
                @click.stop="onArrowClick(item)"
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
                :class="item.file.is_dir ? '' : treeColorClass(item.file)"
            >
                <!-- Per-extension rich SVG icon (same as FileItem) -->
                <div
                    v-if="richIcon(item.file)"
                    class="tree-rich-icon"
                    v-html="richIcon(item.file)"
                ></div>
                <!-- Folder icon (skip for bundles) -->
                <svg
                    v-else-if="item.file.is_dir && !isBundle(item.file)"
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
                formatFileSize(item.file.size)
            }}</span>
        </div>
    </div>
</template>

<script setup lang="ts">
import { useFileStore } from "@/stores/fileStore";
import type { FileEntry } from "@/types";
import {
    getFileCategory,
    treeColorClassForCategory,
    formatFileSize,
} from "@/utils/fileTypes";
import { getFileIconSvg, isBundleDirectory } from "@/utils/fileIcons";

export interface TreeViewItem {
    file: FileEntry;
    depth: number;
    expanded: boolean;
    hasChildren: boolean;
}

defineProps<{
    items: TreeViewItem[];
}>();

const emit = defineEmits<{
    fileContextMenu: [file: FileEntry, e: MouseEvent];
}>();

const store = useFileStore();

function isSelected(path: string): boolean {
    return store.selectedFiles.has(path);
}

function treeColorClass(file: FileEntry): string {
    return treeColorClassForCategory(
        getFileCategory(file.extension, file.is_dir),
    );
}

function richIcon(file: FileEntry): string | null {
    if (file.is_dir && !isBundleDirectory(file.extension, file.is_dir))
        return null;
    return getFileIconSvg(file.extension, false);
}

function isBundle(file: FileEntry): boolean {
    return isBundleDirectory(file.extension, file.is_dir);
}

function onArrowClick(item: TreeViewItem) {
    if (item.file.is_dir) {
        store.toggleTreeExpand(item.file.path);
    }
}

function onItemClick(item: TreeViewItem, e: MouseEvent) {
    const multi = e.ctrlKey || e.metaKey;
    store.selectFile(item.file, multi);
}

async function onItemDoubleClick(item: TreeViewItem) {
    if (item.file.is_dir) {
        await store.toggleTreeExpand(item.file.path);
    } else {
        await store.openSelectedFile(item.file);
    }
}

function onContextMenu(file: FileEntry, e: MouseEvent) {
    store.selectFile(file, e.ctrlKey || e.metaKey);
    emit("fileContextMenu", file, e);
}
</script>

<style scoped>
.view-tree {
    padding: 2px 0;
}
.tree-item {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 3px 12px;
    cursor: pointer;
    font-size: 13px;
    min-height: 34px;
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

.tree-rich-icon {
    width: 18px;
    height: 18px;
}

.tree-rich-icon svg {
    width: 100%;
    height: 100%;
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
    white-space: nowrap;
}
.tree-size {
    flex-shrink: 0;
    font-size: var(--font-size-sm, 11px);
    color: var(--text-muted);
    margin-left: auto;
    padding-left: 12px;
    white-space: nowrap;
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
/* ── macOS folder blue ── */
[data-platform="macos"] .tree-icon-wrap {
    --file-icon-primary: #5ea8f5;
    --file-icon-secondary: #3b8de0;
    --folder-back: #3b8de0;
}
[data-platform="macos"][data-theme="light"] .tree-icon-wrap {
    --file-icon-primary: #3994f5;
    --file-icon-secondary: #2578d8;
    --folder-back: #2578d8;
}
</style>
