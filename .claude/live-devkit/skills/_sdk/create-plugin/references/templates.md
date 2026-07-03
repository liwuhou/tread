# Plugin Templates

Code templates for all plugin creation. Replace variables when using:
- `${pluginName}` -> lowercase plugin name (e.g. `lottery`)
- `${PluginName}` -> PascalCase plugin name (e.g. `Lottery`)
- `${sdkDir}` -> `plugin-` + pluginName (e.g. `plugin-lottery`)
- `${baseDir}` -> full target path (e.g. `packages/plugins/plugin-lottery/`)

---

## Core File Templates

### 1. package.json

Path: `baseDir/package.json`

```json
{
  "name": "@xiaoe/live-${pluginName}-sdk",
  "version": "1.0.0",
  "private": true,
  "main": "dist/index.js",
  "module": "dist/index.js",
  "miniprogram": "dist",
  "scripts": {
    "build": "sdk-builder build",
    "watch": "sdk-builder watch"
  },
  "peerDependencies": {
    "@xiaoe/live-room-common-sdk": "^1.1.55-alpha.13"
  },
  "devDependencies": {
    "@xiaoe/sdk-builder": "1.0.1"
  }
}
```

### 2. sdk.config.js

Path: `baseDir/sdk.config.js`

```javascript
module.exports = {
  entry: ['src/index.ts'],
  format: ['cjs'],
  dts: false,
  sourcemap: true,
  demoTarget: 'apps/wx-demo/package-watch-${pluginName}/live-${pluginName}'
}
```

> **`demoTarget` explanation**: The target directory that gets synced to during watch mode. Prompt the user to confirm the actual path is correct.

### 3. tsconfig.json

Path: `baseDir/tsconfig.json`

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "lib": ["ES2020"],
    "moduleResolution": "bundler",
    "resolveJsonModule": true,
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "declaration": true,
    "declarationDir": "./dist/types",
    "outDir": "./dist",
    "rootDir": "./src",
    "typeRoots": ["../../../node_modules/@types"]
  },
  "include": ["src/**/*.ts"],
  "exclude": ["node_modules", "dist"]
}
```

> **Notes**:
> - `typeRoots` should be adjusted based on the actual relative depth from `baseDir` to `node_modules`.
> - `moduleResolution: "bundler"` is suitable for bundlers like tsup/Vite. If using a Node.js environment, change to `"node16"`.
> - When bundling with tsup, use the `sdk.config.js` configuration. `tsconfig.json` is only used for IDE type checking and `tsc --noEmit`.

### 4. Plugin Main File

Path: `baseDir/src/${PluginName}Plugin.ts`

```typescript
import type {
  BusEventMap,
  EventBus,
  IPlugin,
  IImAdapter,
  INetworkAdapter,
  IReadonlyLiveRoomStore,
  ISDKContext,
  LiveRoomData
} from '@xiaoe/live-room-common-sdk'

// --- Event Constant Definitions --------------------------------------------------------

export const ${PluginName}Events = {
  INIT: 'init',                      // subscribeState initial sync, not through eventBus
  UPDATE: '${pluginName}:update'     // State change event
} as const

export type ${PluginName}StateEvent = (typeof ${PluginName}Events)[keyof typeof ${PluginName}Events]

// --- Type Definitions ------------------------------------------------------------------

export interface ${PluginName}State {
  // TODO: Define plugin state data structure
}

export interface ${PluginName}UpdateData extends ${PluginName}State {}

export type ${PluginName}EventMap = {
  [${PluginName}Events.UPDATE]: [state: ${PluginName}UpdateData]
}

// --- Extend Global Event Bus Types -----------------------------------------------------

declare module '@xiaoe/live-room-common-sdk' {
  interface BusEventMap extends ${PluginName}EventMap {}
}

// --- Plugin Implementation -------------------------------------------------------------

export class ${PluginName}Plugin implements IPlugin {
  readonly name = '${pluginName}'

  #store!: IReadonlyLiveRoomStore
  #eventBus!: EventBus<BusEventMap>
  #im!: IImAdapter
  #network!: INetworkAdapter

