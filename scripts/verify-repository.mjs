// Repository/documentation gate: no package downloads and no product execution.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const read = file => fs.readFileSync(path.join(root, file), 'utf8').replace(/^\uFEFF/, '');
const checks = [];
function check(name, action) {
  try { const detail = action(); checks.push({ name, ok: true, detail }); console.log(`PASS ${name}: ${detail}`); }
  catch (error) { checks.push({ name, ok: false, detail: error.message }); console.error(`FAIL ${name}: ${error.message}`); }
}
function assert(condition, message) { if (!condition) throw new Error(message); }
const trackedAndNew = [...new Set(execFileSync('git', ['ls-files', '-c', '-o', '--exclude-standard', '-z'], { cwd: root, encoding: 'utf8' }).split('\0').filter(Boolean))].filter(file => fs.existsSync(path.join(root,file)));
const required = ['.gitignore','.editorconfig','LICENSE','README.md','AGENTS.md','CONTRIBUTING.md','SECURITY.md','.github/PULL_REQUEST_TEMPLATE.md','.github/ISSUE_TEMPLATE/bug_report.md','.github/ISSUE_TEMPLATE/task.md','docs/architecture.md','docs/risks.md','docs/acceptance.md','docs/backlog.json','docs/backlog.md','docs/requirements.md','docs/progress.md'];

