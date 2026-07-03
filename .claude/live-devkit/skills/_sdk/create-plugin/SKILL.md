---
name: create-plugin
description: >
  Create a standard SDK plugin package, following the live-room-common-sdk plugin architecture.
  Must use this skill: when the user mentions "new plugin", "create SDK plugin", "create plugin",
  "add *-sdk", "write a plugin", "lottery plugin", "comment plugin", "danmaku plugin",
  "gift plugin", "answer plugin", "mic plugin", "player plugin", "interactive plugin",
  or any requirement involving live room SDK plugin development, even without explicitly saying "create skill".
  Also applicable when needing to reference standard plugin patterns, understand plugin lifecycle,
  or EventBus integration.
compatibility: Requires Bash tool to create files and directories
---

# Create SDK Plugin Package

Create a standard SDK plugin package, following the `@xiaoe/live-room-common-sdk` plugin architecture specification.

## Parameters

`$ARGUMENTS` -- Format: `<plugin-name> [target-directory]`

- **Plugin name**: kebab-case (e.g. `lottery`, `gift`, `comment`)
- **Target directory**: Optional, defaults to `packages/plugins/`

**If the user does not provide a target directory, use `packages/plugins/` as the default.**

### Ask About Demo Requirements

Before creating the plugin, ask the user which platform demos they need (multiple selection):
- WeChat mini-program
- Vue 2
- Vue 3

The user can select multiple. If the user has no special preference, select all by default.

---

## Naming Rules

| Identifier   | Rule                      | Example (name=lottery)    |
| ------------ | ------------------------- | ------------------------- |
| `pluginName` | kebab-case                | `lottery`                 |
| `PluginName` | PascalCase                | `Lottery`                 |
| `sdkDir`     | `plugin-` + pluginName    | `plugin-lottery`          |
| `npmName`    | `@xiaoe/live-` + pluginName + `-sdk` | `@xiaoe/live-lottery-sdk` |
| `baseDir`    | target directory + / + sdkDir | `packages/plugins/plugin-lottery/` |

**Default target directory**: `packages/plugins/`

---

## Creation Flow

### Step 1: Check Target Directory

If `baseDir` already exists, warn the user and ask whether to overwrite. Default is not to overwrite.

### Step 2: Create Core Files

Create the following files (templates in `references/templates.md`):

1. `package.json` - Package configuration
2. `sdk.config.js` - SDK build configuration
3. `tsconfig.json` - TypeScript configuration
4. `src/${PluginName}Plugin.ts` - Plugin main file
5. `src/index.ts` - Barrel exports

### Step 3: Create Demos

Based on the user's selected platforms, create corresponding demos under `baseDir/demo/` (templates in `references/templates.md`).

### Step 4: Update Root package.json

Add to the root `package.json` `scripts`:
```json
"build:${sdkDir}": "pnpm --filter ./${baseDir} build"
```

### Step 5: Install Dependencies and Build

Guide the user to run:
```bash
pnpm install
pnpm run build:${sdkDir}
```

---

## Plugin Standard Style Guide

For detailed rules, see `references/style-guide.md`. Key points:

### Private Fields
Use `#` private fields, not the `private` keyword.

### State Management Five-Piece Set
`#state` + `#stateSubscribers` + `subscribeState` + `getState` + `#setState`

### Event Bus
- Event constants: `XxxEvents = { INIT: 'init', UPDATE: 'xxx:update' }`
- Module augmentation: `declare module` extends `BusEventMap`

### Lifecycle
| Hook | When called | Responsibility |
|------|-------------|----------------|
| `install(ctx)` | When `sdk.use(plugin)` is called | Save ctx, register event listeners |
| `onReady(baseInfo)` | When baseInfo is loaded | Read store, make network requests |
| `onDestroy()` | When SDK is destroyed | Clean up subscriptions, timers |

---

## Rules

- **Follow standard style guide** - Do not use legacy patterns (see `references/style-guide.md`)
- **Do not add business logic beyond the template** - Only generate scaffold code, use TODO comments to prompt user to implement
- **Do not create README, tests, or other extra files** - Unless the user explicitly requests it
- **Must confirm when target directory already exists** - Default is not to overwrite
- **Prompt user to verify `demoTarget` and `typeRoots` paths** - Ensure paths are correct
- **Ask user which platform demos they need before creating** - mini-program, Vue2, Vue3, default to all

---

## Template References

All code templates are in `references/templates.md`. Copy the corresponding template when creating files, replacing variables:

- `${pluginName}` -> lowercase plugin name (e.g. `lottery`)
- `${PluginName}` -> PascalCase plugin name (e.g. `Lottery`)
- `${sdkDir}` -> `plugin-` + pluginName (e.g. `plugin-lottery`)
- `${baseDir}` -> full target path (e.g. `packages/plugins/plugin-lottery/`)