  #state: ${PluginName}State = {
    // TODO: Initialize state defaults
  }

  #stateSubscribers = new Set<(eventName: ${PluginName}StateEvent, state: ${PluginName}State) => void>()
  #unsubs: Array<() => void> = []

  // -- Lifecycle Hooks --------------------------------------------------------------------

  /**
   * Registration phase (called when sdk.use(plugin)).
   * Save references, mount long-lived event listeners.
   */
  install(ctx: ISDKContext): void {
    this.#store = ctx.store
    this.#eventBus = ctx.eventBus
    this.#im = ctx.im
    this.#network = ctx.network

    // Listen for store data changes
    this.#unsubs.push(
      this.#store.subscribe(() => {
        // TODO: Handle store changes here
      })
    )

    // Listen for IM events
    // this.#im.on('EVENT_NAME', this.#onImEvent);
    // this.#unsubs.push(() => this.#im.off('EVENT_NAME', this.#onImEvent));
  }

  /**
   * SDK is ready, baseInfo has been loaded (store has been patched).
   * Read initial data and make plugin-specific network requests here.
   */
  onReady(_baseInfo: LiveRoomData): void {
    // TODO: this.#initData();
  }

  /**
   * SDK is destroyed, release plugin resources.
   * PluginManager calls in reverse registration order, ensuring safe dependency relationships.
   */
  onDestroy(): void {
    this.#unsubs.forEach(fn => fn())
    this.#unsubs = []
  }

  // -- Public API -------------------------------------------------------------------------

  /**
   * Subscribe to plugin state changes, immediately invokes callback with current state (for component initialization).
   * Returns an unsubscribe function.
   */
  subscribeState(callback: (eventName: ${PluginName}StateEvent, state: ${PluginName}State) => void): () => void {
    this.#stateSubscribers.add(callback)
    callback(${PluginName}Events.INIT, this.#state)
    return () => {
      this.#stateSubscribers.delete(callback)
    }
  }

  /**
   * Get current state snapshot.
   */
  getState(): ${PluginName}State {
    return this.#state
  }

  // -- Internal Methods -------------------------------------------------------------------

  #setState(eventName: ${PluginName}StateEvent, partialState: Partial<${PluginName}State>): void {
    this.#state = { ...this.#state, ...partialState }
    // Notify UI subscribers
    this.#stateSubscribers.forEach(fn => fn(eventName, this.#state))
    // Notify other plugins ('init' is only used for subscribeState initial sync, not through eventBus)
    if (eventName !== ${PluginName}Events.INIT) {
      this.#eventBus.emit(eventName, this.#state)
    }
  }
}
```

### 5. Barrel Exports

Path: `baseDir/src/index.ts`

```typescript
export { ${PluginName}Plugin, ${PluginName}Events } from './${PluginName}Plugin'
export type { ${PluginName}UpdateData, ${PluginName}StateEvent, ${PluginName}State } from './${PluginName}Plugin'
```

---

## Demo Templates

### WeChat Mini-Program Demo

#### Page WXML

Path: `baseDir/demo/miniprogram/pages/${pluginName}/${pluginName}.wxml`

```wxml
<view class="${pluginName}-container">
  <view class="title">${PluginName} Plugin Demo</view>
  <view class="status">Event: {{eventName}}</view>
  <view class="status">State: {{state}}</view>
</view>
```

#### Page JS

Path: `baseDir/demo/miniprogram/pages/${pluginName}/${pluginName}.js`

```javascript
const liveRoomSdk = getApp().liveRoomSdk

Page({
  data: {
    eventName: 'init',
    state: null
  },

  onLoad() {
    const plugin = liveRoomSdk.getPlugin('${pluginName}')
    this._unsubscribe = plugin.subscribeState((eventName, state) => {
      this.setData({ eventName, state })
    })
  },

  onUnload() {
    if (this._unsubscribe) {
      this._unsubscribe()
    }
  }
})
```

#### Page WXSS

Path: `baseDir/demo/miniprogram/pages/${pluginName}/${pluginName}.wxss`

```wxss
.${pluginName}-container {
  padding: 20rpx;
}

.title {
  font-size: 32rpx;
  font-weight: bold;
  margin-bottom: 20rpx;
}

