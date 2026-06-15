<template>
    <template v-if="compact">
        <FileItem
            v-for="file in files"
            :key="file.path"
            :file="file"
            :compact="true"
            :selected="isSelected(file.path)"
            :is-cut="store.isFileCut(file.path)"
            :show-path="showPath"
            @click="(e: MouseEvent) => $emit('fileClick', file, e)"
            @dblclick="(e: MouseEvent) => $emit('fileDblClick', file, e)"
            @contextmenu="(e: MouseEvent) => $emit('fileContextMenu', file, e)"
        />
    </template>
    <template v-else>
        <FileItem
            v-for="file in files"
            :key="file.path"
            :file="file"
            :compact="false"
            :selected="isSelected(file.path)"
            :is-cut="store.isFileCut(file.path)"
            :show-path="showPath"
            @click="(e: MouseEvent) => $emit('fileClick', file, e)"
            @dblclick="(e: MouseEvent) => $emit('fileDblClick', file, e)"
            @contextmenu="(e: MouseEvent) => $emit('fileContextMenu', file, e)"
        />
    </template>
</template>

<script setup lang="ts">
import { useFileStore } from "@/stores/fileStore";
import type { FileEntry } from "@/types";
import FileItem from "@/components/FileItem.vue";

defineProps<{
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

function isSelected(path: string): boolean {
    return store.selectedFiles.has(path);
}
</script>
