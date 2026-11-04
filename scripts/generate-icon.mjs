// Reproducible brand icon generator aligned with the Sheets Monitor visual identity:
// dark forest-green rounded tile (#15362b -> #112b21), warm off-white -5deg badge (#f1ecdf),
// and crisp horizontal pulse/activity mark (#15362b), supersampled for all Windows DPIs.
import { deflateSync } from "node:zlib";
import { mkdirSync, writeFileSync } from "node:fs";

const BG_TOP = [22, 55, 43]; // #16372b
const BG_BOTTOM = [17, 43, 33]; // #112b21
const TILE_BORDER = [40, 82, 65]; // subtle edge highlight
const BADGE_COLOR = [242, 236, 224]; // #f2ece0 (Imagem 01 graphical mark)
const WAVE_COLOR = [21, 54, 43]; // #15362b

// Normalized pulse waveform matching Lucide Activity path inside [0..1, 0..1] badge space
const WAVE_POINTS = [
  [0.18, 0.5],
  [0.32, 0.5],
  [0.42, 0.24],
  [0.57, 0.77],
  [0.67, 0.5],
  [0.82, 0.5],
];

function clamp01(v) {
  return v < 0 ? 0 : v > 1 ? 1 : v;
}

function lerp(a, b, t) {
  return a + (b - a) * t;
}

function sdRoundedBox(px, py, bx, by, r) {
  const qx = Math.abs(px) - bx + r;
  const qy = Math.abs(py) - by + r;
  const ox = Math.max(qx, 0);
  const oy = Math.max(qy, 0);
  return Math.hypot(ox, oy) + Math.min(Math.max(qx, qy), 0) - r;
}

function distToSegment(px, py, ax, ay, bx, by) {
  const vx = bx - ax;
  const vy = by - ay;
  const wx = px - ax;
  const wy = py - ay;
  const len2 = vx * vx + vy * vy;
  const t = len2 > 0 ? clamp01((wx * vx + wy * vy) / len2) : 0;
  return Math.hypot(px - (ax + t * vx), py - (ay + t * vy));
}

function distToWave(u, v) {
  let min = Infinity;
  for (let i = 0; i < WAVE_POINTS.length - 1; i++) {
    const [ax, ay] = WAVE_POINTS[i];
    const [bx, by] = WAVE_POINTS[i + 1];
    const d = distToSegment(u, v, ax, ay, bx, by);
    if (d < min) min = d;
  }
  return min;
}

function renderRgba(size) {
  const sub = size <= 32 ? 6 : size <= 64 ? 4 : 3;
  const totalSamples = sub * sub;
  const raw = Buffer.alloc(size * (1 + 4 * size));
  const angle = (-5 * Math.PI) / 180;
  const cosA = Math.cos(-angle);
  const sinA = Math.sin(-angle);

  // At very small sizes (16-24px), enlarge the graphical mark slightly for maximum legibility
  const markScale = size <= 24 ? 1.12 : size <= 32 ? 1.06 : 1.0;
  const badgeHalfW = 0.255 * markScale;
  const badgeHalfH = 0.31 * markScale;
  const badgeRadius = 0.082 * markScale;
  const strokeRadius = (size <= 24 ? 0.026 : size <= 32 ? 0.023 : 0.0205) * markScale;

  const tileHalf = 0.465;
  const tileRadius = 0.2;
  const borderWidth = 1.1 / size;

  for (let y = 0; y < size; y++) {
    const rowOffset = y * (1 + 4 * size);
    raw[rowOffset] = 0; // PNG filter: None
    for (let x = 0; x < size; x++) {
      let rAcc = 0;
      let gAcc = 0;
      let bAcc = 0;
      let aAcc = 0;

      for (let sy = 0; sy < sub; sy++) {
        const ny = (y + (sy + 0.5) / sub) / size - 0.5;
        const gradT = clamp01(ny + 0.5);
        const bgR = lerp(BG_TOP[0], BG_BOTTOM[0], gradT);
        const bgG = lerp(BG_TOP[1], BG_BOTTOM[1], gradT);
        const bgB = lerp(BG_TOP[2], BG_BOTTOM[2], gradT);

        for (let sx = 0; sx < sub; sx++) {
          const nx = (x + (sx + 0.5) / sub) / size - 0.5;
          const dTile = sdRoundedBox(nx, ny, tileHalf, tileHalf, tileRadius);
          if (dTile > 0) continue;

          let pr = bgR;
          let pg = bgG;
          let pb = bgB;

          if (dTile > -borderWidth && size >= 32) {
            const bt = 0.45;
            pr = lerp(pr, TILE_BORDER[0], bt);
            pg = lerp(pg, TILE_BORDER[1], bt);
            pb = lerp(pb, TILE_BORDER[2], bt);
          }

          // Rotate sample into tilted badge coordinates (-5deg badge)
          const rx = nx * cosA - ny * sinA;
          const ry = nx * sinA + ny * cosA;
          const dBadge = sdRoundedBox(rx, ry, badgeHalfW, badgeHalfH, badgeRadius);

          if (dBadge <= 0) {
            pr = BADGE_COLOR[0];
            pg = BADGE_COLOR[1];
            pb = BADGE_COLOR[2];

            // Wave stays horizontal in world space, centered inside the badge
            const u = nx / (badgeHalfW * 2) + 0.5;
            const v = ny / (badgeHalfH * 2) + 0.5;
            if (distToWave(u, v) * (badgeHalfW * 2) <= strokeRadius) {
              pr = WAVE_COLOR[0];
              pg = WAVE_COLOR[1];
              pb = WAVE_COLOR[2];
            }
          }

          rAcc += pr;
          gAcc += pg;
          bAcc += pb;
          aAcc += 255;
        }
      }

      const pxOffset = rowOffset + 1 + 4 * x;
      if (aAcc === 0) {
        raw[pxOffset] = 0;
        raw[pxOffset + 1] = 0;
        raw[pxOffset + 2] = 0;
        raw[pxOffset + 3] = 0;
      } else {
        const covered = aAcc / 255;
        raw[pxOffset] = Math.round(rAcc / covered);
        raw[pxOffset + 1] = Math.round(gAcc / covered);
        raw[pxOffset + 2] = Math.round(bAcc / covered);
        raw[pxOffset + 3] = Math.round(aAcc / totalSamples);
      }
    }
  }
  return raw;
}

