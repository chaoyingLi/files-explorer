<template>
    <div class="code-root" :style="{ fontSize: FONT_SIZES[settings.fontSize] }">
        <Codemirror
            :key="editorKey"
            v-model:value="codeText"
            :options="cmOptions"
            border
            height="100%"
            width="100%"
            @ready="onReady"
        />
    </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onUnmounted } from "vue";
import { useSettingsStore } from "@/stores/settingsStore";
import Codemirror from "codemirror-editor-vue3";
import type { CmComponentRef } from "codemirror-editor-vue3";

// ── Core CodeMirror modes ──
import "codemirror/mode/javascript/javascript.js";
import "codemirror/mode/jsx/jsx.js";
import "codemirror/mode/vue/vue.js";
import "codemirror/mode/rust/rust.js";
import "codemirror/mode/python/python.js";
import "codemirror/mode/go/go.js";
import "codemirror/mode/clike/clike.js"; // java/kotlin/c/cpp/csharp
import "codemirror/mode/swift/swift.js";
import "codemirror/mode/dart/dart.js";
import "codemirror/mode/ruby/ruby.js";
import "codemirror/mode/php/php.js";
import "codemirror/mode/lua/lua.js";
import "codemirror/mode/r/r.js";
import "codemirror/mode/perl/perl.js";
import "codemirror/mode/css/css.js";
import "codemirror/mode/sass/sass.js";
import "codemirror/mode/htmlmixed/htmlmixed.js";
import "codemirror/mode/xml/xml.js";
import "codemirror/mode/sql/sql.js";
import "codemirror/mode/shell/shell.js";
import "codemirror/mode/powershell/powershell.js";
import "codemirror/mode/dockerfile/dockerfile.js";
import "codemirror/mode/yaml/yaml.js";
import "codemirror/mode/toml/toml.js";
import "codemirror/mode/markdown/markdown.js";
import "codemirror/mode/diff/diff.js";
import "codemirror/mode/groovy/groovy.js";
import "codemirror/mode/clojure/clojure.js";
import "codemirror/mode/haskell/haskell.js";
import "codemirror/mode/erlang/erlang.js";
import "codemirror/mode/elm/elm.js";
import "codemirror/mode/coffeescript/coffeescript.js";
import "codemirror/mode/pug/pug.js";
import "codemirror/mode/stylus/stylus.js";
import "codemirror/mode/protobuf/protobuf.js";
import "codemirror/mode/cmake/cmake.js";
import "codemirror/mode/mllike/mllike.js";
import "codemirror/mode/vb/vb.js";
import "codemirror/mode/vhdl/vhdl.js";
import "codemirror/mode/verilog/verilog.js";
import "codemirror/mode/properties/properties.js";
// Themes
import "codemirror/theme/material-darker.css";
import "codemirror/theme/eclipse.css";
import "codemirror/lib/codemirror.css";

const props = defineProps<{ code: string; ext: string }>();
const settings = useSettingsStore();

const codeText = ref(props.code);
const cmRef = ref<CmComponentRef>();

// ── Font size from settings ──
const FONT_SIZES: Record<string, string> = {
    small: "11px",
    medium: "13px",
    large: "15px",
};
const editorKey = ref(0);
watch(
    () => settings.fontSize,
    () => {
        editorKey.value++;
    },
);

// ── Extension → CodeMirror mode ──
const EXT_TO_MODE: Record<string, string> = {
    js: "javascript",
    mjs: "javascript",
    cjs: "javascript",
    ts: "javascript",
    tsx: "jsx",
    jsx: "jsx",
    json: "application/json",
    jsonc: "application/json",
    json5: "application/json",
    vue: "vue",
    rs: "rust",
    py: "python",
    pyi: "python",
    pyx: "python",
    go: "go",
    java: "text/x-java",
    kt: "text/x-kotlin",
    kts: "text/x-kotlin",
    scala: "text/x-scala",
    sc: "text/x-scala",
    c: "text/x-csrc",
    h: "text/x-csrc",
    cpp: "text/x-c++src",
    cc: "text/x-c++src",
    cxx: "text/x-c++src",
    hpp: "text/x-c++src",
    hxx: "text/x-c++src",
    cs: "text/x-csharp",
    csx: "text/x-csharp",
    swift: "swift",
    dart: "dart",
    rb: "ruby",
    php: "php",
    lua: "lua",
    r: "r",
    pl: "perl",
    css: "css",
    scss: "sass",
    sass: "sass",
    less: "text/x-less",
    html: "htmlmixed",
    htm: "htmlmixed",
    xml: "xml",
    svg: "xml",
    xaml: "xml",
    yaml: "yaml",
    yml: "yaml",
    toml: "toml",
    md: "markdown",
    mdx: "markdown",
    sql: "sql",
    sh: "shell",
    bash: "shell",
    zsh: "shell",
    fish: "shell",
    bat: "shell",
    cmd: "shell",
    ps1: "powershell",
    psm1: "powershell",
    dockerfile: "dockerfile",
    ini: "properties",
    cfg: "properties",
    conf: "properties",
    env: "properties",
    editorconfig: "properties",
    gitignore: "shell",
    hs: "haskell",
    lhs: "haskell",
    erl: "erlang",
    hrl: "erlang",
    ex: "erlang",
    exs: "erlang",
    fs: "javascript",
    fsx: "javascript",
    fsi: "javascript",
    vb: "vb",
    v: "verilog",
    sv: "verilog",
    vh: "verilog",
    vhd: "vhdl",
    vhdl: "vhdl",
    ml: "mllike",
    mli: "mllike",
    clj: "clojure",
    cljs: "clojure",
    edn: "clojure",
    elm: "elm",
    groovy: "groovy",
    gvy: "groovy",
    gradle: "groovy",
    coffee: "coffeescript",
    litcoffee: "coffeescript",
    pug: "pug",
    jade: "pug",
    styl: "stylus",
    proto: "protobuf",
    graphql: "javascript",
    gql: "javascript",
    cmake: "cmake",
    diff: "diff",
    patch: "diff",
    log: "shell",
    txt: "shell",
    tex: "stex",
    bib: "stex",
};

function getMode(ext: string): string {
    return EXT_TO_MODE[ext.toLowerCase()] || "shell";
}

const cmOptions = computed(() => ({
    mode: getMode(props.ext),
    readOnly: true,
    theme: settings.theme === "light" ? "eclipse" : "material-darker",
    lineNumbers: true,
    styleActiveLine: true,
    matchBrackets: true,
    lineWrapping: false,
    tabSize: 4,
    viewportMargin: Infinity,
}));

function onReady() {
    // Prevent editing cursor
    const el = document.querySelector(".code-root .CodeMirror");
    if (el) (el as HTMLElement).style.cursor = "default";
}

watch(
    () => props.code,
    (v) => {
        codeText.value = v;
    },
);

onUnmounted(() => {
    cmRef.value?.destroy();
});
</script>

<style scoped>
.code-root {
    width: 100%;
    height: 100%;
}
.code-root :deep(.CodeMirror) {
    height: 100%;
    font-family:
        "SF Mono", "Fira Code", "JetBrains Mono", "Cascadia Code", Consolas,
        monospace;
    line-height: 1.65;
}
.code-root :deep(.CodeMirror-gutters) {
    border-right: 1px solid var(--border);
    background: var(--bg-secondary);
}
.code-root :deep(.CodeMirror-linenumber) {
    color: var(--text-muted);
    opacity: 0.45;
}
</style>
