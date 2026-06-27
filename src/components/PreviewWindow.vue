<template>
    <div class="pw-root">
        <div class="pw-header">
            <span class="pw-title">{{ fileName || "Preview" }}</span>
            <button class="pw-close" @click="closeWindow">✕</button>
        </div>
        <!-- Back to archive -->
        <div v-if="viewingExtracted" class="pw-back-bar">
            <button class="pw-back-btn" @click="backToArchive">
                ← {{ archiveFileName }}
            </button>
        </div>
        <div class="pw-body">
            <!-- Left: tree -->
            <div class="pw-tree" :style="{ width: treeWidth + 'px' }">
                <div class="pw-tree-nav">
                    <button
                        class="pw-nav-btn"
                        :disabled="!canGoUp"
                        @click="goUp"
                    >
                        ← {{ $t("toolbar.up") }}
                    </button>
                </div>
                <div class="pw-tree-list" @contextmenu.prevent="onTreeCtxMenu">
                    <template v-for="node in flatTree" :key="node.path">
                        <div
                            class="pw-tree-item"
                            :class="{
                                'pw-tree-item--active':
                                    node.path === activePath,
                                'pw-tree-item--dir': node.isDir,
                            }"
                            :style="{ paddingLeft: 8 + node.depth * 18 + 'px' }"
                            @click="onTreeClick(node)"
                            @dblclick="onTreeDblClick(node)"
                            @contextmenu.stop="onItemCtxMenu(node, $event)"
                        >
                            <!-- Chevron -->
                            <span
                                class="pw-tree-chevron"
                                :class="{
                                    'pw-tree-chevron--expanded':
                                        node.isDir && node.expanded,
                                    'pw-tree-chevron--hidden':
                                        !node.isDir || !node.hasChildren,
                                }"
                                @click.stop="toggleExpand(node)"
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
                            <!-- Icon -->
                            <span
                                class="pw-tree-icon-wrap"
                                :class="node.isDir ? '' : fileColorClass(node)"
                            >
                                <div
                                    v-if="!node.isDir || isBundle(node)"
                                    class="pw-tree-icon-svg"
                                    v-html="fileIcon(node)"
                                ></div>
                                <svg
                                    v-else
                                    class="pw-tree-icon-svg"
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
                            </span>
                            <span class="pw-tree-name">{{ node.name }}</span>
                            <span v-if="!node.isDir" class="pw-tree-size">{{
                                node.sizeFmt
                            }}</span>
                        </div>
                        <!-- Children (keep alive, hide when collapsed) -->
                        <template v-if="node.isDir && node.expanded">
                            <div
                                v-if="node.loading"
                                class="pw-tree-loading"
                                :style="{
                                    paddingLeft:
                                        8 + (node.depth + 1) * 18 + 'px',
                                }"
                            >
                                <span class="pw-spinner-sm"></span>
                            </div>
                        </template>
                    </template>
                </div>
            </div>

            <!-- Resize handle -->
            <div class="pw-split-handle" @mousedown.stop="onSplitStart" />

            <!-- Right: preview -->
            <div class="pw-preview">
                <button
                    v-if="viewingExtracted"
                    class="pw-preview-back"
                    :title="
                        $t('properties.backToArchive', {
                            name: archiveFileName,
                        })
                    "
                    @click="backToArchive"
                >
                    ↰
                </button>
                <div v-if="previewLoading" class="pw-status">
                    <span class="pw-spinner"></span>
                </div>
                <div
                    v-else-if="previewError"
                    class="pw-status pw-status--error"
                >
                    <span>{{ previewError }}</span>
                </div>
                <div v-else-if="previewType === 'image'" class="pw-image-wrap">
                    <img class="pw-image" :src="previewSrc" alt="" />
                </div>
                <div v-else-if="previewType === 'docx'" class="pw-office">
                    <VueOfficeDocx
                        v-if="officeData"
                        :src="officeData"
                        style="height: 100%"
                    />
                </div>
                <div v-else-if="previewType === 'xlsx'" class="pw-office">
                    <VueOfficeExcel
                        v-if="officeData"
                        :src="officeData"
                        style="height: 100%"
                    />
                </div>
                <div v-else-if="previewType === 'pdf'" class="pw-office">
                    <VueOfficePdf
                        v-if="officeData"
                        :src="officeData"
                        style="height: 100%"
                    />
                </div>
                <div v-else-if="previewType === 'pptx'" class="pw-office">
                    <PptxPreview v-if="officeData" :data="officeData" />
                </div>
                <div v-else-if="previewType === 'text'" class="pw-code">
                    <CodePreview :code="previewContent" :ext="previewExt" />
                </div>
                <div
                    v-else-if="previewType === 'markdown'"
                    class="pw-markdown"
                    v-html="renderedMarkdown"
                />
                <div v-else-if="previewType === 'archive'" class="pw-archive">
                    <div class="pw-archive-hdr">
                        {{ archiveEntries.length }} entries
                    </div>
                    <div class="pw-archive-list">
                        <div
                            v-for="e in archiveEntries"
                            :key="e.path"
                            class="pw-archive-item"
                            :class="{
                                'pw-archive-item--active':
                                    !e.is_dir && selectedArchivePath === e.path,
                            }"
                            :style="{
                                paddingLeft:
                                    8 +
                                    (e.path.match(/\//g) || []).length * 14 +
                                    'px',
                            }"
                            @click="onArchiveEntryClick(e)"
                            @dblclick="onArchiveEntryDblClick(e)"
                            @contextmenu.prevent="onArchiveCtxMenu(e, $event)"
                        >
                            <span>{{ e.is_dir ? "📁" : "📄" }}</span>
                            <span class="pw-archive-name">{{ e.name }}</span>
                            <span v-if="!e.is_dir" class="pw-archive-size">{{
                                e.size
                            }}</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <!-- Context menu -->
        <div
            v-if="ctxMenu.show"
            class="pw-ctx-overlay"
            @click="ctxMenu.show = false"
        >
            <div
                class="pw-ctx-menu"
                :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }"
            >
                <button class="pw-ctx-item" @click="ctxOpenFile">
                    ↗ {{ $t("contextMenu.open") }}
                </button>
                <button class="pw-ctx-item" @click="ctxShowInExplorer">
                    📂 {{ $t("contextMenu.showInFinder") }}
                </button>
                <button class="pw-ctx-item" @click="ctxOpenInTerminal">
                    ⌨ {{ $t("contextMenu.openInTerminal") }}
                </button>
                <div class="pw-ctx-sep"></div>
                <button class="pw-ctx-item" @click="ctxCopyPath">
                    📋 {{ $t("contextMenu.copyPath") }}
                </button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, onMounted, watch, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import {
    listDirectory,
    getFilePreview,
    readFileBytes,
    listArchiveContents,
    extractArchiveEntry,
    openFile,
    showInExplorer,
    openInTerminal,
} from "@/utils/tauri";
import { convertFileSrc } from "@tauri-apps/api/core";
import { marked } from "marked";
import DOMPurify from "dompurify";
import VueOfficeDocx from "@vue-office/docx";
import VueOfficeExcel from "@vue-office/excel";
import VueOfficePdf from "@vue-office/pdf";
import PptxPreview from "@/components/PptxPreview.vue";
import CodePreview from "@/components/CodePreview.vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { getFileIconSvg, isBundleDirectory } from "@/utils/fileIcons";
import {
    getFileCategory,
    treeColorClassForCategory,
    formatFileSize,
} from "@/utils/fileTypes";
import type { FileEntry } from "@/types";
import type { ArchiveEntry } from "@/utils/tauri";

