<template>
    <div class="preview-panel" v-if="visible" :style="{ width: width + 'px' }">
        <div class="preview-resize-handle" @mousedown.stop="onResizeStart" />
        <div class="preview-header">
            <span>{{ $t("properties.title") }}</span>
            <div class="preview-header-actions">
                <button
                    v-if="file"
                    class="preview-popout"
                    :title="$t('properties.popOut')"
                    @click="openPreviewWindow"
                >
                    ↗
                </button>
                <button class="preview-close" @click="$emit('close')">✕</button>
            </div>
        </div>

        <!-- Empty / multi-select state -->
        <div v-if="!file && multiCount <= 1" class="preview-empty">
            <span>{{ $t("properties.noFileSelected") }}</span>
        </div>
        <div v-else-if="!file && multiCount > 1" class="preview-multi">
            <span class="preview-multi-text">{{
                $t("properties.itemsSelected", { count: multiCount })
            }}</span>
            <span class="preview-meta"
                >{{ $t("properties.totalSize") }}:
                {{ formatSize(totalSize) }}</span
            >
        </div>

        <template v-else-if="file">
            <!-- ════ Preview area (top, fills available space) ════ -->
            <div class="preview-area">
                <!-- Loading -->
                <div v-if="previewLoading" class="preview-status">
                    <span class="preview-spinner"></span>
                    <span>{{ $t("properties.previewLoading") }}</span>
                </div>
                <!-- Error -->
                <div
                    v-else-if="previewError"
                    class="preview-status preview-status-error"
                >
                    <span>{{ previewError }}</span>
                    <button class="preview-btn" @click="openFileExternally">
                        {{ $t("properties.openExternally") }}
                    </button>
                </div>
                <!-- Image -->
                <div
                    v-else-if="previewType === 'image'"
                    class="preview-image-wrap"
                >
                    <img
                        class="preview-image"
                        :src="previewSrc"
                        alt=""
                        @click.stop
                    />
                </div>
                <!-- DOCX -->
                <div v-else-if="previewType === 'docx'" class="preview-office">
                    <VueOfficeDocx
                        v-if="officeData"
                        :src="officeData"
                        style="height: 100%"
                    />
                </div>
                <!-- XLSX -->
                <div v-else-if="previewType === 'xlsx'" class="preview-office">
                    <VueOfficeExcel
                        v-if="officeData"
                        :src="officeData"
                        style="height: 100%"
                    />
                </div>
                <!-- PDF -->
                <div v-else-if="previewType === 'pdf'" class="preview-office">
                    <VueOfficePdf
                        v-if="officeData"
                        :src="officeData"
                        style="height: 100%"
                    />
                </div>
                <!-- PPTX -->
                <div v-else-if="previewType === 'pptx'" class="preview-office">
                    <PptxPreview v-if="officeData" :data="officeData" />
                </div>
                <!-- Archive listing -->
                <div
                    v-else-if="previewType === 'archive'"
                    class="preview-archive"
                >
                    <div class="preview-archive-header">
                        {{ archiveTotal }} entries
                    </div>
                    <div class="preview-archive-list">
                        <div
                            v-for="entry in archiveEntries"
                            :key="entry.path"
                            class="preview-archive-item"
                            :class="{
                                'preview-archive-item--dir': entry.is_dir,
                            }"
                            :style="{
                                paddingLeft:
                                    8 +
                                    entry.path.split('/').length * 14 +
                                    'px',
                            }"
                        >
                            <span class="preview-archive-icon">{{
                                entry.is_dir ? "📁" : "📄"
                            }}</span>
                            <span class="preview-archive-name">{{
                                entry.name
                            }}</span>
                            <span
                                v-if="!entry.is_dir"
                                class="preview-archive-size"
                                >{{ formatSize(entry.size) }}</span
                            >
                        </div>
                    </div>
                </div>

                <!-- Code / Text -->
                <div v-else-if="previewType === 'text'" class="preview-code">
                    <CodePreview :code="previewContent" :ext="previewExt" />
                </div>
                <!-- Markdown -->
                <div
                    v-else-if="previewType === 'markdown'"
                    class="preview-markdown"
                    v-html="renderedMarkdown"
                ></div>
            </div>

            <!-- ════ File info (bottom, compact) ════ -->
            <div class="preview-info">
                <div class="preview-info-header">
                    <img
                        v-if="osIconSrc"
                        class="preview-info-icon"
                        :src="osIconSrc"
                        alt=""
                    />
                    <div
                        v-else
                        class="preview-info-icon"
                        v-html="defaultFileIcon"
                    ></div>
                    <span class="preview-info-name">{{ file.name }}</span>
                </div>

                <div class="preview-info-grid">
                    <div class="preview-info-cell">
                        <span class="preview-info-label">{{
                            $t("fileList.type")
                        }}</span>
                        <span class="preview-info-value">{{ fileType }}</span>
                    </div>
                    <div class="preview-info-cell">
                        <span class="preview-info-label">{{
                            $t("fileList.size")
                        }}</span>
                        <span class="preview-info-value">{{
                            formatSize(file.size)
                        }}</span>
                    </div>
                    <div v-if="imageInfo" class="preview-info-cell">
                        <span class="preview-info-label">{{
                            $t("properties.dimensions")
                        }}</span>
                        <span class="preview-info-value"
                            >{{ imageInfo.width }} ×
                            {{ imageInfo.height }}</span
                        >
                    </div>
                    <div v-else-if="file.is_dir" class="preview-info-cell">
                        <span class="preview-info-label">{{
                            $t("properties.contents")
                        }}</span>
                        <span class="preview-info-value">{{
                            $t("properties.itemsCount", { count: dirItemCount })
                        }}</span>
                    </div>
                </div>

                <div class="preview-info-row">
                    <span class="preview-info-label">{{
                        $t("fileList.dateModified")
                    }}</span>
                    <span class="preview-info-value">{{
                        formatDate(file.modified)
                    }}</span>
                </div>
                <div class="preview-info-row">
                    <span class="preview-info-label">{{
                        $t("fileList.dateCreated")
                    }}</span>
                    <span class="preview-info-value">{{
                        formatDate(file.created)
                    }}</span>
                </div>
                <div class="preview-info-row preview-info-path">
                    <span class="preview-info-label">{{
                        $t("properties.fullPath")
                    }}</span>
                    <span class="preview-info-value">{{ file.path }}</span>
                </div>

                <div class="preview-info-actions">
                    <button class="preview-btn" @click="openFileExternally">
                        {{ $t("properties.openExternally") }}
                    </button>
                </div>
            </div>
        </template>
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
import type { ArchiveEntry } from "@/utils/tauri";
import { formatFileSize } from "@/utils/fileTypes";
import {
    getFileIcon,
    listDirectory,
    getFilePreview,
    readFileBytes,
    listArchiveContents,
    openFile,
} from "@/utils/tauri";
import { marked } from "marked";
import DOMPurify from "dompurify";
import VueOfficeDocx from "@vue-office/docx";
import VueOfficeExcel from "@vue-office/excel";
import VueOfficePdf from "@vue-office/pdf";
import PptxPreview from "@/components/PptxPreview.vue";
import CodePreview from "@/components/CodePreview.vue";
import "@vue-office/docx/lib/index.css";
import "@vue-office/excel/lib/index.css";
import { convertFileSrc } from "@tauri-apps/api/core";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

