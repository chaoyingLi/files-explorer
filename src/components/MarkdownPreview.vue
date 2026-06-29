<template>
    <div class="md-root">
        <div class="md-toolbar">
            <div class="md-toolbar-group">
                <button
                    class="md-tb-btn"
                    :class="{ active: mdMode === 'preview' }"
                    :title="$t('markdown.preview')"
                    @click.stop="mdMode = 'preview'"
                >
                    👁 {{ $t("markdown.preview") }}
                </button>
                <button
                    class="md-tb-btn"
                    :class="{ active: mdMode === 'edit' }"
                    :title="$t('markdown.edit')"
                    @click.stop="mdMode = 'edit'"
                >
                    ✏️ {{ $t("markdown.edit") }}
                </button>
            </div>
            <div class="md-toolbar-sep"></div>
            <div class="md-toolbar-group">
                <button
                    class="md-tb-btn"
                    :title="$t('markdown.exportHtml')"
                    @click.stop="exportAs('html')"
                >
                    HTML
                </button>
                <button
                    class="md-tb-btn"
                    :title="$t('markdown.exportWord')"
                    @click.stop="exportAs('doc')"
                >
                    Word
                </button>
                <button
                    class="md-tb-btn"
                    :title="$t('markdown.exportPdf')"
                    @click.stop="exportAs('pdf')"
                >
                    PDF
                </button>
            </div>
        </div>
        <div v-if="mdMode === 'preview'" class="md-preview">
            <div v-if="hlLoading" class="md-loading">
                <span class="md-spinner"></span>
                <span>{{ $t("markdown.loading") }}</span>
            </div>
            <div
                v-else
                class="md-content"
                v-html="renderedHtml"
                @contextmenu.stop
            />
        </div>
        <textarea
            v-else
            class="md-editor"
            :value="content"
            @input="onEdit"
            spellcheck="false"
        ></textarea>
    </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, nextTick } from "vue";
import { useI18n } from "vue-i18n";
import { marked } from "marked";
import DOMPurify from "dompurify";
import { saveTextFile } from "@/utils/tauri";
import { save } from "@tauri-apps/plugin-dialog";
import { createHighlighter, type Highlighter } from "shiki";
import { createJavaScriptRawEngine } from "@shikijs/engine-javascript";
import { useSettingsStore } from "@/stores/settingsStore";

// ── Shiki highlighter singleton ──
let _highlighter: Highlighter | null = null;
async function getHighlighter(): Promise<Highlighter> {
    if (_highlighter) return _highlighter;
    _highlighter = await createHighlighter({
        themes: ["catppuccin-mocha", "catppuccin-latte"],
        // Most common languages only — faster initial load
        langs: [
            "javascript",
            "typescript",
            "tsx",
            "jsx",
            "vue",
            "python",
            "rust",
            "go",
            "java",
            "cpp",
            "c",
            "csharp",
            "bash",
            "shellscript",
            "css",
            "html",
            "json",
            "yaml",
            "toml",
            "markdown",
            "sql",
            "php",
            "swift",
            "kotlin",
            "dart",
            "lua",
            "diff",
            "dockerfile",
            "makefile",
            "powershell",
            "graphql",
        ],
        engine: createJavaScriptRawEngine(),
    });
    return _highlighter;
}

function getTheme(): string {
    return useSettingsStore().theme === "light"
        ? "catppuccin-latte"
        : "catppuccin-mocha";
}

const props = defineProps<{
    content: string;
    ext: string;
    filePath?: string;
}>();
const emit = defineEmits<{ edit: [text: string] }>();

marked.setOptions({ gfm: true, breaks: true });

const { t } = useI18n();
const mdMode = ref<"preview" | "edit">("preview");
const editableContent = ref(props.content);
const renderedHtml = ref("");
const hlLoading = ref(true);

watch(
    () => props.content,
    (v) => {
        editableContent.value = v;
    },
);
watch(
    () => props.content,
    () => {
        renderMarkdown();
    },
);
watch(mdMode, (v) => {
    if (v === "preview") renderMarkdown();
});
onMounted(() => {
    renderMarkdown();
});

function onEdit(e: Event) {
    editableContent.value = (e.target as HTMLTextAreaElement).value;
}

