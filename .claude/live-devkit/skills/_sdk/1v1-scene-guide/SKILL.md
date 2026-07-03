---
name: 1v1-scene-guide
description: >
  从零搭建 1v1 直播间（H5）的完整引导技能。
  当用户提到"搭建 1v1 直播间"、"接入 live-base"、"接入 live-ui-vue"、
  "新建直播间工程"、"1v1 场景接入"时使用此技能。
  引导用户完成：创建 Vite 工程 → 安装依赖 → 配置环境变量与代理 → 
  接入逻辑层 → 接入 UI 层 → 跑起来。
compatibility: 需要 Node.js >= 18、pnpm 或 npm、能访问内部 npm registry
---

# 1v1 直播间搭建引导

你正在帮助用户从零搭建一个 1v1 直播间 H5 工程。按照下面的步骤依次引导，每步完成后确认再继续。

---

## 架构概览

在开始前，先让用户理解整体架构，这样后面每步做什么他都清楚：

```
┌─────────────────────────────────────────────────────┐
│                   你的工程 (Vite + Vue 3)            │
│                                                     │
│  main.ts                                            │
│  └── App.vue                                        │
│      ├── useVisitorLiveRoom()   ← 逻辑层（状态+动作）│
│      ├── <VideoLayer>           ← 视频渲染（UI库）   │
│      └── <ToolingLayer>         ← 控制栏（UI库）     │
│                                                     │
│  依赖的 npm 包：                                      │
│  ├── @xiaoe/live-base       核心状态机（纯 TS）      │
│  ├── @xiaoe/live-ui-vue     Vue 组件库               │
│  ├── @xiaoe/live-rtc-sdk    RTC 音视频               │
│  ├── @xiaoe/live-room-common-sdk  直播间 SDK         │
│  └── @xiaoe/xiaoe-im-sdk    IM 消息                  │
└─────────────────────────────────────────────────────┘
```

**三层职责**：
- **逻辑层** (`useVisitorLiveRoom`)：管理直播间状态（是否开麦、是否上麦、美颜配置等），封装所有动作（开关麦、挂断、美颜）
- **视频层** (`<VideoLayer>`)：渲染远端全屏视频 + 本地 PiP 小窗，处理 fullscreen ↔ pip 切换。来自 `@xiaoe/live-ui-vue`
- **工具层** (`<ToolingLayer>`)：底部控制栏、美颜面板、倒计时等。子组件（`BottomControls`、`BeautyPanel`、`CountdownBadge`、`MoreMenu`）来自 `@xiaoe/live-ui-vue`，**`ToolingLayer` 本身由开发者自己写，把子组件组合起来**

---

## Step 1：创建工程

**必须指定 Vite 6**，不要用最新版（Vite 8 基于 rolldown，要求 Node 20.19+，容易踩坑）：

```bash
npm create vite@6 my-1v1-room -- --template vue-ts
cd my-1v1-room
```

> 如果已经用 `npm create vite@latest` 创建出来报 rolldown 错误，删掉重建：
> ```bash
> rm -rf my-1v1-room
> npm create vite@6 my-1v1-room -- --template vue-ts
> cd my-1v1-room
> ```

---

## Step 2：配置内部 npm registry

这些包发布在内部 registry，需要先配置。**用 Bash 工具直接帮用户执行**：

```bash
echo "@xiaoe:registry=http://111.230.199.61:6888/" >> .npmrc
```

---

## Step 3：安装依赖

**用 Bash 工具直接帮用户执行**（不要让用户自己跑）：

```bash
pnpm add @xiaoe/live-base @xiaoe/live-ui-vue @xiaoe/live-rtc-sdk @xiaoe/live-room-common-sdk @xiaoe/xiaoe-im-sdk @xiaoe/live-builder-helper
pnpm add -D sass-embedded @vitejs/plugin-vue postcss-pxtorem
```