const { t } = useI18n();

// ── Start path ──
const params = new URLSearchParams(window.location.search);
const startPath = decodeURIComponent(params.get("preview") || "");

const activePath = ref(startPath);

// ── Tree state ──
interface TreeNode {
    path: string;
    name: string;
    isDir: boolean;
    size: number;
    sizeFmt: string;
    ext: string;
    depth: number;
    expanded: boolean;
    hasChildren: boolean;
    loading: boolean;
}
const treeNodes = shallowRef<Map<string, TreeNode>>(new Map());
const treeOrder = ref<string[]>([]);

const flatTree = computed<TreeNode[]>(() => {
    const result: TreeNode[] = [];
    const visited = new Set<string>();
    function walk(paths: string[], depth: number) {
        for (const p of paths) {
            if (visited.has(p)) continue;
            visited.add(p);
            const node = treeNodes.value.get(p);
            if (!node) continue;
            node.depth = depth;
            result.push(node);
            if (node.isDir && node.expanded) {
                const children = getChildPaths(p);
                walk(children, depth + 1);
            }
        }
    }
    walk(treeOrder.value, 0);
    return result;
});

function getChildPaths(parentPath: string): string[] {
    return treeOrder.value.filter((p) => {
        const dir = p.replace(/\\/g, "/").replace(/\/[^/]+$/, "");
        return dir === parentPath.replace(/\\/g, "/") || dir === parentPath;
    });
}

