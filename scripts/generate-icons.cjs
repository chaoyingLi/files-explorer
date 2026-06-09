// Generates a minimal valid 32x32 ICO file (blue folder icon)
const fs = require("fs");
const path = require("path");

function createMinimalICO() {
  const width = 32;
  const height = 32;
  const bpp = 32;

  const bihSize = 40;
  const imageSize = width * height * 4;
  const totalImageSize = bihSize + imageSize + Math.ceil(width * height / 8);

  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0);
  header.writeUInt16LE(1, 2);
  header.writeUInt16LE(1, 4);

  const entry = Buffer.alloc(16);
  entry.writeUInt8(width, 0);
  entry.writeUInt8(height, 1);
  entry.writeUInt8(0, 2);
  entry.writeUInt8(0, 3);
  entry.writeUInt16LE(1, 4);
  entry.writeUInt16LE(bpp, 6);
  entry.writeUInt32LE(totalImageSize, 8);
  entry.writeUInt32LE(22, 12);

  const bih = Buffer.alloc(bihSize);
  bih.writeUInt32LE(bihSize, 0);
  bih.writeInt32LE(width, 4);
  bih.writeInt32LE(height * 2, 8);
  bih.writeUInt16LE(1, 12);
  bih.writeUInt16LE(bpp, 14);
  bih.writeUInt32LE(0, 16);
  bih.writeUInt32LE(imageSize, 20);
  bih.writeInt32LE(0, 24);
  bih.writeInt32LE(0, 28);
  bih.writeUInt32LE(0, 32);
  bih.writeUInt32LE(0, 36);

  const pixels = Buffer.alloc(imageSize);
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const idx = (y * width + x) * 4;
      pixels[idx] = 30;
      pixels[idx + 1] = 30;
      pixels[idx + 2] = 46;
      pixels[idx + 3] = 255;

      if (y < 10 && x >= 2 && x <= 14 && y >= 2) {
        pixels[idx] = 250;
        pixels[idx + 1] = 180;
        pixels[idx + 2] = 137;
        pixels[idx + 3] = 255;
      }
      if (y >= 6 && y <= 30 && x >= 2 && x <= 30) {
        pixels[idx] = 250;
        pixels[idx + 1] = 180;
        pixels[idx + 2] = 137;
        pixels[idx + 3] = 255;
      }
    }
  }

  const andMask = Buffer.alloc(Math.ceil(width * height / 8));
  return Buffer.concat([header, entry, bih, pixels, andMask]);
}

const icoPath = path.join(__dirname, "..", "src-tauri", "icons", "icon.ico");
const icoData = createMinimalICO();
fs.writeFileSync(icoPath, icoData);
console.log(`Created icon.ico (${icoData.length} bytes)`);
