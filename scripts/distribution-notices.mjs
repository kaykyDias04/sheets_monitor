// Build-time notice collection, never executed by the installed application.
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const metadata = JSON.parse(fs.readFileSync(path.join(root, ".tooling/cargo-metadata.json"), "utf8").replace(/^\uFEFF/, ""));
const lock = JSON.parse(fs.readFileSync(path.join(root, "package-lock.json"), "utf8"));
const inventory = [];
const sections = [];
const sysroot = execFileSync("rustc", ["--print", "sysroot"], { encoding: "utf8" }).trim();
const rustDocs = path.join(sysroot, "share/doc/rust");
fs.mkdirSync(path.join(root, "distribution/rust-notices"), { recursive: true });
fs.copyFileSync(path.join(rustDocs, "COPYRIGHT-library.html"), path.join(root, "distribution/rust-notices/COPYRIGHT-library.html"));
fs.cpSync(path.join(rustDocs, "licenses"), path.join(root, "distribution/rust-notices/licenses"), { recursive: true });
function collect(name, version, license, directory, declaredFile, extraFiles = []) {
  const candidates = fs.readdirSync(directory)
    .filter(name => /^(licen[sc]e|copying|notice|copyright)([._-]|$)/i.test(name))
    .map(name => path.join(directory, name));
  if (declaredFile) candidates.push(path.resolve(directory, declaredFile));
  candidates.push(...extraFiles);
  const files = [...new Set(candidates)].filter(file => fs.existsSync(file) && fs.statSync(file).isFile());
  inventory.push({ name, version, license: license || null, notice_files: files.map(file => path.basename(file)), complete: files.length > 0 });
  sections.push(`\n${"=".repeat(72)}\n${name} ${version}\nLicense metadata: ${license || "UNRESOLVED"}\n`);
  for (const file of files) sections.push(`\n--- ${path.basename(file)} ---\n${fs.readFileSync(file, "utf8")}\n`);
}
// Include the resolved Rust graph conservatively, including build/dev packages.
// Inclusion here does not imply every package is linked into the executable.
for (const pkg of metadata.packages.sort((a, b) => a.id.localeCompare(b.id))) {
  if (pkg.source) {
    collect(`Rust: ${pkg.name}`, pkg.version, pkg.license, path.dirname(pkg.manifest_path), pkg.license_file,
      [path.join(root, `distribution/licenses/${pkg.name === "webview2-com-sys" ? "webview2-com" : pkg.name}.txt`)]);
    sections.push(`Unmodified source package: https://crates.io/api/v1/crates/${pkg.name}/${pkg.version}/download\n`);
  }
}
for (const [relative, pkg] of Object.entries(lock.packages).sort()) {
  if (!relative || pkg.dev || pkg.optional && !fs.existsSync(path.join(root, relative))) continue;
  const name = pkg.name || relative.split("node_modules/").at(-1);
  const parentNotice = name.startsWith("@esbuild/") ? "node_modules/esbuild/LICENSE.md"
    : name.startsWith("@rollup/") ? "node_modules/rollup/LICENSE.md" : `distribution/licenses/${name}.txt`;
  collect(`npm: ${name}`, pkg.version, pkg.license, path.join(root, relative), null, [path.join(root, parentNotice)]);
}
const missing = inventory.filter(pkg => !pkg.complete || !pkg.license);
fs.mkdirSync(path.join(root, "distribution"), { recursive: true });
fs.writeFileSync(path.join(root, "distribution/THIRD-PARTY-NOTICES.txt"),
  "Sheets Monitor — third-party notices\nConservative inventory of resolved Rust packages and production npm packages.\nEach component retains its own license. Toolchains and WebView2 are not bundled here.\nRust standard-library notices are in rust-notices/COPYRIGHT-library.html and the accompanying licenses directory.\n" + sections.join(""));
fs.writeFileSync(path.join(root, "docs/evidence/sprint06/distribution-licenses.json"), JSON.stringify({
  scope: "Resolved Rust graph (including build/dev) and production npm; not proof of final binary composition",
  package_count: inventory.length, unresolved_count: missing.length, inventory,
}, null, 2) + "\n");
console.log(`Notice inventory: ${inventory.length} packages, ${missing.length} unresolved`);
if (missing.length) {
  console.error(missing.map(pkg => `${pkg.name}@${pkg.version}`).join("\n"));
  process.exitCode = 1;
}