// ── Tree width resize ──
const treeWidth = ref(loadTreeWidth());

function loadTreeWidth(): number {
    try {
        const v = localStorage.getItem("pw-tree-width");
        return v ? Math.max(160, parseInt(v)) : 220;
    } catch {
        return 220;
    }
}

let _splitStartX = 0;
let _splitStartW = 0;

function onSplitStart(e: MouseEvent) {
    _splitStartX = e.clientX;
    _splitStartW = treeWidth.value;
    addEventListener("mousemove", onSplitMove);
    addEventListener("mouseup", onSplitEnd);
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
    e.preventDefault();
}

function onSplitMove(e: MouseEvent) {
    treeWidth.value = Math.max(
        160,
        Math.min(500, _splitStartW + e.clientX - _splitStartX),
    );
}

function onSplitEnd() {
    removeEventListener("mousemove", onSplitMove);
    removeEventListener("mouseup", onSplitEnd);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    localStorage.setItem("pw-tree-width", String(treeWidth.value));
}

// ── Tree operations ──
async function loadDirTree(dir: string, parentPath?: string) {
    try {
        const items = await listDirectory(dir);
        const normDir = dir.replace(/\\/g, "/");
        const nodes = new Map(treeNodes.value);

        for (const f of items) {
            const p = f.path.replace(/\\/g, "/");
            if (nodes.has(p)) continue;
            const node: TreeNode = {
                path: f.path,
                name: f.name,
                isDir: f.is_dir,
                size: f.size,
                sizeFmt: f.is_dir ? "" : formatFileSize(f.size),
                ext: f.extension,
                depth: 0,
                expanded: false,
                hasChildren: f.is_dir,
                loading: false,
            };
            nodes.set(p, node);
        }

        // Update order: remove old children of this dir, insert new ones sorted
        const newOrder = items.map((f) => f.path.replace(/\\/g, "/"));
        let order = [...treeOrder.value];
        if (parentPath) {
            const pp = parentPath.replace(/\\/g, "/");
            order = order.filter((p) => {
                const d = p.replace(/\\/g, "/").replace(/\/[^/]+$/, "");
                return d !== pp;
            });
        }
        // Insert after parent (if parent in list) or at end
        if (parentPath) {
            const pp = parentPath.replace(/\\/g, "/");
            const idx = order.indexOf(pp);
            if (idx >= 0) {
                order.splice(idx + 1, 0, ...newOrder);
            } else {
                order.push(...newOrder);
            }
        } else {
            order = [...newOrder];
        }

        treeNodes.value = nodes;
        treeOrder.value = order;
    } catch {
        /* ignore */
    }
}

async function toggleExpand(node: TreeNode) {
    if (!node.isDir) return;
    const map = new Map(treeNodes.value);
    const n = map.get(node.path);
    if (!n) return;

    if (n.expanded) {
        n.expanded = false;
        treeNodes.value = map;
        return;
    }

    // Check if has children (load if first expand)
    n.loading = true;
    treeNodes.value = new Map(map);
    await loadDirTree(node.path, node.path);
    const map2 = new Map(treeNodes.value);
    const n2 = map2.get(node.path);
    if (n2) {
        n2.expanded = true;
        n2.loading = false;
        n2.hasChildren =
            treeOrder.value.filter((p) => {
                const d = p.replace(/\\/g, "/").replace(/\/[^/]+$/, "");
                return d === node.path.replace(/\\/g, "/");
            }).length > 0;
        treeNodes.value = map2;
    }
}

function onTreeClick(node: TreeNode) {
    if (!node.isDir) {
        activePath.value = node.path;
    }
}

function onTreeDblClick(node: TreeNode) {
    if (node.isDir) {
        toggleExpand(node);
    }
}

async function onArchiveEntryDblClick(entry: ArchiveEntry) {
    if (entry.is_dir) return;
    try {
        // Save archive state for return
        lastArchivePath.value = archivePath.value;
        lastArchiveEntries.value = [...archiveEntries.value];
        viewingExtracted.value = true;

        const result = await extractArchiveEntry(archivePath.value, entry.path);
        activePath.value = result.temp_path;
    } catch (e: any) {
        previewError.value = String(e || "Extract failed");
    }
}

