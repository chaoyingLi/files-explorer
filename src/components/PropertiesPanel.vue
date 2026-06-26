<template>
    <div class="properties-panel" v-if="visible">
        <div class="props-header">
            <span>{{ $t("contextMenu.properties") }}</span>
            <button class="props-close" @click="$emit('close')">✕</button>
        </div>
        <div class="props-body" v-if="file">
            <!-- File icon & name -->
            <div class="props-icon-row">
                <img
                    v-if="osIconSrc"
                    class="props-icon props-os-icon"
                    :src="osIconSrc"
                    alt=""
                />
                <div v-else class="props-icon" v-html="defaultFileIcon"></div>
                <div class="props-name">{{ file.name }}</div>
            </div>

            <div class="props-section">
                <div class="props-label">{{ $t("fileList.type") }}</div>
                <div class="props-value">{{ fileType }}</div>
            </div>
            <div class="props-section">
                <div class="props-label">{{ $t("fileList.size") }}</div>
                <div class="props-value">{{ formatSize(file.size) }}</div>
            </div>
            <div class="props-section" v-if="file.is_dir">
                <div class="props-label">{{ $t("properties.contents") }}</div>
                <div class="props-value">
                    {{ $t("properties.itemsCount", { count: dirItemCount }) }}
                </div>
            </div>
            <div class="props-section">
                <div class="props-label">{{ $t("fileList.dateModified") }}</div>
                <div class="props-value">{{ formatDate(file.modified) }}</div>
            </div>
            <div class="props-section">
                <div class="props-label">{{ $t("fileList.dateCreated") }}</div>
                <div class="props-value">{{ formatDate(file.created) }}</div>
            </div>
            <div class="props-section">
                <div class="props-label">{{ $t("properties.fullPath") }}</div>
                <div class="props-value props-path">{{ file.path }}</div>
            </div>

            <!-- Image dimensions -->
            <div class="props-section" v-if="imageInfo">
                <div class="props-label">{{ $t("properties.dimensions") }}</div>
                <div class="props-value">
                    {{ imageInfo.width }} × {{ imageInfo.height }}
                </div>
            </div>
        </div>

        <!-- Multi-selection summary -->
        <div class="props-body" v-else-if="multiCount > 1">
            <div class="props-icon-row">
                <div class="props-name">
                    {{ $t("properties.itemsSelected", { count: multiCount }) }}
                </div>
            </div>
            <div class="props-section">
                <div class="props-label">{{ $t("properties.totalSize") }}</div>
                <div class="props-value">{{ formatSize(totalSize) }}</div>
            </div>
        </div>

        <div class="props-body props-empty" v-else>
            <span>{{ $t("properties.noFileSelected") }}</span>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import { useSelectionStore } from "@/stores/selectionStore";
import { useViewStore } from "@/stores/viewStore";
import { useTabStore } from "@/stores/tabStore";
import type { FileEntry } from "@/types";
import { formatFileSize } from "@/utils/fileTypes";

const { t } = useI18n();
const store = useFileStore();
const sel = useSelectionStore();
const view = useViewStore();
const tabStore = useTabStore();

defineProps<{ visible: boolean }>();
defineEmits<{ close: [] }>();

const imageInfo = ref<{ width: number; height: number } | null>(null);
const dirItemCount = ref(0);
const osIconSrc = ref("");

/**
 * Find a FileEntry by path.
 * 1. Search store.files (works for details/list/grid/root tree nodes)
 * 2. Fall back to column stack files (works for column view)
 * 3. Fall back to tree children cache (works for tree view sub-nodes)
 */
function findFileByPath(path: string): FileEntry | null {
    // Primary: search store.files
    const f = store.files.find((x) => x.path === path);
    if (f) return f;
    // Fallback: search column stack (column view)
    if (view.viewMode === "column") {
        const tab = tabStore.getFocusedTab();
        if (tab?.columnStack) {
            for (const col of tab.columnStack) {
                const found = col.files.find((x) => x.path === path);
                if (found) return found;
            }
        }
    }
    // Fallback 2: search tree children cache (tree view)
    if (view.viewMode === "tree") {
        for (const children of view.treeChildrenCache.values()) {
            const found = children.find((x: FileEntry) => x.path === path);
            if (found) return found;
        }
    }
    return null;
}

