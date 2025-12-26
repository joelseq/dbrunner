#!/usr/bin/env node

/**
 * Version Bumping Script for DBRunner
 *
 * Updates version in package.json, Cargo.toml, and tauri.conf.json
 *
 * Usage:
 *   pnpm version:bump <new-version>
 *   Example: pnpm version:bump 0.2.0
 */

import { readFileSync, writeFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const rootDir = join(__dirname, '..');

function updateVersion(newVersion) {
  // Validate version format
  if (!/^\d+\.\d+\.\d+$/.test(newVersion)) {
    console.error('Error: Version must be in format X.Y.Z (e.g., 0.2.0)');
    process.exit(1);
  }

  console.log(`Updating version to ${newVersion}...`);

  // Update package.json
  const packageJsonPath = join(rootDir, 'package.json');
  const packageJson = JSON.parse(readFileSync(packageJsonPath, 'utf8'));
  const oldVersion = packageJson.version;
  packageJson.version = newVersion;
  writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2) + '\n');
  console.log(`✓ Updated package.json (${oldVersion} → ${newVersion})`);

  // Update Cargo.toml
  const cargoTomlPath = join(rootDir, 'src-tauri', 'Cargo.toml');
  let cargoToml = readFileSync(cargoTomlPath, 'utf8');
  cargoToml = cargoToml.replace(
    /^version = ".*"$/m,
    `version = "${newVersion}"`
  );
  writeFileSync(cargoTomlPath, cargoToml);
  console.log(`✓ Updated Cargo.toml`);

  // Update tauri.conf.json
  const tauriConfPath = join(rootDir, 'src-tauri', 'tauri.conf.json');
  const tauriConf = JSON.parse(readFileSync(tauriConfPath, 'utf8'));
  tauriConf.version = newVersion;
  writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + '\n');
  console.log(`✓ Updated tauri.conf.json`);

  console.log('\nVersion bump complete!');
  console.log('\nNext steps:');
  console.log('  1. Review the changes: git diff');
  console.log('  2. Commit the changes: git add -A && git commit -m "Bump version to v' + newVersion + '"');
  console.log('  3. Create and push tag: git tag v' + newVersion + ' && git push && git push --tags');
  console.log('\nThe GitHub Action will automatically build and create a release when the tag is pushed.');
}

// Get version from command line
const newVersion = process.argv[2];

if (!newVersion) {
  console.error('Usage: node scripts/bump-version.js <new-version>');
  console.error('Example: node scripts/bump-version.js 0.2.0');
  process.exit(1);
}

updateVersion(newVersion);