const IMG_EXTS = ["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg", "ico"];
const ARCHIVE_EXTS = [
    "zip",
    "7z",
    "rar",
    "tar",
    "gz",
    "tgz",
    "bz2",
    "tbz2",
    "xz",
    "txz",
];

// ── Configure marked ──
marked.setOptions({
    gfm: true,
    breaks: true,
});

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
const previewExt = ref<string>("");
const renderedMarkdown = ref("");
const previewLoading = ref(false);
const previewError = ref("");
const officeData = ref<ArrayBuffer | null>(null);
const archiveEntries = ref<ArchiveEntry[]>([]);
const archiveTotal = ref(0);

const OFFICE_EXTS: Record<string, string> = {
    docx: "docx",
    xlsx: "xlsx",
    pptx: "pptx",
};

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
    previewType.value = "";
    previewSrc.value = "";
    previewContent.value = "";
    previewExt.value = "";
    previewLoading.value = false;
    previewError.value = "";
    officeData.value = null;
    archiveEntries.value = [];
    archiveTotal.value = 0;
    if (!f) return;

    // ── Load OS file icon ──
    try {
        const b64 = await getFileIcon(f.path);
        osIconSrc.value = "data:image/png;base64," + b64;
    } catch {
        /* OS icon not available */
    }

    // ── Image: use native asset protocol (no base64, no size limit) ──
    if (IMG_EXTS.includes(f.extension.toLowerCase()) && !f.is_dir) {
        const assetUrl = convertFileSrc(f.path);
        // Get image dimensions from browser-native loading
        const img = new Image();
        img.onload = () => {
            imageInfo.value = {
                width: img.naturalWidth,
                height: img.naturalHeight,
            };
        };
        img.onerror = () => {
            imageInfo.value = null;
        };
        img.src = assetUrl;
    }

    // ── Directory item count ──
    if (f.is_dir) {
        try {
            const items = await listDirectory(f.path);
            dirItemCount.value = items.length;
        } catch {
            /* directory read failed */
        }
    }

    // ── Load file preview ──
    const ext = f.extension.toLowerCase();

    if (IMG_EXTS.includes(ext) && !f.is_dir) {
        // Image preview — use native asset protocol, no size limit
        previewType.value = "image";
        previewSrc.value = convertFileSrc(f.path);
    } else if (OFFICE_EXTS[ext] && !f.is_dir) {
        // Office documents: docx / xlsx / pptx — read raw bytes
        previewLoading.value = true;
        try {
            const buf = await loadFileAsArrayBuffer(f.path);
            officeData.value = buf;
            previewType.value = OFFICE_EXTS[ext];
            previewLoading.value = false;
        } catch (e: any) {
            previewError.value = String(e || "Preview unavailable");
            previewLoading.value = false;
        }
    } else if (ext === "pdf" && !f.is_dir) {
        // PDF — use @vue-office/pdf (read raw bytes)
        previewLoading.value = true;
        try {
            const buf = await loadFileAsArrayBuffer(f.path);
            officeData.value = buf;
            previewType.value = "pdf";
            previewLoading.value = false;
        } catch (e: any) {
            previewError.value = String(e || "Preview unavailable");
            previewLoading.value = false;
        }
    } else if (!f.is_dir && ARCHIVE_EXTS.includes(ext)) {
        // Archive preview: list contents
        previewLoading.value = true;
        try {
            const entries = await listArchiveContents(f.path);
            archiveEntries.value = entries;
            archiveTotal.value = entries.length;
            previewType.value = "archive";
        } catch (e: any) {
            previewError.value = String(e || "Preview unavailable");
        } finally {
            previewLoading.value = false;
        }
    } else if (!f.is_dir) {
        // Text / Markdown preview via Rust
        previewLoading.value = true;
        try {
            const result = await getFilePreview(f.path);
            if (result.type === "markdown") {
                previewType.value = "markdown";
                const content = result.content || "";
                const rawHtml = await marked.parse(content);
                renderedMarkdown.value = DOMPurify.sanitize(rawHtml);
            } else if (result.type === "text") {
                previewType.value = "text";
                previewContent.value = result.content || "";
                previewExt.value = result.ext || f.extension;
            }
        } catch (e: any) {
            previewError.value = String(e || "Preview unavailable");
        } finally {
            previewLoading.value = false;
        }
    }
});

