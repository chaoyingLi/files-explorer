<template>
    <div class="code-root">
        <div v-if="loading" class="code-status">
            <span class="code-spinner"></span>
            {{ $t("properties.previewLoading") }}
        </div>
        <div
            v-else-if="renderedHtml"
            class="code-editor"
            v-html="renderedHtml"
        />
        <pre v-else class="code-fallback"><code>{{ fallbackCode }}</code></pre>
    </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "@/stores/settingsStore";
import { createHighlighter, type Highlighter, type ThemedToken } from "shiki";
import { createJavaScriptRawEngine } from "@shikijs/engine-javascript";

const { t } = useI18n();
const props = defineProps<{ code: string; ext: string }>();

const loading = ref(true);
const renderedHtml = ref("");
const fallbackCode = ref("");

const EXT_TO_LANG: Record<string, string> = {
    // ── JavaScript / TypeScript ──
    js: "javascript",
    mjs: "javascript",
    cjs: "javascript",
    ts: "typescript",
    mts: "typescript",
    cts: "typescript",
    tsx: "tsx",
    jsx: "jsx",
    vue: "vue",
    svelte: "svelte",
    // ── Rust / Python / Go ──
    rs: "rust",
    py: "python",
    pyi: "python",
    pyx: "python",
    ipynb: "python",
    go: "go",
    // ── JVM ──
    java: "java",
    kt: "kotlin",
    kts: "kotlin",
    scala: "scala",
    sc: "scala",
    groovy: "groovy",
    gvy: "groovy",
    gradle: "groovy",
    // ── Apple ──
    swift: "swift",
    m: "objc",
    mm: "objc",
    // ── Dart / Ruby / PHP / Lua / R / Perl ──
    dart: "dart",
    rb: "ruby",
    php: "php",
    lua: "lua",
    r: "r",
    pl: "perl",
    // ── C / C++ / C# ──
    c: "c",
    h: "c",
    cpp: "cpp",
    cc: "cpp",
    cxx: "cpp",
    hpp: "cpp",
    hxx: "cpp",
    hh: "cpp",
    inl: "cpp",
    cs: "csharp",
    csx: "csharp",
    // ── F# / VB / PowerShell ──
    fs: "fsharp",
    fsx: "fsharp",
    fsi: "fsharp",
    vb: "vb",
    ps1: "powershell",
    psm1: "powershell",
    psd1: "powershell",
    // ── CSS ──
    css: "css",
    scss: "scss",
    sass: "sass",
    less: "less",
    styl: "stylus",
    // ── HTML / XML / JSON / YAML / TOML ──
    html: "html",
    htm: "html",
    xml: "xml",
    svg: "xml",
    xaml: "xml",
    json: "json",
    jsonc: "jsonc",
    json5: "json5",
    yaml: "yaml",
    yml: "yaml",
    toml: "toml",
    // ── Markdown / SQL ──
    md: "markdown",
    mdx: "markdown",
    sql: "sql",
    // ── Shell / Batch / Config ──
    sh: "shellscript",
    bash: "shellscript",
    zsh: "shellscript",
    fish: "shellscript",
    bat: "bat",
    cmd: "bat",
    ini: "ini",
    cfg: "ini",
    conf: "ini",
    env: "ini",
    editorconfig: "ini",
    properties: "properties",
    gitignore: "ignore",
    dockerfile: "dockerfile",
    // ── Infrastructure ──
    tf: "hcl",
    tfvars: "hcl",
    hcl: "hcl",
    proto: "protobuf",
    prisma: "prisma",
    graphql: "graphql",
    gql: "graphql",
    // ── Functional ──
    hs: "haskell",
    lhs: "haskell",
    erl: "erlang",
    hrl: "erlang",
    ex: "elixir",
    exs: "elixir",
    clj: "clojure",
    cljs: "clojure",
    edn: "clojure",
    // ── Systems / Embedded ──
    zig: "zig",
    nim: "nim",
    v: "verilog",
    sv: "systemverilog",
    vh: "verilog",
    vhd: "vhdl",
    vhdl: "vhdl",
    // ── Blockchain ──
    sol: "solidity",
    // ── Misc ──
    ml: "ocaml",
    mli: "ocaml",
    coffee: "coffeescript",
    litcoffee: "coffeescript",
    pug: "pug",
    jade: "pug",
    tex: "latex",
    sty: "latex",
    cls: "latex",
    bib: "bibtex",
    makefile: "makefile",
    cmake: "cmake",
    log: "log",
    txt: "text",
    diff: "diff",
    patch: "diff",
};

let _highlighter: Highlighter | null = null;