function crc32(data) {
  let c = 0xffffffff;
  for (const b of data) {
    c ^= b;
    for (let k = 0; k < 8; k++) c = (c >>> 1) ^ (c & 1 ? 0xedb88320 : 0);
  }
  return (c ^ 0xffffffff) >>> 0;
}

function pngChunk(type, data) {
  const t = Buffer.from(type, "ascii");
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  const end = Buffer.alloc(4);
  end.writeUInt32BE(crc32(Buffer.concat([t, data])), 0);
  return Buffer.concat([len, t, data, end]);
}

function encodePng(size) {
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(size, 0);
  ihdr.writeUInt32BE(size, 4);
  ihdr[8] = 8; // 8-bit
  ihdr[9] = 6; // RGBA
  const pixels = renderRgba(size);
  return Buffer.concat([
    Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
    pngChunk("IHDR", ihdr),
    pngChunk("IDAT", deflateSync(pixels, { level: 9 })),
    pngChunk("IEND", Buffer.alloc(0)),
  ]);
}

function encodeIco(sizes) {
  const pngs = sizes.map((size) => ({ size, data: encodePng(size) }));
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0);
  header.writeUInt16LE(1, 2); // ICO type
  header.writeUInt16LE(pngs.length, 4);

  const entries = [];
  let offset = 6 + pngs.length * 16;
  for (const { size, data } of pngs) {
    const entry = Buffer.alloc(16);
    entry[0] = size >= 256 ? 0 : size;
    entry[1] = size >= 256 ? 0 : size;
    entry[2] = 0;
    entry[3] = 0;
    entry.writeUInt16LE(1, 4); // color planes
    entry.writeUInt16LE(32, 6); // bits per pixel
    entry.writeUInt32LE(data.length, 8);
    entry.writeUInt32LE(offset, 12);
    entries.push(entry);
    offset += data.length;
  }
  return Buffer.concat([header, ...entries, ...pngs.map((p) => p.data)]);
}

mkdirSync("src-tauri/icons", { recursive: true });
mkdirSync("public", { recursive: true });

const png32 = encodePng(32);
const png128 = encodePng(128);
const png256 = encodePng(256);
const png512 = encodePng(512);
const ico = encodeIco([16, 24, 32, 48, 64, 128, 256]);
const faviconIco = encodeIco([16, 32, 48]);

writeFileSync("src-tauri/icons/32x32.png", png32);
writeFileSync("src-tauri/icons/128x128.png", png128);
writeFileSync("src-tauri/icons/128x128@2x.png", png256);
writeFileSync("src-tauri/icons/icon.png", png512);
writeFileSync("src-tauri/icons/icon.ico", ico);
writeFileSync("public/favicon.ico", faviconIco);
writeFileSync("public/icon-32.png", png32);
