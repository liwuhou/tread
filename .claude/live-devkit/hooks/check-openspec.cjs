'use strict';
var fs = require('fs');
var path = require('path');

// Walk up from CWD to find project root (directory containing .claude/)
function findProjectRoot() {
  var dir = process.cwd();
  while (dir !== path.dirname(dir)) {
    if (fs.existsSync(path.join(dir, '.claude'))) return dir;
    dir = path.dirname(dir);
  }
  if (fs.existsSync(path.join(dir, '.claude'))) return dir;
  return null;
}

// Read stdin
var input;
try { input = fs.readFileSync(0, 'utf8'); } catch (e) { process.exit(0); }

// Parse JSON
var data;
try { data = JSON.parse(input); } catch (e) { process.exit(0); }

var toolName = data.tool_name || '';
var toolInput = data.tool_input || {};
var filePath = (toolInput.file_path || '').replace(/\\/g, '/');

// Only check Edit and Write tools
if (toolName !== 'Edit' && toolName !== 'Write') { process.exit(0); }

// Only check source files (packages/ and apps/scenes/)
if (filePath.indexOf('/packages/') === -1 && filePath.indexOf('/apps/scenes/') === -1) { process.exit(0); }

// Exclude test files and config files
if (/\.test\./.test(filePath) || /\.spec\./.test(filePath) ||
    /vitest\.config/.test(filePath) || /vite\.config/.test(filePath) ||
    /tsconfig/.test(filePath)) { process.exit(0); }

// Find project root (CWD-safe)
var root = findProjectRoot();
if (!root) { process.exit(0); }

// Check skip marker (30 min TTL)
var skipMarker = path.join(root, '.claude', '.openspec-skip');
if (fs.existsSync(skipMarker)) {
  try {
    var stat = fs.statSync(skipMarker);
    var age = (Date.now() - stat.mtime.getTime()) / 1000;
    if (age < 1800) { process.exit(0); }
    try { fs.unlinkSync(skipMarker); } catch (e) {}
  } catch (e) {}
}

// Check for active OpenSpec changes
var changesDir = path.join(root, 'openspec', 'changes');
if (fs.existsSync(changesDir)) {
  try {
    var entries = fs.readdirSync(changesDir);
    for (var i = 0; i < entries.length; i++) {
      var entryPath = path.join(changesDir, entries[i]);
      try { if (fs.statSync(entryPath).isDirectory()) { process.exit(0); } } catch (e) {}
    }
  } catch (e) {}
}

// No active change, block
console.log('⚠️ 未检测到活跃的 OpenSpec change。本项目采用 SDD 开发模式。');
console.log('');
console.log('建议先使用 OpenSpec 工作流：');
console.log('  /opsx:new        — 新建 change（从探索开始）');
console.log('  /opsx:propose    — 快速创建 change 并生成所有设计文档');
console.log('');
console.log('如果是紧急修复，可以让 agent 创建跳过标记：');
console.log('  touch .claude/.openspec-skip');
console.log('  （30分钟内不再检查）');
process.exit(2);