/**
 * Read a file as ArrayBuffer via Rust backend, with chunked base64 decode
 * to avoid blocking the main thread on large files.
 */
async function loadFileAsArrayBuffer(path: string): Promise<ArrayBuffer> {
    const b64 = await readFileBytes(path);
    // Decode base64 in chunks to yield to the main thread every 1MB
    const CHUNK = 1024 * 1024; // 1MB of base64 chars per chunk
    const binaryChunks: string[] = [];

    for (let i = 0; i < b64.length; i += CHUNK) {
        binaryChunks.push(atob(b64.slice(i, i + CHUNK)));
        // Yield to the main thread to keep UI responsive
        await new Promise((r) => setTimeout(r, 0));
    }

    const binary = binaryChunks.join("");
    const buf = new ArrayBuffer(binary.length);
    const view = new Uint8Array(buf);
    // Fill in chunks to avoid long synchronous loop
    const FILL_CHUNK = 256 * 1024; // 256KB per chunk
    for (let i = 0; i < binary.length; i += FILL_CHUNK) {
        const end = Math.min(i + FILL_CHUNK, binary.length);
        for (let j = i; j < end; j++) {
            view[j] = binary.charCodeAt(j);
        }
        if (i + FILL_CHUNK < binary.length) {
            await new Promise((r) => setTimeout(r, 0));
        }
    }
    return buf;
}

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

