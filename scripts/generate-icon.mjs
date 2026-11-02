// Reproducible app artwork: simple local grid/monitor mark. No external images.
import { deflateSync } from 'node:zlib';
import { mkdirSync, writeFileSync } from 'node:fs';
const n = 256;
const pixels = Buffer.alloc(n * (1 + 4 * n));
for (let y = 0; y < n; y++) for (let x = 0; x < n; x++) {
  const inside = x >= 24 && x < 232 && y >= 24 && y < 232;
  const grid = inside && x >= 55 && x < 201 && y >= 75 && y < 195 && (Math.abs(x - 103) < 3 || Math.abs(x - 151) < 3 || Math.abs(y - 115) < 3 || Math.abs(y - 155) < 3);
  const head = inside && x >= 55 && x < 201 && y >= 52 && y < 66;
  const color = grid || head ? [219, 250, 237, 255] : inside ? [24, 120, 87, 255] : [15, 36, 32, 255];
  pixels.set(color, y * (1 + 4 * n) + 1 + 4 * x);
}
function crc(data) { let c = 0xffffffff; for (const b of data) { c ^= b; for (let k = 0; k < 8; k++) c = (c >>> 1) ^ ((c & 1) ? 0xedb88320 : 0); } return (c ^ 0xffffffff) >>> 0; }
function chunk(type, data) { const t = Buffer.from(type); const h = Buffer.alloc(4); h.writeUInt32BE(data.length); const end = Buffer.alloc(4); end.writeUInt32BE(crc(Buffer.concat([t, data]))); return Buffer.concat([h, t, data, end]); }
const ihdr = Buffer.alloc(13); ihdr.writeUInt32BE(n, 0); ihdr.writeUInt32BE(n, 4); ihdr[8] = 8; ihdr[9] = 6;
const png = Buffer.concat([Buffer.from([137,80,78,71,13,10,26,10]), chunk('IHDR', ihdr), chunk('IDAT', deflateSync(pixels)), chunk('IEND', Buffer.alloc(0))]);
const ico = Buffer.alloc(22); ico.writeUInt16LE(1, 2); ico.writeUInt16LE(1, 4); ico.writeUInt16LE(1, 10); ico.writeUInt16LE(32, 12); ico.writeUInt32LE(png.length, 14); ico.writeUInt32LE(22, 18);
mkdirSync('src-tauri/icons', { recursive: true }); writeFileSync('src-tauri/icons/icon.png', png); writeFileSync('src-tauri/icons/icon.ico', Buffer.concat([ico, png]));
