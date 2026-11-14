// Copies the version from package.json (the single source of truth) into
// src-tauri/Cargo.toml, Cargo.lock and the winget manifests. Runs automatically during
// `npm version <x.y.z>`. tauri.conf.json already reads package.json directly.

import { execSync } from 'node:child_process';
import { readdirSync, readFileSync, writeFileSync } from 'node:fs';

const { version } = JSON.parse(readFileSync('package.json', 'utf8'));
const cargoPath = 'src-tauri/Cargo.toml';
const cargo = readFileSync(cargoPath, 'utf8');
const updated = cargo.replace(/^version = ".*"$/m, `version = "${version}"`);

if (updated === cargo && !cargo.includes(`version = "${version}"`)) {
  console.error('Could not find the version line in Cargo.toml');
  process.exit(1);
}

writeFileSync(cargoPath, updated);
// Only refreshes XPicker's own entry in Cargo.lock; dependencies stay as they are.
execSync('cargo update --workspace --offline', { cwd: 'src-tauri', stdio: 'ignore' });

// The installer's hash and release date only exist once the release is out,
// so .github/workflows/winget.yml adds those.
const wingetDir = 'packaging/winget';
for (const name of readdirSync(wingetDir)) {
  const path = `${wingetDir}/${name}`;
  const text = readFileSync(path, 'utf8')
    .replace(/^PackageVersion: .*$/m, `PackageVersion: ${version}`)
    .replace(/^(\s*InstallerUrl: .*\/download\/v)[^/]+(\/[^/]*_)[^_]+(_x64-setup\.exe)$/m, `$1${version}$2${version}$3`)
    .replace(/^(ReleaseNotesUrl: .*\/tag\/v).*$/m, `$1${version}`);
  writeFileSync(path, text);
}

console.log(`Version ${version} synced to Cargo.toml, Cargo.lock and the winget manifests`);