async function openFileExternally() {
    if (file.value) {
        try {
            await openFile(file.value.path);
        } catch {
            /* ignore */
        }
    }
}

async function openPreviewWindow() {
    if (!file.value) return;
    const label = `preview-${file.value.path.replace(/[^a-zA-Z0-9]/g, "_")}`;
    try {
        const existing = await WebviewWindow.getByLabel(label);
        if (existing) {
            await existing.setFocus();
            return;
        }
    } catch {
        /* window doesn't exist yet, proceed to create */
    }
    try {
        const win = new WebviewWindow(label, {
            url: `/?preview=${encodeURIComponent(file.value.path)}`,
            title: file.value.name,
            width: 960,
            height: 680,
            minWidth: 640,
            minHeight: 400,
            decorations: true,
            resizable: true,
            center: true,
        });
        // Wait for window to be created before proceeding
        await win.once("tauri://created", () => {
            console.log("Preview window created:", label);
        });
        await win.once("tauri://error", (e: any) => {
            console.error("Preview window error:", e);
        });
    } catch (e) {
        console.error("Failed to open preview window:", e);
    }
}
</script>

<style scoped>
/* ── Panel container ── */
.preview-panel {
    position: relative;
    flex-shrink: 0;
    background: var(--bg-secondary);
    border-left: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    font-size: var(--font-size-base);
}
.preview-resize-handle {
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
.preview-resize-handle:hover,
.preview-resize-handle:active {
    background: var(--accent);
    opacity: 0.7;
}

/* ── Header ── */
.preview-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    font-weight: 600;
    color: var(--text-primary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
}
.preview-header-actions {
    display: flex;
    align-items: center;
    gap: 4px;
}
.preview-popout {
    background: none;
    border: 1px solid transparent;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 16px;
    padding: 2px 6px;
    border-radius: 4px;
    line-height: 1;
    transition: all 0.15s;
}
.preview-popout:hover {
    background: var(--bg-hover);
    color: var(--accent);
    border-color: var(--border);
}
.preview-close {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 14px;
    padding: 2px 6px;
    border-radius: 4px;
}
.preview-close:hover {
    background: var(--bg-hover);
}

/* ── Empty / multi-select ── */
.preview-empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    font-size: 13px;
}
.preview-multi {
    padding: 16px 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
}
.preview-multi-text {
    font-weight: 600;
    color: var(--text-primary);
    font-size: 13px;
}
.preview-meta {
    color: var(--text-muted);
    font-size: 12px;
}

/* ── Preview area (top, flex-fill) ── */
.preview-area {
    flex: 1;
    overflow: auto;
    min-height: 0;
    padding: 4px;
}

/* ── Status (loading / error) ── */
.preview-status {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 16px 12px;
    color: var(--text-muted);
    font-size: 12px;
}
.preview-status-error {
    color: var(--danger);
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
}
.preview-spinner {
    width: 14px;
    height: 14px;
    border: 2px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: preview-spin 0.6s linear infinite;
}
@keyframes preview-spin {
    to {
        transform: rotate(360deg);
    }
}

