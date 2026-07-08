#!/usr/bin/env node

/**
 * Build tread and install to ~/.cargo/bin/
 *
 * Usage: node build-and-install.js
 */

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// Configuration
const BINARY_NAME = 'tread';
const DEST_DIR = path.join(process.env.HOME, '.cargo', 'bin');
const DEST_PATH = path.join(DEST_DIR, BINARY_NAME);

console.log('🔨 Building tread...\n');

try {
  // Build release
  execSync('cargo build --release', {
    stdio: 'inherit',
    cwd: __dirname
  });

  console.log('\n✅ Build successful!\n');

  // Find the built binary
  const sourcePath = path.join(__dirname, 'target', 'release', BINARY_NAME);

  if (!fs.existsSync(sourcePath)) {
    console.error(` Binary not found at: ${sourcePath}`);
    process.exit(1);
  }

  // Ensure destination directory exists
  if (!fs.existsSync(DEST_DIR)) {
    console.log(`📁 Creating directory: ${DEST_DIR}`);
    fs.mkdirSync(DEST_DIR, { recursive: true });
  }

  // Copy binary
  console.log(` Installing to: ${DEST_PATH}`);
  fs.copyFileSync(sourcePath, DEST_PATH);

  // Make executable
  fs.chmodSync(DEST_PATH, 0o755);

  console.log('\n✅ Installation complete!');
  console.log(`\nYou can now run: ${BINARY_NAME} <file.md|url> [-i|--interactive]\n`);

} catch (error) {
  console.error('\n❌ Build failed!');
  process.exit(1);
}
