// Material-style icon mapping: extension → category + color
// Extracted from vscode-material-icon-theme (MIT License)

export interface IconMeta {
  category: string; // category key for icon generation
  color: string; // hex color for the icon background
  abbr: string; // 2-letter abbreviation
}

// Extension → icon metadata
export const FILE_ICON_MAP: Record<string, IconMeta> = {
  // TypeScript / JavaScript
  ts: { category: "typescript", color: "#3178C6", abbr: "TS" },
  tsx: { category: "react_ts", color: "#61DAFB", abbr: "TX" },
  js: { category: "javascript", color: "#F7DF1E", abbr: "JS" },
  cjs: { category: "javascript", color: "#F7DF1E", abbr: "JS" },
  mjs: { category: "javascript", color: "#F7DF1E", abbr: "JS" },
  jsx: { category: "react", color: "#61DAFB", abbr: "JX" },
  dts: { category: "typescript", color: "#235A97", abbr: "DT" },

  // Vue / Svelte / Astro
  vue: { category: "vue", color: "#41B883", abbr: "VU" },
  svelte: { category: "svelte", color: "#FF3E00", abbr: "SV" },
  astro: { category: "astro", color: "#FF5D01", abbr: "AS" },

  // Web
  html: { category: "html", color: "#E44D26", abbr: "HT" },
  htm: { category: "html", color: "#E44D26", abbr: "HT" },
  css: { category: "css", color: "#1572B6", abbr: "CS" },
  scss: { category: "sass", color: "#CC6699", abbr: "SC" },
  sass: { category: "sass", color: "#CC6699", abbr: "SA" },
  less: { category: "less", color: "#1D365D", abbr: "LE" },
  styl: { category: "stylus", color: "#333333", abbr: "ST" },

  // Rust
  rs: { category: "rust", color: "#DEA584", abbr: "RS" },
  ron: { category: "rust", color: "#DEA584", abbr: "RN" },

  // Go
  go: { category: "go", color: "#00ADD8", abbr: "GO" },
  mod: { category: "go_mod", color: "#00ADD8", abbr: "GM" },
  sum: { category: "go_mod", color: "#00ADD8", abbr: "GS" },

  // Python
  py: { category: "python", color: "#3776AB", abbr: "PY" },
  pyi: { category: "python", color: "#3776AB", abbr: "PI" },
  pyc: { category: "python", color: "#3776AB", abbr: "PC" },
  ipynb: { category: "jupyter", color: "#F37626", abbr: "JN" },

  // Ruby
  rb: { category: "ruby", color: "#CC342D", abbr: "RB" },
  erb: { category: "ruby", color: "#CC342D", abbr: "ER" },
  gemspec: { category: "ruby", color: "#CC342D", abbr: "GS" },

  // PHP
  php: { category: "php", color: "#777BB4", abbr: "PH" },

  // Java / Kotlin / Scala
  java: { category: "java", color: "#ED8B00", abbr: "JA" },
  jar: { category: "jar", color: "#ED8B00", abbr: "JR" },
  class: { category: "javaclass", color: "#ED8B00", abbr: "CL" },
  kt: { category: "kotlin", color: "#7F52FF", abbr: "KT" },
  scala: { category: "scala", color: "#DC322F", abbr: "SC" },

  // C / C++
  c: { category: "c", color: "#555555", abbr: "C" },
  h: { category: "h", color: "#555555", abbr: "H" },
  cpp: { category: "cpp", color: "#659AD2", abbr: "C+" },
  cc: { category: "cpp", color: "#659AD2", abbr: "CC" },
  cxx: { category: "cpp", color: "#659AD2", abbr: "CX" },
  hpp: { category: "hpp", color: "#659AD2", abbr: "H+" },
  hh: { category: "hpp", color: "#659AD2", abbr: "HH" },

  // C#
  cs: { category: "csharp", color: "#9B4F96", abbr: "C#" },

  // Swift / Objective-C
  swift: { category: "swift", color: "#F05138", abbr: "SW" },
  m: { category: "objective_c", color: "#438EFF", abbr: "OB" },
  mm: { category: "objective_cpp", color: "#438EFF", abbr: "OC" },

  // Rust-likes / System
  zig: { category: "zig", color: "#F7A41D", abbr: "ZG" },
  nim: { category: "nim", color: "#FFE953", abbr: "NM" },

  // Shell
  sh: { category: "console", color: "#4EAA25", abbr: "SH" },
  bash: { category: "console", color: "#4EAA25", abbr: "BA" },
  zsh: { category: "console", color: "#4EAA25", abbr: "ZS" },
  fish: { category: "console", color: "#4EAA25", abbr: "FH" },
  bat: { category: "console", color: "#4EAA25", abbr: "BT" },
  cmd: { category: "console", color: "#4EAA25", abbr: "CM" },
  ps1: { category: "powershell", color: "#012456", abbr: "PS" },

  // Lua
  lua: { category: "lua", color: "#00007C", abbr: "LU" },

  // R
  r: { category: "r", color: "#276DC3", abbr: "R" },
  rmd: { category: "r", color: "#276DC3", abbr: "RM" },

  // Dart / Flutter
  dart: { category: "dart", color: "#00B4AB", abbr: "DA" },

  // Elm / Haskell / PureScript
  elm: { category: "elm", color: "#60B5CC", abbr: "EL" },
  hs: { category: "haskell", color: "#5E5086", abbr: "HS" },
  lhs: { category: "haskell", color: "#5E5086", abbr: "LH" },

  // Elixir / Erlang
  ex: { category: "elixir", color: "#6E4A7E", abbr: "EX" },
  exs: { category: "elixir", color: "#6E4A7E", abbr: "ES" },
  erl: { category: "erlang", color: "#A90533", abbr: "ER" },

  // Clojure
  clj: { category: "clojure", color: "#5881D8", abbr: "CJ" },
  cljs: { category: "clojure", color: "#5881D8", abbr: "CS" },
  edn: { category: "clojure", color: "#5881D8", abbr: "ED" },

  // F#
  fs: { category: "fsharp", color: "#378BBA", abbr: "F#" },
  fsx: { category: "fsharp", color: "#378BBA", abbr: "FX" },

  // Config / Data
  json: { category: "json", color: "#FBC02D", abbr: "JS" },
  jsonc: { category: "json", color: "#FBC02D", abbr: "JC" },
  yaml: { category: "yaml", color: "#CB171E", abbr: "YM" },
  yml: { category: "yaml", color: "#CB171E", abbr: "YL" },
  toml: { category: "toml", color: "#9C4221", abbr: "TO" },
  xml: { category: "xml", color: "#E44D26", abbr: "XM" },
  csv: { category: "table", color: "#1E7145", abbr: "CV" },
  tsv: { category: "table", color: "#1E7145", abbr: "TV" },
  ini: { category: "settings", color: "#6C7086", abbr: "IN" },
  cfg: { category: "settings", color: "#6C7086", abbr: "CF" },
  conf: { category: "settings", color: "#6C7086", abbr: "CN" },
  env: { category: "tune", color: "#F9A825", abbr: "EV" },
  lock: { category: "lock", color: "#636363", abbr: "LK" },

  // Markdown / Text
  md: { category: "markdown", color: "#083FA1", abbr: "MD" },
  mdx: { category: "mdx", color: "#083FA1", abbr: "MX" },
  txt: { category: "document", color: "#4A6A9A", abbr: "TX" },
  rst: { category: "markdown", color: "#083FA1", abbr: "RS" },

  // Images
  png: { category: "image", color: "#6A3E9A", abbr: "PN" },
  jpg: { category: "image", color: "#6A3E9A", abbr: "JP" },
  jpeg: { category: "image", color: "#6A3E9A", abbr: "JE" },
  gif: { category: "image", color: "#6A3E9A", abbr: "GI" },
  webp: { category: "image", color: "#6A3E9A", abbr: "WP" },
  bmp: { category: "image", color: "#6A3E9A", abbr: "BM" },
  svg: { category: "svg", color: "#FFB13B", abbr: "SV" },
  ico: { category: "image", color: "#6A3E9A", abbr: "IC" },
  tiff: { category: "image", color: "#6A3E9A", abbr: "TF" },
  tif: { category: "image", color: "#6A3E9A", abbr: "TF" },
  heic: { category: "image", color: "#6A3E9A", abbr: "HE" },
  avif: { category: "image", color: "#6A3E9A", abbr: "AV" },
  icns: { category: "image", color: "#6A3E9A", abbr: "IN" },

  // Video
  mp4: { category: "video", color: "#6A1E7A", abbr: "M4" },
  mov: { category: "video", color: "#6A1E7A", abbr: "MO" },
  avi: { category: "video", color: "#6A1E7A", abbr: "AV" },
  mkv: { category: "video", color: "#6A1E7A", abbr: "MK" },
  webm: { category: "video", color: "#6A1E7A", abbr: "WM" },
  wmv: { category: "video", color: "#6A1E7A", abbr: "WV" },

  // Audio
  mp3: { category: "audio", color: "#8A6A1E", abbr: "M3" },
  wav: { category: "audio", color: "#8A6A1E", abbr: "WA" },
  flac: { category: "audio", color: "#8A6A1E", abbr: "FL" },
  aac: { category: "audio", color: "#8A6A1E", abbr: "AA" },
  ogg: { category: "audio", color: "#8A6A1E", abbr: "OG" },
  wma: { category: "audio", color: "#8A6A1E", abbr: "WM" },
  m4a: { category: "audio", color: "#8A6A1E", abbr: "MA" },
  opus: { category: "audio", color: "#8A6A1E", abbr: "OP" },

  // Documents
  pdf: { category: "pdf", color: "#D13438", abbr: "PD" },
  doc: { category: "word", color: "#185ABD", abbr: "WD" },
  docx: { category: "word", color: "#185ABD", abbr: "DX" },
  xls: { category: "excel", color: "#1E7145", abbr: "XL" },
  xlsx: { category: "excel", color: "#1E7145", abbr: "XX" },
  ppt: { category: "ppt", color: "#C43E1C", abbr: "PT" },
  pptx: { category: "ppt", color: "#C43E1C", abbr: "PX" },
  odt: { category: "word", color: "#185ABD", abbr: "OD" },
  ods: { category: "excel", color: "#1E7145", abbr: "OS" },
  odp: { category: "ppt", color: "#C43E1C", abbr: "OP" },

  // Archives
  zip: { category: "archive", color: "#6E5A3A", abbr: "ZP" },
  tar: { category: "archive", color: "#6E5A3A", abbr: "TA" },
  gz: { category: "archive", color: "#6E5A3A", abbr: "GZ" },
  bz2: { category: "archive", color: "#6E5A3A", abbr: "BZ" },
  xz: { category: "archive", color: "#6E5A3A", abbr: "XZ" },
  "7z": { category: "archive", color: "#6E5A3A", abbr: "7Z" },
  rar: { category: "archive", color: "#6E5A3A", abbr: "RR" },
  tgz: { category: "archive", color: "#6E5A3A", abbr: "TG" },
  zst: { category: "archive", color: "#6E5A3A", abbr: "ZS" },

  // Docker
  dockerfile: { category: "docker", color: "#2496ED", abbr: "DK" },

  // Git
  gitignore: { category: "git", color: "#F05032", abbr: "GI" },
  gitattributes: { category: "git", color: "#F05032", abbr: "GA" },

  // Licenses
  license: { category: "license", color: "#E64A19", abbr: "LI" },
  licence: { category: "license", color: "#E64A19", abbr: "LC" },

  // Fonts
  ttf: { category: "font", color: "#78909C", abbr: "TF" },
  woff: { category: "font", color: "#78909C", abbr: "WF" },
  woff2: { category: "font", color: "#78909C", abbr: "W2" },
  otf: { category: "font", color: "#78909C", abbr: "OF" },
  eot: { category: "font", color: "#78909C", abbr: "EO" },

  // Database
  sql: { category: "database", color: "#336791", abbr: "SQ" },
  sqlite: { category: "database", color: "#336791", abbr: "SL" },
  sqlite3: { category: "database", color: "#336791", abbr: "S3" },
  db: { category: "database", color: "#336791", abbr: "DB" },

  // Protocol / IDL
  proto: { category: "proto", color: "#4285F4", abbr: "PR" },
  graphql: { category: "graphql", color: "#E10098", abbr: "GQ" },
  gql: { category: "graphql", color: "#E10098", abbr: "GQ" },

  // Executables / Binary
  exe: { category: "exe", color: "#4A5A6A", abbr: "EX" },
  dll: { category: "dll", color: "#4A5A6A", abbr: "DL" },
  so: { category: "dll", color: "#4A5A6A", abbr: "SO" },
  dylib: { category: "dll", color: "#4A5A6A", abbr: "DY" },
  wasm: { category: "webassembly", color: "#654FF0", abbr: "WM" },

  // Disk images
  iso: { category: "disc", color: "#B71C1C", abbr: "IS" },
  dmg: { category: "disc", color: "#B71C1C", abbr: "DM" },

  // Misc
  log: { category: "log", color: "#9E9E9E", abbr: "LG" },
  diff: { category: "diff", color: "#26A69A", abbr: "DF" },
  patch: { category: "diff", color: "#26A69A", abbr: "PA" },
  apk: { category: "android", color: "#3DDC84", abbr: "AK" },
  ipa: { category: "applescript", color: "#808080", abbr: "IP" },

  // Graphics / Adobe
  psd: { category: "adobe_photoshop", color: "#31A8FF", abbr: "PD" },
  ai: { category: "adobe_illustrator", color: "#FF9A00", abbr: "AI" },

  // CAD / 3D
  stl: { category: "3d", color: "#607D8B", abbr: "SL" },
  obj: { category: "3d", color: "#607D8B", abbr: "OB" },
  glb: { category: "3d", color: "#607D8B", abbr: "GL" },
  gltf: { category: "3d", color: "#607D8B", abbr: "GT" },
  fbx: { category: "3d", color: "#607D8B", abbr: "FB" },

  // Subtitles
  srt: { category: "subtitles", color: "#9E9E9E", abbr: "SR" },
  vtt: { category: "subtitles", color: "#9E9E9E", abbr: "VT" },
  sub: { category: "subtitles", color: "#9E9E9E", abbr: "SU" },

  // Tauri
  tauri: { category: "tauri", color: "#FFC131", abbr: "TA" },
};