**当前稳定版本参考**（以实际 registry 最新版为准）：
- `@xiaoe/live-base`: `0.0.1-alpha.5`
- `@xiaoe/live-ui-vue`: `0.0.2-alpha.7`
- `@xiaoe/live-rtc-sdk`: `0.0.1-alpha.11`
- `@xiaoe/live-room-common-sdk`: `1.1.57-alpha.5`
- `@xiaoe/xiaoe-im-sdk`: `1.0.63-alpha.15`

---

## Step 4：配置环境变量

**用 Bash 工具创建目录和文件**，然后提示用户填写真实值：

```bash
mkdir -p env
cat > env/.env.development << 'EOF'
VITE_APPID=
VITE_ALIVE_ID=
VITE_TOKEN=
VITE_SHOP_TOKEN=
VITE_ISELINK=false
EOF
```

创建完后，告知用户需要填写以下四个值，**所有值都从线上正式直播间获取**：

| 变量 | 说明 | 获取方式 |
|------|------|---------|
| `VITE_APPID` | 小鹅通应用 ID，格式 `appixxx` | 打开线上直播间页面，URL 中的 `app_id` 参数，或联系运营同学 |
| `VITE_ALIVE_ID` | 直播间 ID，格式 `l_xxx` | 打开线上直播间页面，URL 路径最后一段，或直播间管理后台 |
| `VITE_TOKEN` | 用户登录 token | 登录小鹅通 H5 后，打开 DevTools → Application → Cookies，找 `ko_token` 的值 |
| `VITE_SHOP_TOKEN` | 店铺 token（出海版需要） | 同上，找 `kl_shop_token` 的值（国内版可留空） |
| `VITE_ISELINK` | 是否出海版，`true`/`false` | 国内版填 `false`，出海版（elink.ai）填 `true` |

> 注意：`loadEnv` 读取路径是 `env/` 子目录，不是根目录的 `.env`。
> 这个文件已加入 `.gitignore`，不会被提交。

---

## Step 5：配置 vite.h5.ts 和 vite.config.ts

`vite.h5.ts` 是 H5 工程的 Vite 配置预设，封装了 Vue 插件、px→rem、root font-size 自动注入。**用 Bash 工具创建此文件**：

```ts
// vite.h5.ts
import { defineConfig, type UserConfig, type ConfigEnv, type Plugin } from 'vite'
import vue from '@vitejs/plugin-vue'
import pxtorem from 'postcss-pxtorem'

function cssRemNoticePlugin(): Plugin {
  return {
    name: 'css-rem-notice',
    generateBundle(_, bundle) {
      const notice = '/* This CSS uses rem units (rootValue: 37.5). For correct rendering, set html { font-size: calc(100vw / 10) } */\n'
      for (const chunk of Object.values(bundle)) {
        if (chunk.type === 'asset' && chunk.fileName.endsWith('.css') && typeof chunk.source === 'string') {
          chunk.source = notice + chunk.source
        }
      }
    }
  }
}

function htmlRootFontSizePlugin(): Plugin {
  return {
    name: 'html-root-font-size',
    transformIndexHtml(html) {
      const styleTag = '<style>html{font-size:calc(100vw/10)}</style>'
      if (html.includes('font-size:calc(100vw/10)') || html.includes('font-size: calc(100vw / 10)')) {
        return html
      }
      return html.replace('</head>', `${styleTag}\n</head>`)
    }
  }
}

const postcssH5Config = {
  plugins: [
    pxtorem({
      rootValue: 37.5,
      unitPrecision: 5,
      propList: ['*'],
      selectorBlackList: [],
      replace: true,
      mediaQuery: false,
      minPixelValue: 2,
      exclude: /node_modules/i
    })
  ]
}

type UserConfigFn = (env: ConfigEnv) => Partial<UserConfig>

export function defineH5Config(configFn: UserConfigFn) {
  return defineConfig(env => {
    const userConfig = configFn(env)
    const { css: userCss, ...restConfig } = userConfig as Partial<UserConfig> & {
      css?: Record<string, unknown>
    }

    return {
      plugins: [vue(), htmlRootFontSizePlugin(), cssRemNoticePlugin()],
      envDir: 'env',
      ...restConfig,
      css: {
        ...userCss,
        postcss: postcssH5Config
      }
    }
  })
}
```

