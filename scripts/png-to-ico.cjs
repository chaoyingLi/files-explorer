// Convert PNG to ICO for Windows resource embedding
const fs = require("fs");
const path = require("path");
const toIco = require("to-ico");

const files = [
  fs.readFileSync(path.join(__dirname, "..", "src-tauri", "icons", "icon.png")),
];

toIco(files, { resize: true, sizes: [32] })
  .then((buf) => {
    const outPath = path.join(
      __dirname,
      "..",
      "src-tauri",
      "icons",
      "icon.ico",
    );
    fs.writeFileSync(outPath, buf);
    console.log(`Created icon.ico (${buf.length} bytes)`);
  })
  .catch((err) => {
    console.error("Error:", err.message);
    process.exit(1);
  });
