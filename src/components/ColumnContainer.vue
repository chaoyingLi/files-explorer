<template>
    <div
        class="column-container"
        ref="scrollRef"
        tabindex="0"
        @keydown="onKeydown"
    >
        <ColumnPane
            v-for="(col, idx) in stack"
            :key="idx"
            :column="col"
            :col-idx="idx"
            @select="onSelect"
            @dblclick="onDblClick"
            @contextmenu="onContextMenu"
        />
        <div class="column-filler" />
    </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from "vue";
import type { FileEntry } from "@/types";
import type { ColumnState } from "@/stores/viewStore";
import { useFileStore } from "@/stores/fileStore";
import { useViewStore } from "@/stores/viewStore";
import { useSelectionStore } from "@/stores/selectionStore";
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

async function onSelect(colIdx: number, fileIdx: number) {
    const file = props.stack[colIdx]?.files[fileIdx];
    if (!file) return;
    await view.columnLoadDirectory(props.stack, colIdx, file);
    emit("updateStack", [...props.stack]);
    await nextTick();
    if (scrollRef.value)
        scrollRef.value.scrollLeft = scrollRef.value.scrollWidth;
}

function onDblClick(colIdx: number, fileIdx: number) {
    const file = props.stack[colIdx]?.files[fileIdx];
    if (!file) return;
    view.columnLoadDirectory(props.stack, colIdx, file);
    emit("updateStack", [...props.stack]);
}

function onContextMenu(file: FileEntry, event: MouseEvent) {
    sel.selectFile(file, event.ctrlKey || event.metaKey);
    emit("contextMenu", file, event);
}

function onKeydown(e: KeyboardEvent) {
    if (e.target instanceof HTMLInputElement) return;
    const lastIdx = props.stack.length - 1;
    if (e.key === "ArrowUp") {
        e.preventDefault();
        view.columnNavigateUp(props.stack, lastIdx);
        emit("updateStack", [...props.stack]);
    } else if (e.key === "ArrowDown") {
        e.preventDefault();
        view.columnNavigateDown(props.stack, lastIdx);
        emit("updateStack", [...props.stack]);
    } else if (e.key === "ArrowRight" || e.key === "Enter") {
        e.preventDefault();
        const col = props.stack[lastIdx];
        if (col && col.selectedIndex >= 0) {
            view.columnLoadDirectory(
                props.stack,
                lastIdx,
                col.files[col.selectedIndex],
            );
            emit("updateStack", [...props.stack]);
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
.column-filler {
    flex: 1;
    min-width: 20px;
}
</style>