const file = computed<FileEntry | null>(() => {
    if (sel.selectedFiles.size !== 1) return null;
    const path = [...sel.selectedFiles][0];
    return findFileByPath(path);
});

const multiCount = computed(() => sel.selectedFiles.size);

const totalSize = computed(() => {
    let total = 0;
    for (const p of sel.selectedFiles) {
        const f = findFileByPath(p);
        if (f) total += f.size;
    }
    return total;
});

watch(file, async (f) => {
    imageInfo.value = null;
    dirItemCount.value = 0;
    osIconSrc.value = "";
    if (!f) return;

    // Load OS native file icon (Windows SHGetFileInfoW)
    try {
        const { getFileIcon } = await import("@/utils/tauri");
        const b64 = await getFileIcon(f.path);
        osIconSrc.value = "data:image/png;base64," + b64;
    } catch {
        /* OS icon not available — will use default SVG fallback */
    }

    // Try to load image dimensions via base64
    const imgExts = ["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg", "ico"];
    if (imgExts.includes(f.extension.toLowerCase()) && !f.is_dir) {
        try {
            const { getFileBase64 } = await import("@/utils/tauri");
            const result = await getFileBase64(f.path);
            const img = new Image();
            img.onload = () => {
                imageInfo.value = {
                    width: img.naturalWidth,
                    height: img.naturalHeight,
                };
            };
            img.src = `data:${result.mime};base64,${result.data}`;
        } catch {}
    }

    // Count dir items
    if (f.is_dir) {
        try {
            const { listDirectory } = await import("@/utils/tauri");
            const items = await listDirectory(f.path);
            dirItemCount.value = items.length;
        } catch {}
    }
});

const defaultFileIcon = `<svg viewBox="0 0 24 24"><path d="M6.5 2.5h6.8l5.2 5.2V19.5a2 2 0 01-2 2H6.5a2 2 0 01-2-2V4.5a2 2 0 012-2z" fill="currentColor" opacity="0.3"/><path d="M13.3 2.5v4.2c0 .55.45 1 1 1H18" fill="currentColor" opacity="0.5"/></svg>`;

const fileType = computed(() => {
    if (!file.value) return "";
    if (file.value.is_dir) return t("fileTypes.folder");
    const ext = file.value.extension.toUpperCase();
    return ext ? t("properties.fileType", { ext }) : t("fileTypes.file");
});

function formatSize(bytes: number): string {
    return formatFileSize(bytes);
}

function formatDate(ts: number): string {
    if (!ts) return "";
    const d = new Date(ts * 1000);
    return (
        d.toLocaleDateString() +
        " " +
        d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })
    );
}
</script>

<style scoped>
.properties-panel {
    width: 260px;
    flex-shrink: 0;
    background: var(--bg-secondary);
    border-left: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    font-size: var(--font-size-base);
}
.props-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    font-weight: 600;
    color: var(--text-primary);
    border-bottom: 1px solid var(--border);
}
.props-close {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 14px;
    padding: 2px 6px;
    border-radius: 4px;
}
.props-close:hover {
    background: var(--bg-hover);
}
.props-body {
    flex: 1;
    overflow-y: auto;
    padding: 12px;
}
.props-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
}
.props-icon-row {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 16px;
}
.props-icon {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
}
.props-icon :deep(svg) {
    width: 32px;
    height: 32px;
}
.props-name {
    font-weight: 500;
    word-break: break-all;
}
.props-section {
    margin-bottom: 12px;
}
.props-label {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    margin-bottom: 2px;
}
.props-value {
    color: var(--text-primary);
}
.props-path {
    font-size: var(--font-size-xs);
    word-break: break-all;
    color: var(--text-secondary);
}
</style>