function backToArchive() {
    archiveEntries.value = lastArchiveEntries.value;
    archivePath.value = lastArchivePath.value;
    previewType.value = "archive";
    activePath.value = "";
    viewingExtracted.value = false;
    previewError.value = "";
}

function onArchiveEntryClick(entry: ArchiveEntry) {
    if (!entry.is_dir) {
        selectedArchivePath.value = entry.path;
    }
}

function onArchiveCtxMenu(_entry: ArchiveEntry, e: MouseEvent) {
    // Placeholder for future context menu actions
    e.preventDefault();
}

// ── Navigation ──
const canGoUp = computed(() => {
    const p = parentDir.value;
    if (!p) return false;
    if (/^[A-Za-z]:\\?$/.test(p) && p.length <= 3) return false;
    if (p === "/") return false;
    return true;
});

const parentDir = computed(() => {
    const p = activePath.value.replace(/\\/g, "/");
    const lastSep = p.lastIndexOf("/");
    if (lastSep <= 0) return p;
    return p.slice(0, lastSep);
});

const fileName = computed(() => {
    const parts = activePath.value.replace(/\\/g, "/").split("/");
    return parts[parts.length - 1] || "";
});

const archiveFileName = computed(() => {
    const parts = lastArchivePath.value.replace(/\\/g, "/").split("/");
    return parts[parts.length - 1] || "";
});

async function goUp() {
    const pp = parentDir.value;
    if (!pp) return;
    const grandParent =
        pp.replace(/\\/g, "/").replace(/\/[^/]+$/, "") ||
        pp.slice(0, pp.indexOf("/") + 1) ||
        "/";
    treeOrder.value = [];
    treeNodes.value = new Map();
    await loadDirTree(pp);
    // Expand the parent dir so its children are visible
    const map = new Map(treeNodes.value);
    const pn = map.get(pp);
    if (pn) {
        pn.expanded = true;
        treeNodes.value = map;
        await loadDirTree(pp, pp);
        const map2 = new Map(treeNodes.value);
        const pn2 = map2.get(pp);
        if (pn2) {
            pn2.expanded = true;
            pn2.loading = false;
            treeNodes.value = map2;
        }
    }
}

// ── Icons & colors ──
function fileIcon(node: TreeNode): string {
    if (node.isDir && !isBundleDirectory(node.ext, true)) return "";
    return getFileIconSvg(node.ext, false) || "";
}
function isBundle(node: TreeNode): boolean {
    return isBundleDirectory(node.ext, true);
}
function fileColorClass(node: TreeNode): string {
    return treeColorClassForCategory(getFileCategory(node.ext, false));
}

// ── Context menu ──
const ctxMenu = reactive({ show: false, x: 0, y: 0, path: "" });

function onTreeCtxMenu(e: MouseEvent) {
    ctxMenu.show = false;
}

function onItemCtxMenu(node: TreeNode, e: MouseEvent) {
    ctxMenu.show = true;
    ctxMenu.x = e.clientX;
    ctxMenu.y = e.clientY;
    ctxMenu.path = node.path;
}
function ctxOpenFile() {
    openFile(ctxMenu.path).catch(() => {});
    ctxMenu.show = false;
}
function ctxShowInExplorer() {
    showInExplorer(ctxMenu.path).catch(() => {});
    ctxMenu.show = false;
}
function ctxOpenInTerminal() {
    openInTerminal(ctxMenu.path).catch(() => {});
    ctxMenu.show = false;
}
async function ctxCopyPath() {
    try {
        await navigator.clipboard.writeText(ctxMenu.path);
    } catch {
        /* ignore */
    }
    ctxMenu.show = false;
}

// ── Preview ──
const previewType = ref("");
const previewSrc = ref("");
const previewContent = ref("");
const previewExt = ref("");
const previewLoading = ref(false);
const previewError = ref("");
const officeData = ref<ArrayBuffer | null>(null);
const archiveEntries = ref<ArchiveEntry[]>([]);
const archivePath = ref("");
const selectedArchivePath = ref("");
const lastArchivePath = ref("");
const lastArchiveEntries = ref<ArchiveEntry[]>([]);
const viewingExtracted = ref(false);
const renderedMarkdown = ref("");

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
const OFFICE_EXTS: Record<string, string> = {
    docx: "docx",
    xlsx: "xlsx",
    pptx: "pptx",
};

