// Copies the version from package.json (the single source of truth) into
// src-tauri/Cargo.toml. Runs automatically during `npm version <x.y.z>`.
// tauri.conf.json already reads package.json directly.

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
console.log(`Version ${version} synced to Cargo.toml`);
