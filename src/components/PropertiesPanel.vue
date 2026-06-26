<template>
    <div
        class="properties-panel"
        v-if="visible"
        :style="{ width: width + 'px' }"
    >
        <!-- Resize handle -->
        <div class="props-resize-handle" @mousedown.stop="onResizeStart" />
        <div class="props-header">
            <span>{{ $t("contextMenu.properties") }}</span>
            <button class="props-close" @click="$emit('close')">✕</button>
        </div>
        <div class="props-body" v-if="file">
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

            <!-- File preview -->
            <div v-if="previewType === 'image'" class="props-preview-wrap">
                <img
                    class="props-preview"
                    :src="previewSrc"
                    alt=""
                    @click.stop
                />
            </div>
            <div v-else-if="previewType === 'pdf'" class="props-preview-wrap">
                <embed
                    class="props-preview props-preview-pdf"
                    :src="previewSrc"
                    type="application/pdf"
                />
            </div>
            <div v-else-if="previewType === 'text'" class="props-preview-wrap">
                <pre
                    class="props-text-preview"
                ><code v-html="previewContent"></code></pre>
            </div>
            <div
                v-else-if="previewType === 'markdown'"
                class="props-preview-wrap"
            >
                <div
                    class="props-markdown-preview"
                    v-html="renderedMarkdown"
                ></div>
            </div>

            <div class="props-section" v-if="imageInfo">
                <div class="props-label">{{ $t("properties.dimensions") }}</div>
                <div class="props-value">
                    {{ imageInfo.width }} × {{ imageInfo.height }}
                </div>
            </div>
        </div>

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
import { marked } from "marked";
import hljs from "highlight.js";

const { t } = useI18n();
const store = useFileStore();
const sel = useSelectionStore();
const view = useViewStore();
const tabStore = useTabStore();

defineProps<{ visible: boolean; width: number }>();
const emit = defineEmits<{ close: []; resizeStart: [e: MouseEvent] }>();

const imageInfo = ref<{ width: number; height: number } | null>(null);
const dirItemCount = ref(0);
const osIconSrc = ref("");
const previewType = ref<string>("");
const previewSrc = ref<string>("");
const previewContent = ref<string>("");
const renderedMarkdown = ref("");