check('required governance files', () => { required.forEach(p => assert(fs.existsSync(path.join(root,p)), `Missing ${p}`)); return `${required.length} files`; });
check('official specification preserved', () => {
  const digest = crypto.createHash('sha256').update(fs.readFileSync(path.join(root, 'docs/specification-v1.0.original.md'))).digest('hex');
  assert(digest === '994a2faa0e1d9c48450c5fe2b7b88ab9e237c17054efe3a70e1ec291671f765e', 'Original specification was modified');
  return digest;
});
check('official backlog and dependency graph', () => {
  const backlog = JSON.parse(read('docs/backlog.json'));
  const official = [...read('docs/specification-v1.0.original.md').matchAll(/^\|\s*(SM-\d{3})\s*\|\s*([^|]+?)\s*\|\s*([^|]+?)\s*\|/gm)];
  assert(backlog.items.length === 28 && official.length === 28, 'Expected 28 main items');
  const map = new Map(backlog.items.map(i => [i.id,i]));
  assert(map.size === 28, 'Duplicate parent IDs');
  let subtasks = 0;
  for (let n = 1; n <= 28; n++) {
    const id = `SM-${String(n).padStart(3,'0')}`;
    const item = map.get(id), source = official.find(row => row[1] === id);
    assert(item && source && item.title === source[2].trim() && item.acceptance === source[3].trim(), `Official title/acceptance drift: ${id}`);
    assert(backlog.statuses.includes(item.status), `Invalid status: ${id}`);
    assert(item.sprint >= 1 && item.sprint <= 6, `Invalid sprint: ${id}`);
    const seenSubtasks = new Set();
    for (const sub of item.subtasks) {
      assert(sub.id.startsWith(`${id}.`) && /^SM-\d{3}\.\d{2}$/.test(sub.id) && !seenSubtasks.has(sub.id), `Invalid subtask: ${sub.id}`);
      assert(backlog.statuses.includes(sub.status), `Invalid subtask state: ${sub.id}`); seenSubtasks.add(sub.id); subtasks++;
    }
    for (const file of item.evidence) assert(fs.existsSync(path.join(root,file)), `Missing evidence for ${id}: ${file}`);
  }
  const visiting = new Set(), visited = new Set();
  function visit(id) {
    assert(map.has(id), `Unknown dependency ${id}`);
    assert(!visiting.has(id), `Dependency cycle at ${id}`);
    if (visited.has(id)) return;
    visiting.add(id); map.get(id).depends_on.forEach(visit); visiting.delete(id); visited.add(id);
  }
  map.forEach((_,id) => visit(id));
  const markdown = read('docs/backlog.md');
  map.forEach(item => assert(markdown.includes(`| ${item.id} | ${item.title} | ${item.status} |`), `Markdown backlog drift: ${item.id}`));
  return `28 original IDs, ${subtasks} subtasks, acyclic dependencies`;
});
check('Sprint 02 original and completion honesty', () => {
  const digest = crypto.createHash('sha256').update(fs.readFileSync(path.join(root,'docs/sprint02.original.md'))).digest('hex');
  assert(digest === 'c468f087db24d0271d332ff936bce4f42968f6d27dd339cc4e6c41c0020dac84', 'Sprint 02 source changed');
  const items = JSON.parse(read('docs/backlog.json')).items.filter(i=>i.sprint===2);
  if (items.some(i=>i.status==='concluido')) {
    const live=JSON.parse(read('docs/evidence/sprint02/google-live.json'));
    assert(live.success===true && live.synthetic===false, 'Real integration completion needs live evidence');
  }
  return 'source hash preserved; live success is never inferred from mocks';
});
check('Sprint 03 contract and evidence boundary', () => {
 const digest=crypto.createHash('sha256').update(fs.readFileSync(path.join(root,'docs/sprint03.original.md'))).digest('hex');
 assert(digest==='90d5fe96681b78eb50fa4f80ee086e3f220ff79eb06bb54c9aeaf527c14b2bca','Sprint 03 original changed');
 for (const file of ['docs/integrity-engine.md','docs/baselines.md','docs/formula-analysis.md','docs/apps-script-dependencies.md','docs/event-model.md']) assert(fs.existsSync(path.join(root,file)),`Missing ${file}`);
 const app=read('src-tauri/src/app.rs');for (const cmd of ['integrity_propose_scope','integrity_capture_snapshot','integrity_create_baseline','integrity_list_baselines','integrity_get_active_baseline','integrity_compare_snapshot','integrity_validate_integrity','integrity_analyze_script_dependencies','integrity_list_integrity_events','integrity_get_integrity_report','integrity_prune_history']) assert(app.includes(cmd),`Missing ${cmd}`);
 const completed=JSON.parse(read('docs/backlog.json')).items.some(i=>i.sprint===3&&i.status==='concluido');
 if(completed){const proof=JSON.parse(read('docs/evidence/sprint03/local-validation.json'));assert(proof.success===true&&proof.synthetic_scenarios===true,'Completion needs local scenario evidence');assert(typeof proof.live_mutations==='string','Real mutation coverage must be explicit');}
 return 'original preserved; Sprint 03 operations only; real and synthetic evidence separate';
});
check('Sprint 04 original and authorized additions', () => {
 const digest=crypto.createHash('sha256').update(fs.readFileSync(path.join(root,'docs/sprint04.original.md'))).digest('hex');
 assert(digest==='4ae6d43afbD53694cc1aec38b485eea584344bc5d6bc16950da637c8e58dab1f'.toLowerCase(),'Sprint 04 original changed');
 const additions=JSON.parse(read('docs/backlog.json')).additions;
 assert(additions.length===2 && additions[0].id==='SM-021A' && additions[1].id==='SM-021B','Authorized additions must remain separate from original 28 IDs');
 for (const item of additions) assert(item.sprint===4 && item.source==='docs/sprint04.original.md','Addition source invalid');
 return 'source preserved; 28 original IDs and two explicit additions';
});
check('raw prototype archive', () => {
  const manifest = JSON.parse(read('docs/evidence/repository/prototype-manifest.json'));
  for (const entry of manifest.files) {
    const file = path.resolve(root,'archive/prototype',entry.path);
    assert(file.startsWith(path.join(root,'archive/prototype') + path.sep), 'Archive path traversal');
    const digest = crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
    assert(digest === entry.sha256, `Archived original changed: ${entry.path}`);
  }
  return `${manifest.files.length} original file hashes match`;
});
check('local documentation links', () => {
  let total = 0;
  for (const file of trackedAndNew.filter(p => p.endsWith('.md') && !p.startsWith('archive/') && !p.endsWith('.prototype.md') && !p.endsWith('.original.md'))) {
    const content = read(file);
    for (const match of content.matchAll(/\[[^\]]*\]\(([^\s)]+)(?:\s+"[^"]*")?\)/g)) {
      const link = match[1].replace(/^<|>$/g,'');
      if (/^(https?:|mailto:|#)/i.test(link)) continue;
      const local = decodeURIComponent(link.split('#')[0]);
      if (!local) continue;
      const target = path.resolve(path.dirname(path.join(root,file)),local);
      assert(target.startsWith(root + path.sep), `Link leaves repository: ${file}`);
      assert(fs.existsSync(target), `Broken link: ${file} -> ${local}`); total++;
    }
  }
  return `${total} local links resolve (historical verbatim files excluded)`;
});
check('license metadata', () => {
  assert(read('LICENSE').startsWith('MIT License'), 'Missing MIT license');
  const pkg = JSON.parse(read('package.json')), lock = JSON.parse(read('package-lock.json'));
  assert(pkg.license === 'MIT' && lock.packages[''].license === 'MIT', 'npm license metadata differs');
  assert(/^license\s*=\s*"MIT"/m.test(read('src-tauri/Cargo.toml')), 'Cargo license metadata differs');
  return 'MIT applies to own package; external inventory reviewed separately';
});
check('issue and PR templates', () => {
  for (const file of ['.github/ISSUE_TEMPLATE/bug_report.md','.github/ISSUE_TEMPLATE/task.md']) {
    assert(/^---\r?\nname: .+\r?\nabout: .+\r?\ntitle: .+\r?\nlabels: .+\r?\nassignees: .+\r?\n---/m.test(read(file)), `Invalid template metadata: ${file}`);
  }
  assert(read('.github/PULL_REQUEST_TEMPLATE.md').includes('Critérios de aceite'), 'PR template lacks acceptance');
  return 'two issue templates and PR acceptance/evidence sections';
});
check('Sprint 04 scope and modular structure', () => {
  for (const directory of ['src/components','src/pages','src/hooks','src/services','src/types','src-tauri/src/database','src-tauri/src/auth','src-tauri/src/google','src-tauri/src/integrity','src-tauri/src/monitoring','src-tauri/src/notifications','src-tauri/migrations']) {
    assert(fs.statSync(path.join(root,directory)).isDirectory(), `Missing module ${directory}`);
  }
  const app = read('src-tauri/src/app.rs');
  assert((app.match(/#\[tauri::command\]/g) || []).length === 43 && app.includes('foundation_get_dashboard') && app.includes('foundation_save_settings'), 'Unexpected native command surface');
  const cargo = read('src-tauri/Cargo.toml');
  assert(!/^tauri-plugin-notification\s*=/m.test(cargo), 'Future notification dependency activated');
  for (const command of ['integration_get_state','integration_configure','integration_connect','integration_disconnect','integration_revoke','integration_test_sheet','integration_test_script']) assert(app.includes(command), `Missing command ${command}`);
  assert(!/pub mod (oauth|store|engine|fixtures|analysis);/.test(read('src-tauri/src/lib.rs')), 'Prototype modules exported');
  for (const file of trackedAndNew.filter(p => /^(src|src-tauri\/src)\//.test(p) && /\.(rs|tsx?|css)$/.test(p))) {
    const content = read(file);
    assert(!/(?:from\s*["']|include!\s*\(\s*["'])[^"']*archive\//.test(content), `Active import of archive: ${file}`);
  }
  return 'foundation + Google + operational management, future scheduler and notifications isolated';
});
check('active source encoding', () => {
  const source=trackedAndNew.filter(p=>/^(src|src-tauri\/src)\//.test(p) && /\.(tsx?|rs|css)$/.test(p));
  for (const file of source) assert(!read(file).includes('\uFFFD'), `Invalid Unicode replacement in ${file}`);
  return `${source.length} source files without replacement characters`;
});
check('known credential patterns', () => {
  const patterns = [/-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/,/AIza[0-9A-Za-z_-]{30,}/,/gh[pousr]_[A-Za-z0-9]{30,}/,/github_pat_[A-Za-z0-9_]{40,}/,/GOCSPX-[A-Za-z0-9_-]{20,}/,/ya29\.[A-Za-z0-9_-]{25,}/,/AKIA[0-9A-Z]{16}/];
  let count = 0;
  for (const file of trackedAndNew) {
    const absolute = path.join(root,file);
    if (!fs.existsSync(absolute) || !fs.statSync(absolute).isFile()) continue;
    const data = fs.readFileSync(absolute);
    let text;
    if (data[0] === 0xff && data[1] === 0xfe) text = data.toString('utf16le');
    else if (data.includes(0)) continue;
    else text = data.toString('utf8');
    patterns.forEach((regex,index) => assert(!regex.test(text), `Credential pattern ${index+1} in ${file}; value not printed`)); count++;
  }
  return `${count} text files scanned; no known credential signatures (not a complete secret audit)`;
});
check('completion evidence matches sprint status', () => {
  const items = JSON.parse(read('docs/backlog.json')).items;
  const complete = items.filter(item => item.sprint === 1).every(item => item.status === 'concluido');
  if (!complete) return 'Sprint 01 still in progress; native acceptance is not inferred';
  for (const item of items.filter(i => i.sprint === 1)) assert(item.evidence.length > 0, `Completed item lacks evidence: ${item.id}`);
  const qa = JSON.parse(read('docs/evidence/sprint01/native-qa.json'));
  assert(qa.success === true && qa.renderer_origin === 'http://tauri.localhost' && qa.desktop_bridge === true, 'Native runtime not established');
  assert(['dashboard','sheets','history','settings'].every(page => qa.navigation.includes(page)), 'Native navigation not established');
  assert(qa.persistence.saved_interval === 120 && qa.persistence.after_restart_interval === 120 && qa.persistence.final_interval === 60, 'Native persistence not established');
  assert(qa.diagnostics.final_debug_port_closed === true && qa.cloud_access === false, 'Diagnostics or scope gate incomplete');
  assert(items.filter(i => i.sprint > 4).every(i => i.status !== 'concluido'), 'Future sprint completion is unauthorized');
  return 'Completed foundation items have native evidence; future sprints remain uncompleted';
});
const success = checks.every(item => item.ok);
const reportPath = path.join(root,'docs/evidence/sprint04/repository-verification.json');
fs.mkdirSync(path.dirname(reportPath), { recursive:true });
fs.writeFileSync(reportPath, JSON.stringify({ success, scope:'Sprint 04 repository/documentation only; no product or remote tests implied', checks },null,2)+'\n');
process.exitCode = success ? 0 : 1;