async function renderMarkdown() {
    if (!editableContent.value) {
        renderedHtml.value = "";
        hlLoading.value = false;
        return;
    }
    hlLoading.value = true;
    try {
        // Step 1: Parse markdown to HTML
        const rawHtml = (await marked.parse(editableContent.value)) as string;
        // Step 2: Post-process — highlight code blocks with Shiki
        const temp = document.createElement("div");
        temp.innerHTML = rawHtml;
        const codeBlocks = temp.querySelectorAll<HTMLElement>("pre code");
        let highlighted = 0;
        for (const block of codeBlocks) {
            const cls = block.className || "";
            const lang = cls.replace(/^language-/, "") || "";
            const text = block.textContent || "";
            if (lang) {
                try {
                    const hl = await getHighlighter();
                    const theme = getTheme();
                    const shikiHtml = hl.codeToHtml(text, {
                        lang: lang as any,
                        theme,
                    });
                    const wrapper = document.createElement("div");
                    wrapper.innerHTML = shikiHtml;
                    const shikiPre = wrapper.querySelector("pre");
                    if (shikiPre) {
                        // Add copy button
                        const copyBtn = document.createElement("button");
                        copyBtn.className = "md-copy-btn";
                        copyBtn.textContent = "📋";
                        copyBtn.title = "Copy code";
                        copyBtn.addEventListener("click", (e) => {
                            e.stopPropagation();
                            navigator.clipboard.writeText(text).then(() => {
                                copyBtn.textContent = "✅";
                                setTimeout(() => {
                                    copyBtn.textContent = "📋";
                                }, 1500);
                            });
                        });
                        shikiPre.style.position = "relative";
                        shikiPre.appendChild(copyBtn);
                        const pre = block.parentElement;
                        if (pre && pre.parentElement) {
                            pre.parentElement.replaceChild(shikiPre, pre);
                        }
                    }
                } catch {
                    /* fallback — keep original */
                }
            }
            highlighted++;
            // Yield after every 3 blocks to keep UI responsive
            if (highlighted % 3 === 0)
                await new Promise((r) => setTimeout(r, 0));
        }
        renderedHtml.value = DOMPurify.sanitize(temp.innerHTML, {
            ADD_ATTR: ["style"],
        });
    } catch (e) {
        console.error("Markdown render failed:", e);
        renderedHtml.value = "<p>Parse error</p>";
    } finally {
        hlLoading.value = false;
    }
}

function wrapExportHtml(htmlBody: string): string {
    return `<!DOCTYPE html>
<html lang="en">
<head><meta charset="UTF-8"><title>Exported</title>
<style>
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; line-height: 1.6; color: #333; }
img { max-width: 100%; }
pre { background: #f4f4f4; padding: 12px; border-radius: 6px; overflow: auto; position: relative; }
code { background: #f0f0f0; padding: 2px 6px; border-radius: 3px; font-size: 0.9em; }
pre code { background: none; padding: 0; }
</style>
</head>
<body>${htmlBody}</body>
</html>`;
}

function deriveDefaultPath(format: string): string {
    if (props.filePath) {
        const dot = props.filePath.lastIndexOf(".");
        const base =
            dot > 0 ? props.filePath.substring(0, dot) : props.filePath;
        const sep = props.filePath.includes("/") ? "/" : "\\";
        const dir =
            dot > 0
                ? props.filePath.substring(
                      0,
                      props.filePath.lastIndexOf(sep) + 1,
                  )
                : "";
        return `${dir}${base.split(sep).pop() || "export"}.${format}`;
    }
    return `export.${format}`;
}

async function exportAs(format: "html" | "doc" | "pdf") {
    if (format === "pdf") {
        const iframe = document.createElement("iframe");
        iframe.style.position = "fixed";
        iframe.style.top = "-9999px";
        iframe.style.width = "800px";
        iframe.style.height = "600px";
        document.body.appendChild(iframe);
        const doc = iframe.contentWindow?.document;
        if (!doc) return;
        doc.open();
        doc.write(wrapExportHtml(renderedHtml.value));
        doc.close();
        iframe.contentWindow?.focus();
        setTimeout(() => {
            iframe.contentWindow?.print();
            setTimeout(() => document.body.removeChild(iframe), 1000);
        }, 500);
        return;
    }
    try {
        const ext = format === "doc" ? "doc" : "html";
        const dest = await save({
            defaultPath: deriveDefaultPath(ext),
            filters: [
                {
                    name: format === "doc" ? "Word Document" : "HTML File",
                    extensions: [ext],
                },
            ],
        });
        if (!dest) return;
        await saveTextFile(dest, wrapExportHtml(renderedHtml.value));
    } catch (e: any) {
        console.error("Export failed:", e);
    }
}
</script>

<style scoped>
.md-root {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
}
.md-toolbar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-tertiary);
    flex-shrink: 0;
    flex-wrap: wrap;
}
.md-toolbar-group {
    display: flex;
    align-items: center;
    gap: 2px;
}
.md-toolbar-sep {
    width: 1px;
    height: 16px;
    background: var(--border);
    flex-shrink: 0;
}
.md-tb-btn {
    background: var(--bg-hover);
    border: 1px solid transparent;
    color: var(--text-secondary);
    cursor: pointer;
    border-radius: 4px;
    padding: 2px 8px;
    font-size: 11px;
    line-height: 1.4;
    white-space: nowrap;
    transition: all 0.15s;
}
.md-tb-btn:hover:not(:disabled) {
    background: var(--accent);
    color: #fff;
}
.md-tb-btn.active {
    background: var(--accent);
    color: #fff;
    border-color: var(--accent);
}