function findFileByPath(path: string): FileEntry | null {
    const f = store.files.find((x) => x.path === path);
    if (f) return f;
    if (view.viewMode === "column") {
        const tab = tabStore.getFocusedTab();
        if (tab?.columnStack) {
            for (const col of tab.columnStack) {
                const found = col.files.find((x) => x.path === path);
                if (found) return found;
            }
        }
    }
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

    try {
        const { getFileIcon } = await import("@/utils/tauri");
        const b64 = await getFileIcon(f.path);
        osIconSrc.value = "data:image/png;base64," + b64;
    } catch {
        /* OS icon not available */
    }

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

    if (f.is_dir) {
        try {
            const { listDirectory } = await import("@/utils/tauri");
            const items = await listDirectory(f.path);
            dirItemCount.value = items.length;
        } catch {}
    }

    // ── Load file preview ──
    previewType.value = "";
    previewSrc.value = "";
    previewContent.value = "";

    const ext = f.extension.toLowerCase();

    // Image preview — reuse the existing imgExts from above
    if (imgExts.includes(ext) && !f.is_dir) {
        // Image preview — reuse existing get_file_base64
        try {
            const { getFileBase64 } = await import("@/utils/tauri");
            const result = await getFileBase64(f.path);
            const maxBytes = 500 * 1024;
            if (result.data.length * 0.75 < maxBytes) {
                previewType.value = "image";
                previewSrc.value = `data:${result.mime};base64,${result.data}`;
            }
        } catch {}
    } else if (!f.is_dir) {
        // Text / PDF / docx preview via Rust
        try {
            const { getFilePreview } = await import("@/utils/tauri");
            const result = await getFilePreview(f.path);
            if (result.type === "markdown") {
                previewType.value = "markdown";
                previewContent.value = result.content || "";
                renderedMarkdown.value = await marked.parse(
                    previewContent.value,
                );
            } else if (result.type === "text") {
                previewType.value = "text";
                const code = result.content || "";
                try {
                    const highlighted = hljs.highlightAuto(code);
                    previewContent.value = highlighted.value;
                } catch {
                    previewContent.value = code;
                }
            } else if (result.type === "pdf") {
                previewType.value = "pdf";
                previewSrc.value = `data:application/pdf;base64,${result.data}`;
            }
        } catch {
            /* preview not available */
        }
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

function onResizeStart(e: MouseEvent) {
    emit("resizeStart", e);
}
</script>

<style scoped>
.properties-panel {
    position: relative;
    flex-shrink: 0;
    background: var(--bg-secondary);
    border-left: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    font-size: var(--font-size-base);
}
.props-resize-handle {
    position: absolute;
    top: 0;
    left: -3px;
    width: 6px;
    height: 100%;
    cursor: col-resize;
    z-index: 2;
    background: transparent;
    transition: background 0.1s;
}
.props-resize-handle:hover,
.props-resize-handle:active {
    background: var(--accent);
    opacity: 0.7;
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
.props-os-icon {
    object-fit: contain;
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
.props-preview-wrap {
    border-top: 1px solid var(--border);
    padding: 8px 0;
    margin-top: 8px;
}
.props-preview {
    width: 100%;
    max-height: 300px;
    object-fit: contain;
    border-radius: 4px;
    cursor: pointer;
}
.props-preview-pdf {
    height: 400px;
    border: none;
}
.props-text-preview {
    margin: 0;
    padding: 8px;
    background: var(--input-bg);
    border: 1px solid var(--border);
    border-radius: 4px;
    font-size: 11px;
    line-height: 1.5;
    max-height: 300px;
    overflow: auto;
    white-space: pre-wrap;
    word-break: break-all;
    color: var(--text-primary);
}
.props-markdown-preview {
    padding: 8px;
    font-size: 12px;
    line-height: 1.6;
    max-height: 350px;
    overflow: auto;
    color: var(--text-primary);
}
.props-markdown-preview h1,
.props-markdown-preview h2,
.props-markdown-preview h3,
.props-markdown-preview h4 {
    margin: 8px 0 4px;
    font-weight: 600;
}
.props-markdown-preview h1 {
    font-size: 15px;
}
.props-markdown-preview h2 {
    font-size: 14px;
}
.props-markdown-preview h3 {
    font-size: 13px;
}
.props-markdown-preview p {
    margin: 4px 0;
}
.props-markdown-preview ul,
.props-markdown-preview ol {
    padding-left: 20px;
    margin: 4px 0;
}
.props-markdown-preview code {
    background: var(--input-bg);
    padding: 1px 4px;
    border-radius: 3px;
    font-size: 11px;
}
.props-markdown-preview pre {
    background: var(--input-bg);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 8px;
    overflow: auto;
}
.props-markdown-preview pre code {
    background: none;
    padding: 0;
}
.props-markdown-preview table {
    border-collapse: collapse;
    width: 100%;
    margin: 8px 0;
}
.props-markdown-preview th,
.props-markdown-preview td {
    border: 1px solid var(--border);
    padding: 4px 8px;
    text-align: left;
}
.props-markdown-preview th {
    background: var(--bg-secondary);
    font-weight: 600;
}
.props-markdown-preview blockquote {
    border-left: 3px solid var(--accent);
    padding-left: 8px;
    margin: 4px 0;
    color: var(--text-secondary);
}
.props-markdown-preview img {
    max-width: 100%;
    border-radius: 4px;
}
.props-markdown-preview hr {
    border: none;
    border-top: 1px solid var(--border);
    margin: 8px 0;
}
</style>