async function getHighlighter(): Promise<Highlighter> {
    if (_highlighter) return _highlighter;
    _highlighter = await createHighlighter({
        themes: ["catppuccin-mocha", "catppuccin-latte"],
        langs: [
            "javascript",
            "typescript",
            "tsx",
            "jsx",
            "vue",
            "svelte",
            "rust",
            "python",
            "go",
            "java",
            "kotlin",
            "scala",
            "groovy",
            "swift",
            "objc",
            "dart",
            "ruby",
            "php",
            "lua",
            "r",
            "perl",
            "c",
            "cpp",
            "csharp",
            "fsharp",
            "vb",
            "powershell",
            "css",
            "scss",
            "sass",
            "less",
            "stylus",
            "html",
            "xml",
            "json",
            "jsonc",
            "json5",
            "yaml",
            "toml",
            "markdown",
            "sql",
            "shellscript",
            "bat",
            "ini",
            "properties",
            "ignore",
            "dockerfile",
            "hcl",
            "protobuf",
            "prisma",
            "graphql",
            "haskell",
            "erlang",
            "elixir",
            "clojure",
            "zig",
            "nim",
            "verilog",
            "systemverilog",
            "vhdl",
            "solidity",
            "ocaml",
            "coffeescript",
            "pug",
            "latex",
            "bibtex",
            "makefile",
            "cmake",
            "log",
            "text",
            "diff",
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

/** Escape HTML special chars */
function esc(s: string): string {
    return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

/** Build token-level inline styles from Shiki themed token */
function tokenStyle(t: ThemedToken): string {
    const parts: string[] = [];
    if (t.color) parts.push(`color:${t.color}`);
    if (t.bgColor) parts.push(`background:${t.bgColor}`);
    if (t.fontStyle && t.fontStyle !== (0 as any)) {
        const fs = String(t.fontStyle);
        if (fs.includes("Italic")) parts.push("font-style:italic");
        if (fs.includes("Bold")) parts.push("font-weight:bold");
        if (fs.includes("Underline")) parts.push("text-decoration:underline");
    }
    return parts.join(";");
}

/** Build per-token <span> for a single token */
function tokenHtml(t: ThemedToken): string {
    const s = tokenStyle(t);
    if (!s) return esc(t.content);
    return `<span style="${s}">${esc(t.content)}</span>`;
}

/** Build one line: <span class="code-line">...tokens...</span> */
function lineHtml(tokens: ThemedToken[]): string {
    let inner = "";
    for (const t of tokens) {
        inner += tokenHtml(t);
    }
    if (!inner) inner = " ";
    return `<span class="code-line">${inner}</span>`;
}

async function highlight() {
    const code = props.code;
    if (!code || !code.trim()) {
        loading.value = false;
        renderedHtml.value = "";
        fallbackCode.value = "";
        return;
    }
    const langId = EXT_TO_LANG[props.ext.toLowerCase()] || "text";
    loading.value = true;
    try {
        const hl = await getHighlighter();
        const theme = getTheme();
        // Get per-line tokens from Shiki
        const result = hl.codeToTokens(code, {
            lang: langId as any,
            theme,
        });
        const lines: ThemedToken[][] =
            "tokens" in result ? (result.tokens as ThemedToken[][]) : [];
        // Build HTML manually: full control over line spans
        let html = "";
        for (const lineTokens of lines) {
            html += lineHtml(lineTokens) + "\n";
        }
        // Get theme background color for the <pre>
        const themeBg = hl.getTheme(theme).bg || "#1e1e2e";
        renderedHtml.value = `<pre style="background-color:${themeBg};tab-size:4;-moz-tab-size:4"><code>${html}</code></pre>`;
        fallbackCode.value = "";
    } catch {
        fallbackCode.value = code;
        renderedHtml.value = "";
    } finally {
        loading.value = false;
    }
}

watch(
    () => [props.code, props.ext],
    () => highlight(),
);
onMounted(() => highlight());
</script>

<style>
/* Plain <style> without scoped — guarantees match on v-html content */
.code-editor pre {
    counter-reset: code-line-nr;
    margin: 0;
    padding: 12px 0;
    font-family:
        "SF Mono", "Fira Code", "JetBrains Mono", "Cascadia Code", "Consolas",
        monospace;
    font-size: 12px;
    line-height: 1.65;
    min-height: 100%;
    overflow: auto;
}
.code-editor code {
    display: block;
    font-family: inherit;
    font-size: inherit;
    line-height: inherit;
}
.code-line {
    display: block;
    padding: 0 16px 0 56px;
    position: relative;
    min-height: 1.65em;
    transition: background 0.08s;
    counter-increment: code-line-nr;
}
.code-line:hover {
    background: var(--bg-hover);
}
.code-line::before {
    content: counter(code-line-nr);
    position: absolute;
    left: 0;
    top: 0;
    width: 44px;
    padding-right: 8px;
    text-align: right;
    color: var(--text-muted);
    opacity: 0.45;
    font-size: 11px;
    line-height: inherit;
    user-select: none;
    border-right: 1px solid var(--border);
}
</style>

<style scoped>
.code-root {
    width: 100%;
    height: 100%;
    overflow: auto;
}
.code-status {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 16px;
    color: var(--text-muted);
    font-size: 12px;
}
.code-spinner {
    width: 14px;
    height: 14px;
    border: 2px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: code-spin 0.6s linear infinite;
}
@keyframes code-spin {
    to {
        transform: rotate(360deg);
    }
}
.code-fallback {
    margin: 0;
    padding: 12px;
    font-family: monospace;
    font-size: 11px;
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-all;
    color: var(--text-primary);
}
</style>