然后创建 `vite.config.ts`，结构与 psychology-room/h5 完全一致：

```ts
// vite.config.ts
import { loadEnv } from 'vite'
import { resolve } from 'path'
import { getProxyConfig } from '@xiaoe/live-builder-helper'
import { defineH5Config } from './vite.h5'

export default defineH5Config(({ mode }) => {
  const env = loadEnv(mode, `${process.cwd()}/env`, '')
  const { VITE_APPID, VITE_TOKEN, VITE_SHOP_TOKEN, VITE_ISELINK } = env

  const klTarget = `https://${VITE_APPID}.elink.ai`
  const xetTarget = `https://${VITE_APPID}.h5.xiaoeknow.com`
  const klCookie = `kl_token=${VITE_TOKEN}; kl_shop_token=${VITE_SHOP_TOKEN}; logintime=1739262981; lang=zh; currency=usd; currency=usd`
  const koCookie = `ko_token=${VITE_TOKEN}`

  return {
    resolve: {
      alias: {
        '@': resolve(__dirname, 'src'),
      },
    },
    build: {
      assetsInlineLimit: 8192,
      rollupOptions: {
        output: {
          entryFileNames: 'js/[name].js',
          chunkFileNames: 'js/[name].js',
          assetFileNames: 'assets/[name].[ext]',
          manualChunks: (id) => {
            if (/@xiaoe[\\/]+xiaoe-im-sdk/.test(id)) return 'vendor-im-sdk'
            if (/@xiaoe[\\/]+live-rtc-sdk/.test(id)) return 'vendor-rtc-sdk'
          },
        },
      },
    },
    server: {
      host: true,
      port: 5176,
      proxy: {
        ...getProxyConfig({
          target: VITE_ISELINK ? klTarget : xetTarget,
          cookie: VITE_ISELINK ? klCookie : koCookie,
          proxyConfig: ['/base', '/user', '/msg'],
        }),
        // /_alive 仅走 xet 国内域名
        ...getProxyConfig({
          target: xetTarget,
          cookie: koCookie,
          proxyConfig: ['/_alive'],
        }),
      },
    },
  }
})
```

---

## Step 6：配置 index.html

替换 `index.html`，加上 H5 必要的 meta：

```html
<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <meta
      name="viewport"
      content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no"
    />
    <title>直播间</title>
  </head>
  <style>
    * { margin: 0; padding: 0; }
  </style>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

---

## Step 7：编写 main.ts

```ts
// src/main.ts
import { createApp } from 'vue'
import '@xiaoe/live-ui-vue/style.css'   // UI 库样式，必须在最前面
import App from './App.vue'

const app = createApp(App)
app.mount('#app')
```

> `@xiaoe/live-ui-vue/style.css` 必须在其他自定义样式之前导入，这样用户可以在后面覆盖 CSS 变量（`--ovl-*`）。

---

## Step 8：编写逻辑层 `src/visitor-live-room.ts`

逻辑层负责把 `@xiaoe/live-base` 的核心状态机适配成 Vue 响应式数据：