/* ── Preview types ── */
.preview-image-wrap {
    height: 100%;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding: 8px;
}
.preview-image {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    border-radius: 4px;
    cursor: pointer;
}
.preview-office {
    height: 100%;
    overflow: auto;
}
.preview-code {
    height: 100%;
    overflow: auto;
}
.preview-markdown {
    padding: 8px 12px;
    font-size: 12px;
    line-height: 1.6;
    overflow: auto;
    color: var(--text-primary);
    height: 100%;
}

/* ── Info section (bottom, compact) ── */
.preview-info {
    flex-shrink: 0;
    border-top: 1px solid var(--border);
    padding: 10px 12px;
    background: var(--bg-primary);
}
.preview-info-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
}
.preview-info-icon {
    width: 22px;
    height: 22px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
}
.preview-info-icon :deep(svg) {
    width: 22px;
    height: 22px;
}
.preview-info-name {
    font-weight: 600;
    font-size: 13px;
    color: var(--text-primary);
    word-break: break-all;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.preview-info-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4px 12px;
    margin-bottom: 4px;
}
.preview-info-cell {
    display: flex;
    flex-direction: column;
    gap: 1px;
}
.preview-info-row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    padding: 2px 0;
}
.preview-info-path {
    flex-direction: column;
}
.preview-info-label {
    font-size: 10px;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.3px;
    flex-shrink: 0;
    margin-right: 8px;
}
.preview-info-value {
    font-size: 11px;
    color: var(--text-secondary);
    word-break: break-all;
}
.preview-info-actions {
    margin-top: 8px;
    display: flex;
    gap: 6px;
}
.preview-btn {
    background: var(--bg-hover);
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: 3px 10px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
    transition:
        background 0.15s,
        color 0.15s;
}
.preview-btn:hover {
    background: var(--accent);
    color: #fff;
}

/* ── Markdown content styles ── */
.preview-markdown h1,
.preview-markdown h2,
.preview-markdown h3,
.preview-markdown h4 {
    margin: 8px 0 4px;
    font-weight: 600;
}
.preview-markdown h1 {
    font-size: 15px;
}
.preview-markdown h2 {
    font-size: 14px;
}
.preview-markdown h3 {
    font-size: 13px;
}
.preview-markdown p {
    margin: 4px 0;
}
.preview-markdown ul,
.preview-markdown ol {
    padding-left: 20px;
    margin: 4px 0;
}
.preview-markdown code {
    background: var(--input-bg);
    padding: 1px 4px;
    border-radius: 3px;
    font-size: 11px;
}
.preview-markdown pre {
    background: var(--input-bg);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 8px;
    overflow: auto;
}
.preview-markdown pre code {
    background: none;
    padding: 0;
}
.preview-markdown table {
    border-collapse: collapse;
    width: 100%;
    margin: 8px 0;
}
.preview-markdown th,
.preview-markdown td {
    border: 1px solid var(--border);
    padding: 4px 8px;
    text-align: left;
}
.preview-markdown th {
    background: var(--bg-secondary);
    font-weight: 600;
}
.preview-markdown blockquote {
    border-left: 3px solid var(--accent);
    padding-left: 8px;
    margin: 4px 0;
    color: var(--text-secondary);
}
.preview-markdown img {
    max-width: 100%;
    border-radius: 4px;
}
.preview-markdown hr {
    border: none;
    border-top: 1px solid var(--border);
    margin: 8px 0;
}
/* ── Archive listing ── */
.preview-archive {
    height: 100%;
    display: flex;
    flex-direction: column;
}
.preview-archive-header {
    padding: 6px 8px;
    font-size: 11px;
    color: var(--text-muted);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
}
.preview-archive-list {
    flex: 1;
    overflow: auto;
    padding: 2px 0;
}
.preview-archive-item {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px 6px;
    font-size: 11px;
    white-space: nowrap;
}
.preview-archive-icon {
    flex-shrink: 0;
    font-size: 12px;
}
.preview-archive-name {
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
}
.preview-archive-size {
    margin-left: auto;
    font-size: 10px;
    color: var(--text-muted);
    flex-shrink: 0;
}
</style>