.status {
  font-size: 28rpx;
  color: #666;
  margin-bottom: 10rpx;
}
```

#### Page JSON

Path: `baseDir/demo/miniprogram/pages/${pluginName}/${pluginName}.json`

```json
{
  "usingComponents": {}
}
```

---

### Vue 2 Demo

#### Composable

Path: `baseDir/demo/vue2/use${PluginName}.ts`

```typescript
import Vue from 'vue'

/**
 * ${PluginName} Plugin Composable (Vue 2)
 */
export function use${PluginName}(
  plugin: {
    subscribeState: (cb: (eventName: string, state: any) => void) => () => void
    getState: () => any
  }
) {
  const state = Vue.observable({ value: plugin.getState() })
  const event = Vue.observable({ value: 'init' as string })

  const unsubscribe = plugin.subscribeState((eventName, newState) => {
    event.value = eventName
    state.value = newState
  })

  return {
    get state() { return state.value },
    get event() { return event.value },
    unsubscribe
  }
}
```

#### Demo Component

Path: `baseDir/demo/vue2/${PluginName}Demo.vue`

```vue
<template>
  <div class="${pluginName}-demo">
    <p>Event: {{ event }}</p>
    <p>State: {{ state }}</p>
  </div>
</template>

<script lang="ts">
import Vue from 'vue'
import { use${PluginName} } from './use${PluginName}'

export default Vue.extend({
  name: '${PluginName}Demo',

  setup() {
    const liveRoomSdk = (window as any).liveRoomSdk
    const ${pluginName}Plugin = liveRoomSdk.getPlugin('${pluginName}')
    const { state, event } = use${PluginName}(${pluginName}Plugin)

    return { state, event }
  }
})
</script>

<style scoped>
.${pluginName}-demo {
  padding: 20px;
}
</style>
```

---

### Vue 3 Demo

#### Generic Composable

Path: `baseDir/demo/vue3/usePlugin.ts`

```typescript
import { ref, onUnmounted, type Ref } from 'vue'

/**
 * Generic plugin state subscription Composable
 */
export function usePlugin<T, E extends string>(
  plugin: {
    subscribeState: (cb: (eventName: E, state: T) => void) => () => void
    getState: () => T
  }
): { state: Ref<T>, event: Ref<E> } {
  const state = ref(plugin.getState()) as Ref<T>
  const event = ref('init' as E)

  const unsubscribe = plugin.subscribeState((eventName, newState) => {
    event.value = eventName
    state.value = newState
  })

  onUnmounted(() => {
    unsubscribe()
  })

  return { state, event }
}
```

#### Plugin-specific Composable

Path: `baseDir/demo/vue3/use${PluginName}.ts`

```typescript
import { computed, type Ref } from 'vue'
import type { ${PluginName}State, ${PluginName}StateEvent } from '../../src/${PluginName}Plugin'
import { ${PluginName}Events } from '../../src/${PluginName}Plugin'
import { usePlugin } from './usePlugin'

/**
 * ${PluginName} Plugin Composable
 */
export function use${PluginName}(
  plugin: {
    subscribeState: (cb: (eventName: ${PluginName}StateEvent, state: ${PluginName}State) => void) => () => void
    getState: () => ${PluginName}State
  }
): { state: Ref<${PluginName}State>, event: Ref<${PluginName}StateEvent>, isUpdate: Ref<boolean> } {
  const { state, event } = usePlugin<${PluginName}State, ${PluginName}StateEvent>(plugin)
  const isUpdate = computed(() => event.value === ${PluginName}Events.UPDATE)

  return { state, event, isUpdate }
}
```

#### Demo Component

Path: `baseDir/demo/vue3/${PluginName}Demo.vue`

```vue
<template>
  <div class="${pluginName}-demo">
    <p>Event: {{ event }}</p>
    <p>State: {{ state }}</p>
  </div>
</template>

<script setup lang="ts">
import type { ${PluginName}State, ${PluginName}StateEvent } from '../../src/${PluginName}Plugin'
import { use${PluginName} } from './use${PluginName}'

declare const liveRoomSdk: {
  getPlugin: (name: string) => any
}

const ${pluginName}Plugin = liveRoomSdk.getPlugin('${pluginName}')
const { state, event, isUpdate } = use${PluginName}(${pluginName}Plugin)
</script>

<style scoped>
.${pluginName}-demo {
  padding: 20px;
}
</style>
```
