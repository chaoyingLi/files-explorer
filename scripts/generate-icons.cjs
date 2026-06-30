// Generate all platform app icons with rounded corners
// Requires: sharp, to-ico (npm dependencies)

const fs = require("fs");
const path = require("path");
const sharp = require("sharp");
const toIco = require("to-ico");

const ICONS_DIR = path.join(__dirname, "..", "src-tauri", "icons");
const SRC_PNG = path.join(ICONS_DIR, "newlogo.png");

// Rounded corner radius (proportional to size, ~22.5% = macOS squircle style)
function roundedMaskSvg(size) {
  const r = Math.round(size * 0.225);
  return Buffer.from(
    `<svg width="${size}" height="${size}" xmlns="http://www.w3.org/2000/svg">
      <rect x="0" y="0" width="${size}" height="${size}" rx="${r}" ry="${r}" fill="white"/>
    </svg>`,
  );
}

async function pngAt(size) {
  const icon = await sharp(SRC_PNG)
    .resize(size, size)
    .ensureAlpha()
    .png()
    .toBuffer();
  const mask = await sharp(roundedMaskSvg(size))
    .resize(size, size)
    .png()
    .toBuffer();
  return sharp(icon)
    .composite([{ input: mask, blend: "dest-in" }])
    .ensureAlpha()
    .png()
    .toBuffer();
}

async function main() {
  // ── icon.png (512x512 RGBA, rounded) ──
  const base512 = await pngAt(512);
  fs.writeFileSync(path.join(ICONS_DIR, "icon.png"), base512);
  console.log("  ✓ icon.png (512x512 RGBA, rounded)");

  // ── logo.png ──
  fs.writeFileSync(path.join(ICONS_DIR, "logo.png"), base512);
  console.log("  ✓ logo.png");

  // ── icon.ico (multi-size: 16..256) ──
  const sizes = [16, 24, 32, 48, 64, 128, 256];
  const icoPngs = await Promise.all(sizes.map(pngAt));
  const icoBuf = await toIco(icoPngs, { resize: false });
  fs.writeFileSync(path.join(ICONS_DIR, "icon.ico"), icoBuf);
  console.log(`  ✓ icon.ico (${icoBuf.length} bytes, ${sizes.length} sizes)`);

  // ── logo.ico ──
  fs.writeFileSync(path.join(ICONS_DIR, "logo.ico"), icoBuf);
  console.log("  ✓ logo.ico");

  // ── icon.icns (macOS, with retina) ──
  const icnsMap = {
    16: "icp4",
    32: "icp5",
    64: "icp6",
    128: "ic07",
    256: "ic08",
    512: "ic09",
    1024: "ic10",
  };
  const retina = { icp4: "ic14", icp5: "ic11", icp6: "ic12", ic07: "ic13" };
  const icnsEntries = [];

  for (const [sz, code] of Object.entries(icnsMap)) {
    icnsEntries.push({ type: code, data: await pngAt(Number(sz)) });
  }
  for (const [baseCode, retCode] of Object.entries(retina)) {
    const baseSize =
      baseCode === "icp4"
        ? 16
        : baseCode === "icp5"
          ? 32
          : baseCode === "icp6"
            ? 64
            : 128;
    icnsEntries.push({ type: retCode, data: await pngAt(baseSize * 2) });
  }

  // Also add ic07 @2x (256) and ic08 @2x (512) for modern macOS
  icnsEntries.push({ type: "ic13", data: await pngAt(256) });
  icnsEntries.push({ type: "ic14", data: await pngAt(32) });

  fs.writeFileSync(path.join(ICONS_DIR, "icon.icns"), buildIcns(icnsEntries));
  console.log(`  ✓ icon.icns (${icnsEntries.length} entries, rounded)`);

  console.log("\n✅ All platform icons with rounded corners!");
  console.log("   Windows → icon.ico / logo.ico");
  console.log("   macOS   → icon.icns  (system applies squircle too)");
  console.log("   Linux   → icon.png / logo.png");
}

function buildIcns(entries) {
  let total = 8;
  for (const e of entries) total += 8 + e.data.length;
  const h = Buffer.alloc(8);
  h.write("icns", 0, 4, "ascii");
  h.writeUInt32BE(total, 4);
  const parts = [h];
  for (const e of entries) {
    const t = Buffer.alloc(8);
    t.write(e.type, 0, 4, "ascii");
    t.writeUInt32BE(8 + e.data.length, 4);
    parts.push(t, e.data);
  }
  return Buffer.concat(parts);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
