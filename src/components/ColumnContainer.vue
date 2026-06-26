<template>
    <div
        class="column-container"
        ref="scrollRef"
        tabindex="0"
        @keydown="onKeydown"
    >
        <template v-for="(col, idx) in stack" :key="idx">
            <ColumnPane
                :column="col"
                :col-idx="idx"
                :width="getColWidth(idx)"
                @select="onSelect"
                @dblclick="onDblClick"
                @contextmenu="onContextMenu"
            />
            <!-- Resize handle between columns (not after the last column) -->
            <div
                v-if="idx < stack.length - 1"
                class="column-resize-handle"
                @mousedown.stop="onResizeStart(idx, $event)"
            />
        </template>
        <div class="column-filler" />
    </div>
</template>

<script setup lang="ts">
import { ref, reactive, nextTick } from "vue";
import type { FileEntry } from "@/types";
import type { ColumnState } from "@/stores/viewStore";
import { useFileStore } from "@/stores/fileStore";
import { useViewStore } from "@/stores/viewStore";
import { useSelectionStore } from "@/stores/selectionStore";
import * as tauri from "@/utils/tauri";
import ColumnPane from "./ColumnPane.vue";

const props = defineProps<{ stack: ColumnState[] }>();
const emit = defineEmits<{
    contextMenu: [file: FileEntry, e: MouseEvent];
    updateStack: [stack: ColumnState[]];
}>();

const store = useFileStore();
const view = useViewStore();
const sel = useSelectionStore();
const scrollRef = ref<HTMLElement>();

// ── Column resize ──
const DEFAULT_COL_WIDTH = 240;
const MIN_COL_WIDTH = 120;

const colWidths = reactive<number[]>(loadColWidths());

function loadColWidths(): number[] {
    try {
        const raw = localStorage.getItem("colw");
        if (raw) {
            const parsed = JSON.parse(raw);
            if (Array.isArray(parsed)) return parsed;
        }
    } catch {
        /* ignore */
    }
    return [];
}

function saveColWidths() {
    localStorage.setItem("colw", JSON.stringify(colWidths));
}

function getColWidth(idx: number): number {
    return colWidths[idx] || DEFAULT_COL_WIDTH;
}

let _resizingIdx = -1;
let _sx = 0;
let _sw = 0;

function onResizeStart(idx: number, e: MouseEvent) {
    _resizingIdx = idx;
    _sx = e.clientX;
    _sw = getColWidth(idx);
    addEventListener("mousemove", onResizeMove);
    addEventListener("mouseup", onResizeEnd);
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
    e.preventDefault();
}

function onResizeMove(e: MouseEvent) {
    if (_resizingIdx < 0) return;
    colWidths[_resizingIdx] = Math.max(MIN_COL_WIDTH, _sw + e.clientX - _sx);
}

function onResizeEnd() {
    removeEventListener("mousemove", onResizeMove);
    removeEventListener("mouseup", onResizeEnd);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    _resizingIdx = -1;
    saveColWidths();
}

// ── Selection sync ──

/**
 * Sync the selected file from column view to the global selectionStore,
 * so that PropertiesPanel and other features can read it.
 */
function syncSelection(colIdx: number, fileIdx: number) {
    const file = props.stack[colIdx]?.files[fileIdx];
    if (file) {
        sel.selectedFiles = new Set([file.path]);
    }
}

// ── Navigation ──

/**
 * Single-click on a column item:
 * - Always sync selection to selectionStore (for PropertiesPanel, etc.)
 * - Directory  → load its contents into the next column
 * - File       → select only, no column push
 */
async function onSelect(colIdx: number, fileIdx: number) {
    const file = props.stack[colIdx]?.files[fileIdx];
    if (!file) return;
    syncSelection(colIdx, fileIdx);
    if (!file.is_dir) return;
    await view.columnLoadDirectory(props.stack, colIdx, file);
    emit("updateStack", [...props.stack]);
    await nextTick();
    if (scrollRef.value)
        scrollRef.value.scrollLeft = scrollRef.value.scrollWidth;
}

/**
 * Double-click on a column item:
 * - Sync selection
 * - Directory  → load its contents into the next column
 * - File       → open with the system default application
 */
async function onDblClick(colIdx: number, fileIdx: number) {
    const file = props.stack[colIdx]?.files[fileIdx];
    if (!file) return;
    syncSelection(colIdx, fileIdx);
    if (file.is_dir) {
        await view.columnLoadDirectory(props.stack, colIdx, file);
        emit("updateStack", [...props.stack]);
    } else {
        try {
            await tauri.openFile(file.path);
        } catch (e) {
            console.error("Failed to open file:", e);
        }
    }
}

function onContextMenu(file: FileEntry, event: MouseEvent) {
    sel.selectFile(file, event.ctrlKey || event.metaKey);
    emit("contextMenu", file, event);
}

function onKeydown(e: KeyboardEvent) {
    if (e.target instanceof HTMLInputElement) return;
    const lastIdx = props.stack.length - 1;
    const col = props.stack[lastIdx];
    const selFile =
        col && col.selectedIndex >= 0 ? col.files[col.selectedIndex] : null;

    if (e.key === "ArrowUp") {
        e.preventDefault();
        view.columnNavigateUp(props.stack, lastIdx);
        emit("updateStack", [...props.stack]);
        // Sync selection after navigation
        syncSelection(lastIdx, props.stack[lastIdx]?.selectedIndex ?? -1);
    } else if (e.key === "ArrowDown") {
        e.preventDefault();
        view.columnNavigateDown(props.stack, lastIdx);
        emit("updateStack", [...props.stack]);
        syncSelection(lastIdx, props.stack[lastIdx]?.selectedIndex ?? -1);
    } else if (e.key === "ArrowRight") {
        e.preventDefault();
        if (selFile && selFile.is_dir) {
            syncSelection(lastIdx, col.selectedIndex);
            view.columnLoadDirectory(props.stack, lastIdx, selFile);
            emit("updateStack", [...props.stack]);
        }
    } else if (e.key === "Enter") {
        e.preventDefault();
        if (selFile) {
            syncSelection(lastIdx, col.selectedIndex);
            if (selFile.is_dir) {
                view.columnLoadDirectory(props.stack, lastIdx, selFile);
                emit("updateStack", [...props.stack]);
            } else {
                tauri
                    .openFile(selFile.path)
                    .catch((err) => console.error("Failed to open file:", err));
            }
        }
    } else if (e.key === "ArrowLeft" || e.key === "Backspace") {
        e.preventDefault();
        view.columnNavigateLeft(props.stack);
        emit("updateStack", [...props.stack]);
    }
}
</script>

<style scoped>
.column-container {
    flex: 1;
    display: flex;
    flex-direction: row;
    overflow-x: auto;
    overflow-y: hidden;
    outline: none;
}

.column-resize-handle {
    width: 6px;
    flex-shrink: 0;
    cursor: col-resize;
    position: relative;
    z-index: 1;
    background: transparent;
    transition: background 0.1s;
    margin-left: -1px;
    margin-right: -1px;
}

.column-resize-handle:hover,
.column-resize-handle:active {
    background: var(--accent);
    opacity: 0.5;
}

.column-filler {
    flex: 1;
    min-width: 20px;
}
</style>
