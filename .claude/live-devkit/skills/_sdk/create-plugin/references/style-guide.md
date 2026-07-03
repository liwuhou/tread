# Plugin Style Guide

Plugin standard style guide - All plugins must follow these rules.

---

## 1. Private Fields

Use `#` private fields, not the `private` keyword.

```typescript
// Correct
#store!: IReadonlyLiveRoomStore
#state: XxxState = { ... }

// Wrong
private store!: IReadonlyLiveRoomStore
```

---

## 2. State Management

Use the five-piece set uniformly: `#state` + `#stateSubscribers` + `subscribeState` + `getState` + `#setState`.

- `subscribeState` returns an unsubscribe function and immediately triggers the INIT callback
- `#setState` flow: merge state -> notify UI subscribers -> notify eventBus (except INIT)
- Scattered state properties are prohibited; consolidate everything into `#state`

```typescript
// Correct
#state: ${PluginName}State = { ... }
#stateSubscribers = new Set<...>()

subscribeState(callback) {
  this.#stateSubscribers.add(callback)
  callback(${PluginName}Events.INIT, this.#state)
  return () => this.#stateSubscribers.delete(callback)
}

#setState(eventName, partialState) {
  this.#state = { ...this.#state, ...partialState }
  this.#stateSubscribers.forEach(fn => fn(eventName, this.#state))
  if (eventName !== ${PluginName}Events.INIT) {
    this.#eventBus.emit(eventName, this.#state)
  }
}

// Wrong - scattered state
listener1: () => void
listener2: () => void
data: any
```

---

## 3. Event Bus

### Event Constant Definition

```typescript
export const XxxEvents = {
  INIT: 'init',           // For subscribeState only
  UPDATE: 'xxx:update'    // Business event
} as const

export type XxxStateEvent = (typeof XxxEvents)[keyof typeof XxxEvents]
```

### Event Type Definition

```typescript
export type XxxEventMap = {
  [XxxEvents.UPDATE]: [state: XxxUpdateData]
}
```

### Module Augmentation

```typescript
declare module '@xiaoe/live-room-common-sdk' {
  interface BusEventMap extends XxxEventMap {}
}
```

### Naming Convention

Event naming: `'plugin-name:action'`, e.g.:
- `player:liveStreamingUrlReady`
- `countdown:update`
- `lottery:start`

---

## 4. Subscription Cleanup

All `on`/`subscribe` calls must push to `#unsubs`. `onDestroy` cleans up all at once.

```typescript
#unsubs: Array<() => void> = []

install(ctx: ISDKContext): void {
  this.#unsubs.push(
    this.#store.subscribe(() => { ... })
  )

  this.#im.on('EVENT', this.#handler)
  this.#unsubs.push(() => this.#im.off('EVENT', this.#handler))
}

onDestroy(): void {
  this.#unsubs.forEach(fn => fn())
  this.#unsubs = []
}
```

---

## 5. Lifecycle

| Hook | When called | Responsibility | Prohibited |
|------|-------------|----------------|------------|
| `install(ctx)` | When `sdk.use(plugin)` is called | Save ctx, register event listeners | Do not make network requests |
| `onReady(baseInfo)` | When baseInfo is loaded | Read store, make network requests | Do not block for too long |
| `onDestroy()` | When SDK is destroyed | Execute `#unsubs`, clean up timers | Do not perform async operations |

**Do not use `onLiveStateChange`**. Use `store.subscribe()` instead.

---

## 6. Barrel Exports

Use `export` for classes and constants, `export type` for types.

```typescript
// Correct
export { XxxPlugin, XxxEvents } from './XxxPlugin'
export type { XxxUpdateData, XxxStateEvent, XxxState } from './XxxPlugin'

// Wrong
export * from './XxxPlugin'  // Exports all internal implementation details
```

---

## 7. TODO Markers

TODO locations in templates:

1. **State data structure** - `interface XxxState { /* TODO */ }`
2. **Initial state defaults** - `#state = { /* TODO */ }`
3. **Store change handling** - `subscribe(() => { /* TODO */ })`
4. **IM event listening** - `this.#im.on('EVENT', ...)`
5. **onReady initialization** - `onReady() { /* TODO: initData */ }`

These TODOs need to be filled in by the plugin developer based on specific business logic.