async function loadPreview(path: string) {
    if (!path) return;
    previewType.value = "";
    previewError.value = "";
    officeData.value = null;
    const ext = (path.split(".").pop() || "").toLowerCase();
    if (IMG_EXTS.includes(ext)) {
        previewType.value = "image";
        previewSrc.value = convertFileSrc(path);
        previewLoading.value = false;
    } else if (OFFICE_EXTS[ext]) {
        previewLoading.value = true;
        try {
            const buf = await loadAsArrayBuffer(path);
            officeData.value = buf;
            previewType.value = OFFICE_EXTS[ext];
        } catch (e: any) {
            previewError.value = String(e);
        }
        previewLoading.value = false;
    } else if (ext === "pdf") {
        previewLoading.value = true;
        try {
            const buf = await loadAsArrayBuffer(path);
            officeData.value = buf;
            previewType.value = "pdf";
        } catch (e: any) {
            previewError.value = String(e);
        }
        previewLoading.value = false;
    } else if (ARCHIVE_EXTS.includes(ext)) {
        previewLoading.value = true;
        try {
            const entries = await listArchiveContents(path);
            archiveEntries.value = entries;
            archivePath.value = path;
            previewType.value = "archive";
        } catch (e: any) {
            previewError.value = String(e);
        }
        previewLoading.value = false;
    } else {
        previewLoading.value = true;
        try {
            const result = await getFilePreview(path);
            if (result.type === "markdown") {
                previewType.value = "markdown";
                renderedMarkdown.value = DOMPurify.sanitize(
                    await marked.parse(result.content || ""),
                );
            } else {
                previewType.value = "text";
                previewContent.value = result.content || "";
                previewExt.value = result.ext || ext;
            }
        } catch (e: any) {
            previewError.value = String(e);
        }
        previewLoading.value = false;
    }
}

async function loadAsArrayBuffer(path: string): Promise<ArrayBuffer> {
    const b64 = await readFileBytes(path);
    const chunks: string[] = [];
    for (let i = 0; i < b64.length; i += 1024 * 1024) {
        chunks.push(atob(b64.slice(i, i + 1024 * 1024)));
        await new Promise((r) => setTimeout(r, 0));
    }
    const bin = chunks.join("");
    const buf = new ArrayBuffer(bin.length);
    const v = new Uint8Array(buf);
    for (let i = 0; i < bin.length; i += 256 * 1024) {
        const end = Math.min(i + 256 * 1024, bin.length);
        for (let j = i; j < end; j++) v[j] = bin.charCodeAt(j);
        if (i + 256 * 1024 < bin.length)
            await new Promise((r) => setTimeout(r, 0));
    }
    return buf;
}

function closeWindow() {
    getCurrentWebviewWindow().close();
}

watch(activePath, (p) => {
    if (p) loadPreview(p);
});

onMounted(async () => {
    if (startPath) {
        await loadDirTree(parentDir.value);
        // Auto-expand the directory containing the active file
        const map = new Map(treeNodes.value);
        const pn = map.get(parentDir.value);
        if (pn) {
            pn.expanded = true;
            pn.loading = false;
            treeNodes.value = map;
            await loadDirTree(parentDir.value, parentDir.value);
            const map2 = new Map(treeNodes.value);
            const pn2 = map2.get(parentDir.value);
            if (pn2) {
                pn2.expanded = true;
                pn2.loading = false;
                treeNodes.value = map2;
            }
        }
        loadPreview(startPath);
    }
});
</script>

