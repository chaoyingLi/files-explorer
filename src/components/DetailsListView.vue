<template>
    <div
        ref="containerRef"
        class="details-list-virtual"
        :style="{ height: '100%', overflow: 'auto' }"
    >
        <div
            :style="{
                height: virtualizer.getTotalSize() + 'px',
                width: '100%',
                position: 'relative',
            }"
        >
            <div
                v-for="row in virtualizer.getVirtualItems()"
                :key="row.index"
                :data-index="row.index"
                :ref="(el) => measureEl(el as HTMLElement)"
                :style="{
                    position: 'absolute',
                    top: 0,
                    left: 0,
                    width: '100%',
                    transform: `translateY(${row.start}px)`,
                }"
            >
                <FileItem
                    :file="props.files[row.index]"
                    :compact="compact"
                    :selected="isSelected(props.files[row.index].path)"
                    :is-cut="sel.isFileCut(props.files[row.index].path)"
                    :show-path="showPath"
                    @click="
                        (e: MouseEvent) =>
                            $emit('fileClick', props.files[row.index], e)
                    "
                    @dblclick="
                        (e: MouseEvent) =>
                            $emit('fileDblClick', props.files[row.index], e)
                    "
                    @contextmenu="
                        (e: MouseEvent) =>
                            $emit('fileContextMenu', props.files[row.index], e)
                    "
                />
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useVirtualizer } from "@tanstack/vue-virtual";
import { useFileStore } from "@/stores/fileStore";
import { useSelectionStore } from "@/stores/selectionStore";
import type { FileEntry } from "@/types";
import FileItem from "@/components/FileItem.vue";

const props = defineProps<{
    files: FileEntry[];
    compact: boolean;
    showPath: boolean;
}>();

defineEmits<{
    fileClick: [file: FileEntry, e: MouseEvent];
    fileDblClick: [file: FileEntry, e: MouseEvent];
    fileContextMenu: [file: FileEntry, e: MouseEvent];
}>();

const store = useFileStore();
const sel = useSelectionStore();

const containerRef = ref<HTMLElement | null>(null);
const rowHeight = computed(() => (props.compact ? 28 : 32));

const virtualizer = useVirtualizer(
    computed(() => ({
        count: props.files.length,
        getScrollElement: () => containerRef.value,
        estimateSize: () => rowHeight.value,
        overscan: 10,
    })),
);

function measureEl(el: HTMLElement) {
    virtualizer.value.measureElement(el);
}

function isSelected(path: string): boolean {
    return sel.selectedFiles.has(path);
}
</script>