// Default icon for unknown extensions
export const DEFAULT_ICON: IconMeta = {
  category: "file",
  color: "#4A6A9A",
  abbr: "FM",
};

// ── Material-full: extension → SVG icon name (from material-icon-theme) ──
export const MATERIAL_ICON_NAMES: Record<string, string> = {
  ts: "typescript",
  tsx: "react_ts",
  js: "javascript",
  jsx: "react",
  cjs: "javascript",
  mjs: "javascript",
  dts: "typescript-def",
  vue: "vue",
  svelte: "svelte",
  astro: "astro",
  html: "html",
  htm: "html",
  css: "css",
  scss: "sass",
  sass: "sass",
  less: "less",
  rs: "rust",
  ron: "rust",
  go: "go",
  py: "python",
  pyi: "python",
  ipynb: "jupyter",
  rb: "ruby",
  erb: "ruby",
  php: "php",
  java: "java",
  jar: "jar",
  class: "javaclass",
  kt: "kotlin",
  scala: "scala",
  c: "c",
  h: "h",
  cpp: "cpp",
  cc: "cpp",
  cxx: "cpp",
  hpp: "hpp",
  hh: "hpp",
  cs: "csharp",
  swift: "swift",
  m: "objective-c",
  mm: "objective-cpp",
  zig: "zig",
  nim: "nim",
  sh: "console",
  bash: "console",
  zsh: "console",
  bat: "console",
  cmd: "console",
  ps1: "powershell",
  lua: "lua",
  r: "r",
  dart: "dart",
  elm: "elm",
  hs: "haskell",
  lhs: "haskell",
  ex: "elixir",
  exs: "elixir",
  erl: "erlang",
  clj: "clojure",
  cljs: "clojure",
  fs: "fsharp",
  fsx: "fsharp",
  json: "json",
  jsonc: "json",
  yaml: "yaml",
  yml: "yaml",
  toml: "toml",
  xml: "xml",
  csv: "table",
  ini: "settings",
  cfg: "settings",
  conf: "settings",
  env: "tune",
  lock: "lock",
  md: "markdown",
  mdx: "mdx",
  txt: "document",
  png: "image",
  jpg: "image",
  jpeg: "image",
  gif: "image",
  webp: "image",
  bmp: "image",
  svg: "svg",
  ico: "image",
  mp4: "video",
  mov: "video",
  avi: "video",
  mkv: "video",
  webm: "video",
  mp3: "audio",
  wav: "audio",
  flac: "audio",
  aac: "audio",
  ogg: "audio",
  pdf: "pdf",
  doc: "word",
  docx: "word",
  xls: "excel",
  xlsx: "excel",
  ppt: "ppt",
  pptx: "ppt",
  zip: "zip",
  tar: "zip",
  gz: "zip",
  rar: "zip",
  "7z": "zip",
  tgz: "zip",
  dockerfile: "docker",
  sql: "database",
  sqlite: "database",
  db: "database",
  proto: "proto",
  graphql: "graphql",
  gql: "graphql",
  exe: "exe",
  dll: "dll",
  so: "dll",
  wasm: "webassembly",
  iso: "disc",
  dmg: "disc",
  log: "log",
  diff: "diff",
  patch: "diff",
  apk: "android",
  psd: "adobe-photoshop",
  ai: "adobe-illustrator",
  ttf: "font",
  woff: "font",
  woff2: "font",
  otf: "font",
  stl: "3d",
  obj: "3d",
  glb: "3d",
  fbx: "3d",
  srt: "subtitles",
  vtt: "subtitles",
  tauri: "tauri",
};