```ts
// src/visitor-live-room.ts
import {
  createAudienceLiveRoom,
  type AudienceLiveRoomOptions,
  type AudienceLiveRoomInstance,
} from '@xiaoe/live-base'

export type { AudienceLiveRoomOptions as UseVisitorLiveRoomOptions }
export type { AudienceLiveRoomInstance as VisitorLiveRoomInstance }
export type { AudienceLiveRoomState as VisitorLiveRoomState, BeautyConfig } from '@xiaoe/live-base'

/**
 * createVisitorLiveRoom — 在 createAudienceLiveRoom 基础上增加「自动上麦」逻辑：
 * 直播间进入 Live 状态 且 RTC SDK 就绪后，自动调用 requestOnStage()。
 * 1v1 咨询场景下，访客进入即自动连麦，无需手动操作。
 */
export function createVisitorLiveRoom(options: AudienceLiveRoomOptions): AudienceLiveRoomInstance {
  let voiceSdkReady = false
  let liveStateIsLive = false
  let applyConnectCalled = false

  function tryAutoConnect() {
    if (voiceSdkReady && liveStateIsLive && !applyConnectCalled) {
      applyConnectCalled = true
      base.requestOnStage()
    }
  }

  const base = createAudienceLiveRoom(options, {
    onVoiceSdkReady(linkState: string) {
      voiceSdkReady = true
      // RTC 已经在连麦中（如刷新重进），跳过自动连麦
      if (linkState === 'waiting' || linkState === 'linking') {
        applyConnectCalled = true
      }
      // 延迟一个 microtask，等待 SDK 内部状态机完成初始化
      Promise.resolve().then(() => tryAutoConnect())
    },
    onLiveStateChange(sdkState: string) {
      if (sdkState === 'Live') {
        liveStateIsLive = true
        tryAutoConnect()
      }
    },
  })

  // 如果进入时直播间已经是 Live 状态
  if (base.getState().liveRoomSdkState === 'Live') {
    liveStateIsLive = true
  }

  return base
}
```

---

## Step 9：编写 Vue 适配层 `src/use-visitor-live-room.ts`

把 `createVisitorLiveRoom` 的状态桥接成 Vue `computed`：

```ts
// src/use-visitor-live-room.ts
import { ref, computed, onUnmounted } from 'vue'
import {
  createVisitorLiveRoom,
  type UseVisitorLiveRoomOptions,
  type VisitorLiveRoomState,
  type BeautyConfig,
} from './visitor-live-room'

export function useVisitorLiveRoom(options: UseVisitorLiveRoomOptions) {
  const core = createVisitorLiveRoom(options)
  const state = ref<VisitorLiveRoomState>(core.getState())

  const unsub = core.onChange((s: VisitorLiveRoomState) => {
    state.value = s
  })

  onUnmounted(() => {
    unsub()
    core.destroy()
  })

  return {
    // 状态
    liveRoomSdkState: computed(() => state.value.liveRoomSdkState),
    visitorState:     computed(() => state.value.audienceState),   // 'audience' | 'onstage'
    localUserId:      computed(() => state.value.localUserId),
    hostUserId:       computed(() => state.value.hostUserId),
    countdown:        computed(() => state.value.countdown),
    micOn:            computed(() => state.value.micOn),
    cameraOn:         computed(() => state.value.cameraOn),
    mirrorOn:         computed(() => state.value.mirrorOn),
    beautyOn:         computed(() => state.value.beautyOn),
    beautyConfig:     computed(() => state.value.beautyConfig),
    // 视频容器就绪回调（由 VideoLayer 触发，通知 SDK 开始渲染远端视频）
    onRemoteContainerReady:   (userId: string) => core.notifyRemoteContainerReady(userId),
    onRemoteContainerDestroy: (userId: string) => core.notifyRemoteContainerDestroy(userId),
    // 动作
    onMicToggle:          () => core.onMicToggle(),
    onCameraToggle:       () => core.onCameraToggle(),
    onBeautyToggle:       () => core.onBeautyToggle(),
    onBeautyConfigChange: (config: Partial<BeautyConfig>) => core.updateBeautyConfig(config),
    onFlipToggle:         () => core.onFlipToggle(),
    onMirrorToggle:       () => core.onMirrorToggle(),
    onHangup:             () => core.onHangup(),
    destroy:              () => core.destroy(),
  }
}
```

---

## Step 10：编写 ToolingLayer.vue

`ToolingLayer` 由开发者自己写，内部使用 `@xiaoe/live-ui-vue` 暴露的子组件组合：

