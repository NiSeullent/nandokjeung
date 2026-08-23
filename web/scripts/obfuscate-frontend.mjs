import { execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const webRoot = path.resolve(__dirname, '..');
const repoRoot = path.resolve(webRoot, '..');
const source = path.join(webRoot, 'src/client/handler.source.json');
const out = path.join(webRoot, 'public/handler.jp');
const tmp = path.join(webRoot, '.tmp-handler.txt');

fs.mkdirSync(path.dirname(out), { recursive: true });

const payload = fs.readFileSync(source, 'utf8').trim();
fs.writeFileSync(tmp, payload, 'utf8');

console.log('[난독증] 프론트 핸들러 Fortress 난독화…');
try {
  execSync(
    `cargo run --quiet -- obfuscate -i "${tmp}" --mode fortress -o "${out}"`,
    { cwd: repoRoot, stdio: 'inherit', env: { ...process.env } },
  );
  const size = fs.statSync(out).size;
  console.log(`[난독증] → public/handler.jp (${size} bytes)`);
} finally {
  fs.unlinkSync(tmp);
}