/* ── Preview content ── */
.md-preview {
    flex: 1;
    overflow: auto;
    padding: 12px 24px;
    background: var(--bg-primary);
    font-size: var(--font-size-base);
    line-height: 1.7;
    color: var(--text-primary);
}
.md-content h1,
.md-content h2,
.md-content h3,
.md-content h4 {
    margin: 16px 0 8px;
    font-weight: 600;
}
.md-content h1 {
    font-size: 20px;
    border-bottom: 1px solid var(--border);
    padding-bottom: 6px;
}
.md-content h2 {
    font-size: 17px;
    border-bottom: 1px solid var(--border);
    padding-bottom: 4px;
}
.md-content h3 {
    font-size: 15px;
}
.md-content h4 {
    font-size: 13px;
}
.md-content p {
    margin: 8px 0;
}
.md-content ul,
.md-content ol {
    padding-left: 28px;
    margin: 8px 0;
}
.md-content li {
    margin: 2px 0;
}

/* ── Task lists (GFM checkboxes) ── */
.md-content ul:has(input[type="checkbox"]),
.md-content ol:has(input[type="checkbox"]) {
    list-style: none;
    padding-left: 4px;
}
.md-content input[type="checkbox"] {
    margin: 0 6px 0 0;
    accent-color: var(--accent);
    transform: scale(0.9);
    vertical-align: middle;
}

/* ── Inline code ── */
.md-content code {
    background: var(--bg-hover);
    padding: 2px 7px;
    border-radius: 5px;
    font-size: 0.88em;
    color: var(--accent);
    font-family: "SF Mono", "Fira Code", "Consolas", monospace;
}

/* ── Code blocks (Shiki generates <pre style="...">) ── */
.md-content :deep(pre) {
    background: var(--bg-tertiary) !important;
    border: 1px solid var(--border);
    border-left: 3px solid var(--accent);
    border-radius: 8px;
    padding: 16px !important;
    overflow: auto;
    margin: 12px 0;
    font-size: 13px;
    line-height: 1.5;
    position: relative;
}
.md-content :deep(pre code) {
    background: none;
    padding: 0;
    font-family: "SF Mono", "Fira Code", "Consolas", monospace;
    tab-size: 4;
    color: inherit;
}
.md-content :deep(code .shiki) {
    background: none !important;
}

/* ── Code copy button ── */
.md-copy-btn {
    position: absolute;
    top: 6px;
    right: 8px;
    background: var(--bg-hover);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 2px 6px;
    font-size: 12px;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.15s;
    line-height: 1.4;
    z-index: 1;
    color: var(--text-muted);
}
.md-content :deep(pre):hover .md-copy-btn,
.md-copy-btn {
    opacity: 0;
}
.md-content :deep(pre):hover .md-copy-btn {
    opacity: 1;
}

/* ── Tables ── */
.md-content table {
    border-collapse: collapse;
    width: 100%;
    margin: 12px 0;
    font-size: 0.95em;
}
.md-content th,
.md-content td {
    border: 1px solid var(--border);
    padding: 8px 12px;
    text-align: left;
}
.md-content th {
    background: var(--bg-secondary);
    font-weight: 600;
}

/* ── Blockquote ── */
.md-content blockquote {
    border-left: 4px solid var(--accent);
    padding: 4px 16px;
    margin: 12px 0;
    color: var(--text-secondary);
    background: var(--bg-secondary);
    border-radius: 0 6px 6px 0;
}

/* ── Images & rules ── */
.md-content img {
    max-width: 100%;
    border-radius: 6px;
    margin: 8px 0;
}
.md-content hr {
    border: none;
    border-top: 1px solid var(--border);
    margin: 20px 0;
}

/* ── Editor ── */
.md-editor {
    flex: 1;
    resize: none;
    border: none;
    outline: none;
    padding: 12px 16px;
    font-family: "SF Mono", "Fira Code", "Consolas", monospace;
    font-size: 13px;
    line-height: 1.6;
    background: var(--bg-primary);
    color: var(--text-primary);
    tab-size: 4;
}

/* ── Loading ── */
.md-loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 48px 12px;
    color: var(--text-muted);
    font-size: 12px;
    min-height: 200px;
}
.md-spinner {
    width: 24px;
    height: 24px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: md-spin 0.6s linear infinite;
    flex-shrink: 0;
}
@keyframes md-spin {
    to {
        transform: rotate(360deg);
    }
}
</style>
