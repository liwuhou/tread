#!/usr/bin/env bash
# generate-figma-manifest.sh
# 在指定目录下生成 .figma-manifest.json，记录所有 SVG 文件的 SHA256 哈希
# 用法: generate-figma-manifest.sh <icons-dir> <figma-file-key>

set -euo pipefail

ICONS_DIR="$1"
FIGMA_SOURCE="${2:-unknown}"

if [[ ! -d "$ICONS_DIR" ]]; then
  echo "Error: Directory $ICONS_DIR does not exist" >&2
  exit 1
fi

MANIFEST="$ICONS_DIR/.figma-manifest.json"

# 使用 node 生成 manifest（项目环境保证 node 可用）
node -e "
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const dir = '$ICONS_DIR';
const source = '$FIGMA_SOURCE';
const files = {};

fs.readdirSync(dir)
  .filter(f => f.endsWith('.svg'))
  .forEach(f => {
    const content = fs.readFileSync(path.join(dir, f));
    const hash = crypto.createHash('sha256').update(content).digest('hex');
    files[f] = hash;
  });

const manifest = {
  source: source,
  exportedAt: new Date().toISOString(),
  files: files
};

fs.writeFileSync('$MANIFEST', JSON.stringify(manifest, null, 2) + '\n');
console.log('Generated manifest: $MANIFEST');
console.log('Tracked files:', Object.keys(files).length);

// 设置 SVG 文件为只读 (OS 层面防护)
Object.keys(files).forEach(f => {
  const filePath = path.join(dir, f);
  fs.chmodSync(filePath, 0o444);
});
console.log('Set all SVG files to read-only (chmod 444)');
"