export const DEFAULT_MATERIAL_ICON = "file";

export const MATERIAL_FOLDER_NAMES: Record<string, string> = {
  src: "folder-src",
  components: "folder-components",
  node_modules: "folder-node",
  assets: "folder-resource",
  dist: "folder-dist",
  build: "folder-dist",
  docs: "folder-docs",
  test: "folder-test",
  tests: "folder-test",
  lib: "folder-lib",
  vendor: "folder-lib",
  config: "folder-config",
  images: "folder-images",
  scripts: "folder-scripts",
  styles: "folder-css",
  css: "folder-css",
  github: "folder-github",
  vscode: "folder-vscode",
  docker: "folder-docker",
  public: "folder-public",
  tools: "folder-tools",
  utils: "folder-utils",
  data: "folder-database",
  database: "folder-database",
};

export const DEFAULT_MATERIAL_FOLDER = "folder";

// Folder name → icon category (simplified subset)
export const FOLDER_ICON_MAP: Record<string, { color: string; abbr: string }> =
  {
    src: { color: "#41B883", abbr: "SR" },
    source: { color: "#41B883", abbr: "SR" },
    sources: { color: "#41B883", abbr: "SR" },
    components: { color: "#C49060", abbr: "CP" },
    component: { color: "#C49060", abbr: "CP" },
    assets: { color: "#F9A825", abbr: "AS" },
    asset: { color: "#F9A825", abbr: "AS" },
    static: { color: "#F9A825", abbr: "ST" },
    public: { color: "#F9A825", abbr: "PB" },
    docs: { color: "#1E88E5", abbr: "DC" },
    doc: { color: "#1E88E5", abbr: "DC" },
    test: { color: "#E53935", abbr: "TT" },
    tests: { color: "#E53935", abbr: "TS" },
    spec: { color: "#E53935", abbr: "SP" },
    node_modules: { color: "#539E43", abbr: "NM" },
    dist: { color: "#FFB300", abbr: "DT" },
    build: { color: "#FFB300", abbr: "BD" },
    target: { color: "#FFB300", abbr: "TG" },
    lib: { color: "#5C6BC0", abbr: "LB" },
    libs: { color: "#5C6BC0", abbr: "LS" },
    vendor: { color: "#5C6BC0", abbr: "VD" },
    config: { color: "#78909C", abbr: "CF" },
    configs: { color: "#78909C", abbr: "CG" },
    images: { color: "#AB47BC", abbr: "IM" },
    image: { color: "#AB47BC", abbr: "IM" },
    img: { color: "#AB47BC", abbr: "IG" },
    icons: { color: "#AB47BC", abbr: "IC" },
    videos: { color: "#7B1FA2", abbr: "VD" },
    video: { color: "#7B1FA2", abbr: "VE" },
    audio: { color: "#D4A017", abbr: "AU" },
    music: { color: "#D4A017", abbr: "MU" },
    scripts: { color: "#00897B", abbr: "SC" },
    script: { color: "#00897B", abbr: "SP" },
    styles: { color: "#EC407A", abbr: "SS" },
    css: { color: "#1572B6", abbr: "CS" },
    scss: { color: "#CC6699", abbr: "SC" },
    utils: { color: "#78909C", abbr: "UT" },
    util: { color: "#78909C", abbr: "UT" },
    tools: { color: "#78909C", abbr: "TL" },
    github: { color: "#24292E", abbr: "GH" },
    vscode: { color: "#007ACC", abbr: "VS" },
    docker: { color: "#2496ED", abbr: "DK" },
    database: { color: "#336791", abbr: "DB" },
    data: { color: "#336791", abbr: "DT" },
  };

export const DEFAULT_FOLDER_COLOR = "#F6C23A";
export const DEFAULT_FOLDER_ABBR = "FD";
