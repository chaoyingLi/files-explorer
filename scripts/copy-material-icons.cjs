// Copy Material Icon Theme SVGs to public/icons/ for static serving
const fs = require("fs");
const path = require("path");

const SRC = path.join(__dirname, "..", "node_modules", "material-icon-theme", "icons");
const DEST = path.join(__dirname, "..", "public", "icons");

if (!fs.existsSync(SRC)) {
  console.error("material-icon-theme icons not found. Run: npm install");
  process.exit(1);
}

fs.mkdirSync(DEST, { recursive: true });

const files = fs.readdirSync(SRC).filter(f => f.endsWith(".svg"));
for (const f of files) {
  fs.copyFileSync(path.join(SRC, f), path.join(DEST, f));
}
console.log(`Copied ${files.length} SVG icons to public/icons/`);
