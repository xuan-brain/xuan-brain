#!/usr/bin/env node
/**
 * Synchronize version from package.json to all project files
 *
 * This script reads the version from package.json and updates:
 * 1. src-tauri/Cargo.toml (Rust backend)
 * 2. src-tauri/tauri.conf.json (Tauri config)
 * 3. src/lib/version.ts (Frontend module)
 *
 * Usage: node scripts/sync-version.js
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const rootDir = path.resolve(__dirname, '..');

// Read version from package.json
const packagePath = path.join(rootDir, 'package.json');
const packageJson = JSON.parse(fs.readFileSync(packagePath, 'utf-8'));
const version = packageJson.version;

if (!version) {
  console.error('Error: No version found in package.json');
  process.exit(1);
}

console.log(`Syncing version: ${version}`);

// 1. Update Cargo.toml
const cargoPath = path.join(rootDir, 'src-tauri', 'Cargo.toml');
let cargoContent = fs.readFileSync(cargoPath, 'utf-8');
const cargoUpdated = cargoContent.replace(
  /^(version\s*=\s*)"[^"]*"/m,
  `$1"${version}"`
);
if (cargoContent !== cargoUpdated) {
  fs.writeFileSync(cargoPath, cargoUpdated);
  console.log(`Updated: ${cargoPath}`);
} else {
  console.log(`No change: ${cargoPath}`);
}

// 2. Update tauri.conf.json
const tauriConfPath = path.join(rootDir, 'src-tauri', 'tauri.conf.json');
const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf-8'));
if (tauriConf.version !== version) {
  tauriConf.version = version;
  fs.writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + '\n');
  console.log(`Updated: ${tauriConfPath}`);
} else {
  console.log(`No change: ${tauriConfPath}`);
}

// 3. Generate src/lib/version.ts
const versionTsPath = path.join(rootDir, 'src', 'lib', 'version.ts');
const versionTsContent = `/**
 * Application version - synced from package.json at build time
 * Run \`yarn sync-version\` to regenerate this file
 *
 * DO NOT EDIT MANUALLY - changes will be overwritten
 */
export const APP_VERSION = '${version}';
`;

// Ensure directory exists
const versionTsDir = path.dirname(versionTsPath);
if (!fs.existsSync(versionTsDir)) {
  fs.mkdirSync(versionTsDir, { recursive: true });
}

fs.writeFileSync(versionTsPath, versionTsContent);
console.log(`Generated: ${versionTsPath}`);

console.log('\nVersion sync complete!');
