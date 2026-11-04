// Generate a conservative notice inventory from the locked, installed inputs.
// No credentials, product database, browser state or network access is used.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const output = path.join(root, 'src-tauri/resources/THIRD-PARTY-NOTICES.txt');
const inventory = path.join(root, 'docs/evidence/sprint06/distribution-licenses.json');
const digest = text => crypto.createHash('sha256').update(text).digest('hex');
const readJson = file => JSON.parse(fs.readFileSync(file, 'utf8').replace(/^\uFEFF/, ''));
const cargo = process.platform === 'win32' ? 'cargo.exe' : 'cargo';
const result = spawnSync(cargo, ['metadata', '--manifest-path', 'src-tauri/Cargo.toml', '--format-version', '1', '--locked', '--offline', '--filter-platform', 'x86_64-pc-windows-msvc', '--features', 'custom-protocol'], { cwd: root, encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 });
if (result.error || result.status !== 0) {
  throw new Error('Locked offline Cargo metadata unavailable; run the reviewed environment setup and fetch the locked dependencies first.');
}
const metadata = JSON.parse(result.stdout);
const nodes = new Map(metadata.resolve.nodes.map(node => [node.id, node]));
const packages = new Map(metadata.packages.map(pkg => [pkg.id, pkg]));
const selected = new Set();
function walk(id) {
  if (selected.has(id)) return;
  selected.add(id);
  for (const dependency of nodes.get(id)?.deps || []) {
    if (dependency.dep_kinds.some(kind => kind.kind !== 'dev')) walk(dependency.pkg);
  }
}
walk(metadata.resolve.root);

const entries = [];
const noticeName = /^(?:licen[cs]e|copying|notice|copyright)(?:[._-]|$)/i;
function noticeFiles(directory, explicit) {
  const files = [];
  for (const item of fs.readdirSync(directory, { withFileTypes: true })) {
    if (item.isFile() && noticeName.test(item.name)) files.push(item.name);
    // Some Rust packages store the selected license texts in a licenses folder.
    if (item.isDirectory() && /^licen[cs]es?$/i.test(item.name)) {
      for (const child of fs.readdirSync(path.join(directory, item.name), { withFileTypes: true })) {
        if (child.isFile()) files.push(path.join(item.name, child.name));
      }
    }
  }
  if (explicit) {
    const absolute = path.resolve(directory, explicit);
    if (!absolute.startsWith(directory + path.sep)) throw new Error('License file leaves package directory');
    if (fs.existsSync(absolute)) files.push(path.relative(directory, absolute));
  }
  return [...new Set(files)].sort().map(file => {
    const text = fs.readFileSync(path.join(directory, file), 'utf8').replace(/\r\n/g, '\n');
    return { file: file.replaceAll('\\', '/'), sha256: digest(text), text };
  });
}
for (const id of selected) {
  if (id === metadata.resolve.root) continue;
  const pkg = packages.get(id);
  const directory = path.dirname(pkg.manifest_path);
  const files = noticeFiles(directory, pkg.license_file);
  entries.push({ ecosystem: 'Rust', name: pkg.name, version: pkg.version, license: pkg.license || 'UNKNOWN', source: `https://crates.io/api/v1/crates/${pkg.name}/${pkg.version}/download`, notices: files });
}

// The production lockfile set is deliberately conservative: it includes build
// dependencies declared as production packages, not just tree-shaken JS bytes.
const lock = readJson(path.join(root, 'package-lock.json'));
for (const [location, pkg] of Object.entries(lock.packages)) {
  if (!location || pkg.dev || pkg.link) continue;
  const directory = path.join(root, location);
  if (!fs.existsSync(path.join(directory, 'package.json'))) {
    if (pkg.optional) continue; // Non-Windows optional binaries are not installed.
    throw new Error(`Required npm package unavailable: ${location}`);
  }
  const actual = readJson(path.join(directory, 'package.json'));
  if (actual.version !== pkg.version) throw new Error(`npm lock mismatch: ${location}`);
  entries.push({ ecosystem: 'npm', name: actual.name, version: actual.version, license: typeof actual.license === 'string' ? actual.license : 'UNKNOWN', source: pkg.resolved || `https://www.npmjs.com/package/${actual.name}/v/${actual.version}`, notices: noticeFiles(directory) });
}
entries.sort((a, b) => `${a.ecosystem}:${a.name}:${a.version}`.localeCompare(`${b.ecosystem}:${b.name}:${b.version}`));
const missing = entries.filter(entry => entry.license === 'UNKNOWN' || entry.notices.length === 0).map(({ ecosystem, name, version, license }) => ({ ecosystem, name, version, license }));
const header = `Sheets Monitor — Third-party notices\n\nThis file preserves license/copyright notices supplied with the locked dependencies.\nOwn application code is MIT; third-party code retains its respective license.\nInventory is a conservative Windows Rust normal/build dependency and npm production set, not proof that every listed package is linked in the installer.\nDev-only test dependencies and noninstalled optional npm platform packages are excluded.\nFor an OR expression, the supplied alternative license texts are preserved; no relicensing is implied.\nCorresponding source for each unmodified Rust package, including MPL-covered packages, is available at its exact versioned source URL below.\nThe application does not modify third-party package source.\nWebView2 is a separately licensed Microsoft runtime downloaded by the official installer when required; it is not relicensed by this file.\nThis generated inventory does not replace legal review.\n`;
const body = entries.map(entry => `\n${'='.repeat(72)}\n${entry.ecosystem}: ${entry.name} ${entry.version}\nLicense expression: ${entry.license}\nCorresponding source/package: ${entry.source}\n${entry.notices.map(file => `\n--- ${file.file} ---\n${file.text}`).join('\n')}`).join('\n');
fs.mkdirSync(path.dirname(inventory), { recursive: true });
fs.writeFileSync(inventory, JSON.stringify({ target: 'x86_64-pc-windows-msvc', scope: 'conservative normal/build Rust closure and installed production npm set', package_count: entries.length, missing, packages: entries.map(entry => ({ ...entry, notices: entry.notices.map(({ text, ...notice }) => notice) })) }, null, 2) + '\n');
if (missing.length) throw new Error(`${missing.length} packages lack license metadata/text; see distribution-licenses.json. Notices not approved.`);
fs.mkdirSync(path.dirname(output), { recursive: true });
fs.writeFileSync(output, header + body + '\n');
console.log(`PASS third-party notices: ${entries.length} packages; sha256=${digest(fs.readFileSync(output))}`);