<style scoped>
.pw-root {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-primary);
    color: var(--text-primary);
}
.pw-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
}
.pw-title {
    font-size: 13px;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.pw-close {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 16px;
    padding: 2px 6px;
    border-radius: 4px;
}
.pw-close:hover {
    background: var(--bg-hover);
    color: var(--danger);
}
/* ── Back bar ── */
.pw-back-bar {
    padding: 4px 8px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
}
.pw-back-btn {
    background: none;
    border: 1px solid var(--border);
    color: var(--accent);
    padding: 2px 10px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    transition: background 0.15s;
}
.pw-back-btn:hover {
    background: var(--bg-hover);
}
.pw-body {
    flex: 1;
    display: flex;
    overflow: hidden;
}

/* ── Tree ── */
.pw-tree {
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border);
    overflow: hidden;
}
.pw-tree-nav {
    padding: 6px 8px;
    border-bottom: 1px solid var(--border);
}
.pw-nav-btn {
    background: var(--bg-hover);
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: 2px 8px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
}
.pw-nav-btn:disabled {
    opacity: 0.3;
    cursor: default;
}
.pw-nav-btn:hover:not(:disabled) {
    background: var(--accent);
    color: #fff;
}
.pw-tree-list {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 2px 0;
}
.pw-tree-item {
    display: flex;
    align-items: center;
    gap: 3px;
    padding: 2px 4px;
    margin: 0 4px;
    cursor: pointer;
    font-size: 12px;
    min-height: 26px;
    border-radius: 4px;
    white-space: nowrap;
    transition: background 0.05s;
}
.pw-tree-item:hover {
    background: var(--bg-hover);
}
.pw-tree-item--active {
    background: var(--bg-selected) !important;
}
.pw-tree-item--active:hover {
    background: var(--bg-selected) !important;
}
.pw-tree-chevron {
    width: 14px;
    height: 14px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    transition: transform 0.15s;
}
.pw-tree-chevron--expanded {
    transform: rotate(90deg);
}
.pw-tree-chevron--hidden {
    visibility: hidden;
}
.pw-tree-icon-wrap {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
}
.pw-tree-icon-svg {
    width: 16px;
    height: 16px;
}
.pw-tree-icon-svg :deep(svg) {
    width: 16px;
    height: 16px;
}
.pw-tree-name {
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
    font-size: 12px;
}
.pw-tree-size {
    margin-left: auto;
    font-size: 10px;
    color: var(--text-muted);
    flex-shrink: 0;
}
.pw-tree-loading {
    padding: 3px 12px;
}
.pw-spinner-sm {
    display: inline-block;
    width: 10px;
    height: 10px;
    border: 2px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: pw-spin 0.6s linear infinite;
}

/* ── Split handle ── */
.pw-split-handle {
    width: 4px;
    flex-shrink: 0;
    cursor: col-resize;
    background: transparent;
    transition: background 0.1s;
    z-index: 1;
}
.pw-split-handle:hover,
.pw-split-handle:active {
    background: var(--accent);
    opacity: 0.7;
}

/* ── Preview ── */
.pw-preview {
    flex: 1;
    overflow: auto;
    display: flex;
    flex-direction: column;
    padding: 8px;
    position: relative;
}
.pw-preview-back {
    position: absolute;
    top: 4px;
    right: 8px;
    z-index: 5;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    color: var(--accent);
    width: 28px;
    height: 28px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0.7;
    transition:
        opacity 0.15s,
        background 0.15s;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.15);
}
.pw-preview-back:hover {
    opacity: 1;
    background: var(--bg-hover);
}
.pw-status {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    font-size: 13px;
}
.pw-status--error {
    color: var(--danger);
}
.pw-spinner {
    width: 20px;
    height: 20px;
    border: 2px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: pw-spin 0.6s linear infinite;
}
@keyframes pw-spin {
    to {
        transform: rotate(360deg);
    }
}
.pw-image-wrap {
    flex: 1;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding: 12px;
}
.pw-image {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    border-radius: 4px;
}
.pw-office {
    flex: 1;
    overflow: auto;
}
.pw-code {
    flex: 1;
    overflow: auto;
}
.pw-markdown {
    padding: 12px 16px;
    font-size: 13px;
    line-height: 1.6;
    overflow: auto;
    flex: 1;
    color: var(--text-primary);
}

/* ── Archive ── */
.pw-archive {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}
.pw-archive-hdr {
    padding: 6px 8px;
    font-size: 11px;
    color: var(--text-muted);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
}
.pw-archive-list {
    flex: 1;
    overflow: auto;
    padding: 2px 0;
}
.pw-archive-item {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px 6px;
    font-size: 11px;
    white-space: nowrap;
    cursor: pointer;
    border-radius: 3px;
    margin: 0 2px;
}
.pw-archive-item:hover {
    background: var(--bg-hover);
}
.pw-archive-item--active {
    background: var(--bg-selected);
}
.pw-archive-name {
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
}
.pw-archive-size {
    margin-left: auto;
    font-size: 10px;
    color: var(--text-muted);
    flex-shrink: 0;
}

/* ── Context menu ── */
.pw-ctx-overlay {
    position: fixed;
    inset: 0;
    z-index: 9999;
}
.pw-ctx-menu {
    position: fixed;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 4px;
    min-width: 180px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
    font-size: 12px;
}
.pw-ctx-item {
    display: block;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    color: var(--text-primary);
    padding: 5px 10px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
}
.pw-ctx-item:hover {
    background: var(--bg-hover);
}
.pw-ctx-sep {
    height: 1px;
    background: var(--border);
    margin: 4px 6px;
}
</style>
