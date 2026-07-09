<template>
    <div
        class="column-pane"
        :style="{ width: width + 'px', minWidth: MIN_COL_WIDTH + 'px' }"
    >
        <div class="column-header" :title="column.name">{{ column.name }}</div>
        <div class="column-body" @click="selectFirstIfNone">
            <div v-if="column.loading" class="column-loading">
                <span>{{ t("fileList.loading") }}</span>
            </div>
            <div
                v-for="(file, idx) in column.files"
                :key="file.path || file.name"
                class="column-item"
                :class="{ selected: isSelected(file.path) }"
                @click="onClick(idx, $event)"
                @dblclick="onDblClick(idx)"
                @contextmenu.prevent="onContextMenu(idx, $event)"
            >
                <div class="column-icon" v-html="getIcon(file)"></div>
                <span class="column-name">{{ file.name }}</span>
                <span v-if="file.is_dir" class="column-arrow">›</span>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { FileEntry } from "@/types";
import type { ColumnState } from "@/stores/viewStore";
import { useSelectionStore } from "@/stores/selectionStore";
import { getFileIconSvg, isBundleDirectory } from "@/utils/fileIcons";

const MIN_COL_WIDTH = 120;

const { t } = useI18n();
const sel = useSelectionStore();
const props = defineProps<{
    column: ColumnState;
    colIdx: number;
    width: number;
}>();
const emit = defineEmits<{
    select: [colIdx: number, fileIdx: number];
    dblclick: [colIdx: number, fileIdx: number];
    contextmenu: [file: FileEntry, event: MouseEvent, colIdx: number];
}>();

function isSelected(path: string): boolean {
    return sel.isSelected(path);
}

function getIcon(file: FileEntry): string | null {
    if (file.is_dir && !isBundleDirectory(file.extension, file.is_dir)) {
        return `<svg viewBox="0 0 16 16"><path d="M1.5 4A1.5 1.5 0 013 2.5h3.2a1.5 1.5 0 011.2.6l.8 1H13A1.5 1.5 0 0114.5 5.5V12a1.5 1.5 0 01-1.5 1.5H3A1.5 1.5 0 011.5 12V4z" fill="var(--file-icon-primary)"/><path d="M1.5 5A1.5 1.5 0 013 3.5h3.2a1.5 1.5 0 011.2.6l.8 1H13A1.5 1.5 0 0114.5 6.5v5a1.5 1.5 0 01-1.5 1.5H3A1.5 1.5 0 011.5 11V5z" fill="var(--folder-back)"/></svg>`;
    }
    return getFileIconSvg(file.extension, file.is_dir);
}

function onClick(idx: number, e: MouseEvent) {
    const file = props.column.files[idx];
    if (!file) return;
    const shift = e.shiftKey;
    const ctrl = e.ctrlKey || e.metaKey;
    // 直接在 ColumnPane 中处理多选，避免修改 emit 签名
    if (shift && sel.selectedFiles.size > 0) {
        sel.selectRange(props.column.files, sel.anchorIndex, idx);
        sel.focusedIndex = idx;
    } else if (ctrl) {
        sel.toggleSelectFile(file);
        sel.focusedIndex = idx;
    } else {
        sel.selectedFiles = new Set([file.path]);
        sel.focusedIndex = idx;
        sel.anchorIndex = idx;
    }
    emit("select", props.colIdx, idx);
}
function onDblClick(idx: number) {
    emit("dblclick", props.colIdx, idx);
}
function onContextMenu(idx: number, event: MouseEvent) {
    const file = props.column.files[idx];
    if (file) emit("contextmenu", file, event, props.colIdx);
}
function selectFirstIfNone(e: MouseEvent) {
    if (e.target !== e.currentTarget) return;
    if (props.column.selectedIndex === -1 && props.column.files.length > 0) {
        emit("select", props.colIdx, 0);
    }
}
</script>

<style scoped>
.column-pane {
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--border);
    background: var(--bg-primary);
}
.column-header {
    padding: 8px 12px;
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    border-bottom: 1px solid var(--border);
    background: var(--bg-secondary);
}
.column-body {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
}
.column-loading {
    padding: 12px;
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    text-align: center;
}
.column-item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 12px;
    cursor: pointer;
    font-size: var(--font-size-base);
    min-height: 28px;
    transition: background 0.05s;
}
.column-item:hover {
    background: var(--bg-hover);
}
.column-item.selected {
    background: var(--bg-selected);
}
.column-icon {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
}
.column-icon :deep(svg) {
    width: 16px;
    height: 16px;
}
.column-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.column-arrow {
    color: var(--accent);
    font-size: var(--font-size-lg);
    flex-shrink: 0;
}
</style>