```vue
<!-- src/components/ToolingLayer.vue -->
<template>
  <div class="tooling-layer">
    <!-- 倒计时（仅 PreLive 阶段显示） -->
    <CountdownBadge
      v-if="liveRoomSdkState === 'PreLive'"
      :countdown="countdown"
      label="即将开始咨询"
    />

    <!-- 底部控制栏 -->
    <BottomControls
      :mode="visitorState === 'onstage' ? '4-buttons' : '5-buttons'"
      :mic-on="micOn"
      :camera-on="cameraOn"
      :mirror-on="mirrorOn"
      :beauty-on="beautyOn"
      @mic-toggle="emit('mic-toggle')"
      @camera-toggle="emit('camera-toggle')"
      @hangup="emit('hangup')"
      @more-action="moreMenuVisible = !moreMenuVisible"
      @flip-toggle="emit('flip-toggle')"
      @mirror-toggle="emit('mirror-toggle')"
      @beauty-toggle="openBeautyPanel"
    />

    <!-- 更多菜单（仅连麦中显示） -->
    <MoreMenu
      v-if="visitorState === 'onstage'"
      :visible="moreMenuVisible"
      @close="moreMenuVisible = false"
      @flip-toggle="emit('flip-toggle')"
      @mirror-toggle="emit('mirror-toggle')"
      @beauty-toggle="openBeautyPanel"
    />

    <!-- 美颜面板 -->
    <BeautyPanel
      v-model:visible="beautyPanelVisible"
      :beauty="beautyConfig.beauty"
      :brightness="beautyConfig.brightness"
      :ruddy="beautyConfig.ruddy"
      :enabled="beautyOn"
      @update:beauty="emit('beauty-config-change', { beauty: $event })"
      @update:brightness="emit('beauty-config-change', { brightness: $event })"
      @update:ruddy="emit('beauty-config-change', { ruddy: $event })"
      @update:enabled="emit('beauty-toggle')"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import {
  CountdownBadge,
  BottomControls,
  MoreMenu,
  BeautyPanel,
} from '@xiaoe/live-ui-vue'

defineProps<{
  visitorState: string
  liveRoomSdkState: string
  countdown: number
  micOn: boolean
  cameraOn: boolean
  mirrorOn: boolean
  beautyOn: boolean
  beautyConfig: { beauty: number; brightness: number; ruddy: number }
}>()

const emit = defineEmits<{
  'mic-toggle': []
  'camera-toggle': []
  'flip-toggle': []
  'mirror-toggle': []
  'beauty-toggle': []
  hangup: []
  'beauty-config-change': [config: { beauty?: number; brightness?: number; ruddy?: number }]
}>()

const moreMenuVisible = ref(false)
const beautyPanelVisible = ref(false)

function openBeautyPanel() {
  moreMenuVisible.value = false
  beautyPanelVisible.value = true
}
</script>

<style scoped>
.tooling-layer {
  position: absolute;
  inset: 0;
  pointer-events: none;
}
.tooling-layer :deep(*) {
  pointer-events: auto;
}
</style>
```

## Step 11：编写 App.vue

