// utils/encoding.rs
// Encoding helpers (base64, text detection, charset).

/// Known text/code extensions — always preview as text.
pub fn is_known_text_ext(ext: &str) -> bool {
    matches!(
        ext,
        "txt"
            | "md"
            | "mdx"
            | "log"
            | "ini"
            | "cfg"
            | "conf"
            | "env"
            | "yml"
            | "yaml"
            | "toml"
            | "json"
            | "jsonc"
            | "xml"
            | "svg"
            | "html"
            | "htm"
            | "xhtml"
            | "css"
            | "scss"
            | "sass"
            | "less"
            | "styl"
            | "js"
            | "mjs"
            | "cjs"
            | "jsx"
            | "ts"
            | "mts"
            | "cts"
            | "tsx"
            | "vue"
            | "svelte"
            | "astro"
            | "py"
            | "pyi"
            | "pyx"
            | "rs"
            | "go"
            | "java"
            | "kt"
            | "kts"
            | "scala"
            | "groovy"
            | "c"
            | "h"
            | "cpp"
            | "cxx"
            | "hpp"
            | "hxx"
            | "cc"
            | "hh"
            | "inl"
            | "cs"
            | "fs"
            | "fsx"
            | "vb"
            | "swift"
            | "m"
            | "mm"
            | "dart"
            | "rb"
            | "php"
            | "pl"
            | "pm"
            | "lua"
            | "r"
            | "hs"
            | "erl"
            | "hrl"
            | "ex"
            | "exs"
            | "sh"
            | "bash"
            | "zsh"
            | "fish"
            | "bat"
            | "cmd"
            | "ps1"
            | "psm1"
            | "psd1"
            | "sql"
            | "graphql"
            | "gql"
            | "proto"
            | "prisma"
            | "tf"
            | "tfvars"
            | "hcl"
            | "makefile"
            | "cmake"
            | "gradle"
            | "diff"
            | "patch"
            | "dockerfile"
            | "ignore"
            | "gitignore"
            | "editorconfig"
            | "properties"
            | "tex"
            | "sty"
            | "cls"
            | "bib"
            | "rst"
            | "clj"
            | "cljs"
            | "edn"
            | "coffee"
            | "litcoffee"
            | "zig"
            | "nim"
            | "v"
            | "vh"
            | "sv"
            | "vhd"
            | "sol"
            | "ml"
            | "mli"
            | "pug"
            | "jade"
            | "asm"
            | "s"
            | "S"
    )
}

/// Known binary extensions — skip text detection entirely.
pub fn is_known_binary_ext(ext: &str) -> bool {
    matches!(
        ext,
        "pdf"
            | "zip"
            | "7z"
            | "rar"
            | "tar"
            | "gz"
            | "tgz"
            | "bz2"
            | "tbz2"
            | "xz"
            | "txz"
            | "png"
            | "jpg"
            | "jpeg"
            | "gif"
            | "webp"
            | "bmp"
            | "svg"
            | "ico"
            | "mp3"
            | "mp4"
            | "avi"
            | "mov"
            | "mkv"
            | "wav"
            | "flac"
            | "ttf"
            | "otf"
            | "woff"
            | "woff2"
            | "eot"
            | "exe"
            | "dll"
            | "so"
            | "dylib"
            | "wasm"
            | "class"
            | "jar"
            | "war"
            | "pyc"
            | "pyo"
            | "docx"
            | "xlsx"
            | "pptx"
            | "doc"
            | "xls"
            | "ppt"
            | "iso"
            | "dmg"
            | "deb"
            | "rpm"
    )
}

/// Check if the first N bytes look like valid UTF-8 text.
pub fn is_probably_text(data: &[u8]) -> bool {
    if data.is_empty() {
        return true;
    }
    let check_len = data.len().min(8192);
    let slice = &data[..check_len];
    let mut nulls = 0usize;
    let mut controls = 0usize;
    for &b in slice.iter() {
        if b == 0 {
            nulls += 1;
        } else if b < 0x08 || (b > 0x0D && b < 0x20) {
            controls += 1;
        }
    }
    if nulls as f64 > check_len as f64 * 0.005 {
        return false;
    }
    if controls as f64 > check_len as f64 * 0.1 {
        return false;
    }
    if std::str::from_utf8(slice).is_err() {
        let printable = slice
            .iter()
            .filter(|&&b| b >= 0x20 && b < 0x7F || b == b'\n' || b == b'\r' || b == b'\t')
            .count();
        if (printable as f64) < check_len as f64 * 0.85 {
            return false;
        }
    }
    true
}

/// BOM + chardetng-based encoding detection, decode bytes to UTF-8 String.
///
/// Detection order:
///   1. Fast path — already valid UTF-8
///   2. BOM detection (UTF-16 LE/BE, UTF-8 with BOM)
///   3. chardetng statistical detection (GBK, Shift-JIS, Latin-1, etc.)
///   4. encoding_rs transcoding → UTF-8
///   5. Last resort — `String::from_utf8_lossy` (original behaviour)
pub fn decode_to_utf8(data: &[u8]) -> String {
    // ── Fast path: already valid UTF-8 ──
    if let Ok(s) = std::str::from_utf8(data) {
        return s.to_string();
    }

    // ── BOM detection ──
    // UTF-16 LE: FF FE
    if data.len() >= 2 && data[0] == 0xFF && data[1] == 0xFE {
        if let Ok(s) = decode_utf16le(&data[2..]) {
            return s;
        }
    }
    // UTF-16 BE: FE FF
    if data.len() >= 2 && data[0] == 0xFE && data[1] == 0xFF {
        if let Ok(s) = decode_utf16be(&data[2..]) {
            return s;
        }
    }
    // UTF-8 with BOM: EF BB BF — strip BOM, re-check pure UTF-8
    let body = if data.len() >= 3 && data[0] == 0xEF && data[1] == 0xBB && data[2] == 0xBF {
        let b = &data[3..];
        if let Ok(s) = std::str::from_utf8(b) {
            return s.to_string();
        }
        b
    } else {
        data
    };

    // ── chardetng statistical encoding detection ──
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(body, true); // `last = true` — all data has been fed
    let encoding = detector.guess(None, true); // no TLD hint, allow UTF-8

    // ── Transcode to UTF-8 via encoding_rs ──
    let (cow, _encoding_used, _had_errors) = encoding.decode(body);
    if !cow.is_empty() || body.is_empty() {
        cow.to_string()
    } else {
        // Last resort: original lossy behaviour
        String::from_utf8_lossy(data).to_string()
    }
}

/// Decode slices of UTF-16 LE bytes (BOM already stripped) into a String.
fn decode_utf16le(bytes: &[u8]) -> Result<String, ()> {
    if bytes.len() % 2 != 0 {
        return Err(());
    }
    let u16s: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    String::from_utf16(&u16s).map_err(|_| ())
}

/// Decode slices of UTF-16 BE bytes (BOM already stripped) into a String.
fn decode_utf16be(bytes: &[u8]) -> Result<String, ()> {
    if bytes.len() % 2 != 0 {
        return Err(());
    }
    let u16s: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect();
    String::from_utf16(&u16s).map_err(|_| ())
}

/// Safely truncate a string to at most `max_chars` chars at a valid UTF-8 boundary.
pub fn truncate_to_chars(s: &str, max_chars: usize) -> &str {
    let mut char_count = 0;
    for (i, _) in s.char_indices() {
        if char_count >= max_chars {
            return &s[..i];
        }
        char_count += 1;
    }
    s
}
