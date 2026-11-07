// Copies the version from package.json (the single source of truth) into
// src-tauri/Cargo.toml and Cargo.lock. Runs automatically during `npm version <x.y.z>`.
// tauri.conf.json already reads package.json directly.

import { execSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';

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
console.log(`Version ${version} synced to Cargo.toml and Cargo.lock`);