```vue
<!-- src/App.vue -->
<template>
  <div class="live-room-container">
    <!-- 直播结束状态 -->
    <div v-if="isEnded" class="ended-message">咨询已结束</div>

    <template v-else>
      <!-- 视频层：来自 @xiaoe/live-ui-vue -->
      <VideoLayer
        :visitor-state="visitorState"
        :local-user-id="localUserId"
        :host-user-id="hostUserId"
        @remote-container-ready="onRemoteContainerReady"
        @remote-container-destroy="onRemoteContainerDestroy"
      />

      <!-- 工具层：自己写的组件，内部用 @xiaoe/live-ui-vue 子组件 -->
      <ToolingLayer
        :visitor-state="visitorState"
        :live-room-sdk-state="liveRoomSdkState"
        :countdown="countdown"
        :mic-on="micOn"
        :camera-on="cameraOn"
        :mirror-on="mirrorOn"
        :beauty-on="beautyOn"
        :beauty-config="beautyConfig"
        @mic-toggle="onMicToggle"
        @camera-toggle="onCameraToggle"
        @flip-toggle="onFlipToggle"
        @mirror-toggle="onMirrorToggle"
        @beauty-toggle="onBeautyToggle"
        @hangup="onHangup"
        @beauty-config-change="onBeautyConfigChange"
      />
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, onUnmounted } from 'vue'
import { VideoLayer } from '@xiaoe/live-ui-vue'
import ToolingLayer from './components/ToolingLayer.vue'
import { useVisitorLiveRoom } from './use-visitor-live-room'

// DEV 模式从 env 读，生产从 URL 解析
const liveId = import.meta.env.DEV
  ? import.meta.env.VITE_ALIVE_ID
  : (location.pathname.split('/').pop() ?? '')

const appId = import.meta.env.DEV
  ? import.meta.env.VITE_APPID
  : (new URLSearchParams(location.search).get('app_id') ?? '')

const {
  liveRoomSdkState, visitorState, localUserId, hostUserId,
  countdown, micOn, cameraOn, mirrorOn, beautyOn, beautyConfig,
  onRemoteContainerReady, onRemoteContainerDestroy,
  onMicToggle, onCameraToggle, onBeautyToggle, onBeautyConfigChange,
  onFlipToggle, onMirrorToggle, onHangup, destroy,
} = useVisitorLiveRoom({ liveId, appId })

const isEnded = computed(
  () => liveRoomSdkState.value === 'PostLive' || liveRoomSdkState.value === 'Paused'
)

onUnmounted(() => destroy())
</script>

<style lang="scss">
/* H5 根字号：375px 视口下 1rem = 37.5px */
html { font-size: calc(100vw / 10); }
@media (min-width: 480px) { html { font-size: 48px; } }
</style>

<style lang="scss" scoped>
.live-room-container {
  position: relative;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  background-color: #000;
}

.ended-message {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  color: #fff;
  font-size: 16px;
}
</style>
```

---

## Step 11：启动

```bash
pnpm dev
# 打开 http://localhost:5176
```

看到黑色全屏 + 底部控制栏说明接入成功。如果摄像头权限弹窗出现，允许后应看到本地画面。

---

## 常见问题

### 启动报 `Cannot find native binding` / rolldown 错误
用了 Vite 8，需要 Node 20.19+。解决：删掉工程用 `npm create vite@6` 重建，或升级 Node 版本。

### 直播间一直是 PreLive 状态（黑屏，只有倒计时）
- 检查 `VITE_ALIVE_ID` 是否正确，且该直播间当前是「直播中」状态
- 检查 `VITE_TOKEN` 是否有效（用户有权限进入该直播间）

### 报错 `Failed to resolve import "@xiaoe/live-ui-vue/style.css"`
- 检查 `node_modules/@xiaoe/live-ui-vue/dist/` 下是否有 `index.css`
- 如果只有 `live-ui-vue.css`，说明安装的是旧版本，升级到 `0.0.2-alpha.7+`

### 代理 404 / 接口报错
- 检查 `VITE_APPID` 是否正确
- 检查 `VITE_TOKEN` 是否过期

### 本地视频不显示
- 浏览器需要 HTTPS 或 localhost 才能访问摄像头
- 用 `localhost:5176` 访问，不要用 IP

---

## 扩展新能力

搭建完成后，想在这个基础上加功能，按这个原则选位置：

| 需求 | 加在哪里 |
|------|---------|
| 新的业务状态（如「举手」状态） | `visitor-live-room.ts` 里扩展 hooks |
| 新的 Vue 响应式数据 | `use-visitor-live-room.ts` 里加 `computed` |
| 新的控制按钮 | 复制 `ToolingLayer` 里的 `ControlButton` 用法 |
| 修改视频布局（如多人） | 替换 `<VideoLayer>` 或自己写视频容器 |
| 自定义 CSS 变量（改颜色/圆角） | `main.ts` 里在 `style.css` 之后覆盖 `--ovl-*` 变量 |

**核心边界**：`@xiaoe/live-base` 和 `@xiaoe/live-ui-vue` 是只读的 npm 包，不要直接改。业务逻辑扩展在 `visitor-live-room.ts`，UI 扩展在 `App.vue` 或新建组件。
