# Plugin Dev Guide - Style Guide

## Frontend Plugin Specification

### 1. Plugin Structure

```typescript
export class <Name>Plugin implements IPlugin {
  readonly name = '<name>'

  // Dependency injection
  #store!: IReadonlyLiveRoomStore
  #eventBus!: EventBus<BusEventMap>
  #im!: IImAdapter
  #network!: INetworkAdapter

  // State management
  #state: <Name>State = { /* initial state */ }
  #stateSubscribers = new Set<(event: string, state: <Name>State) => void>()
  #unsubs: Array<() => void> = []

  // Lifecycle
  install(ctx: ISDKContext): void { }
  onReady(baseInfo: LiveRoomData): void { }
  onDestroy(): void { }

  // Public API
  subscribeState(callback: Fn): () => void { }
  getState(): <Name>State { }
  processIMMessage(data: any): void { }
}
```

### 2. Naming Convention

| Type | Convention | Example |
|------|------------|---------|
| Plugin name | kebab-case | `answer-card` |
| Class name | PascalCase | `AnswerCardPlugin` |
| Directory name | `plugin-` + plugin name | `plugin-answer-card` |
| npm package name | `@xiaoe/live-` + plugin name + `-sdk` | `@xiaoe/live-answer-card-sdk` |
| Event prefix | plugin name + `:` | `answer-card:` |
| IM message Desc | kebab-case | `answer-card` |
| IM message Ext | `interactive-` + kebab-case | `interactive-answer-card` |

### 3. State Management Specification

```typescript
// Good practice
#setState(event: string, partialState: Partial<<Name>State>): void {
  this.#state = { ...this.#state, ...partialState }
  this.#stateSubscribers.forEach(fn => fn(event, this.#state))
  this.#eventBus.emit(event, this.#state)
}

// Bad practice - directly mutating state
this.#state.currentMode = 'answering'
```

### 4. Event Naming Convention

```typescript
export const <Name>Events = {
  INIT: 'init',
  UPDATE: '<name>:update'
} as const

// Lifecycle type extension
declare module '@xiaoe/live-room-common-sdk' {
  interface BusEventMap {
    [<Name>Events.UPDATE]: [state: <Name>State]
  }
}
```

### 5. IM Message Processing

```typescript
// Good practice - supports external invocation
processIMMessage(data: any): void {
  this.#onGetContentMsg({ msg_content: data })
}

// Internal processing
#onGetContentMsg = (data: { msg_content: any }) => {
  const msgContent = data.msg_content
  const msgData = msgContent?.data || msgContent.Data

  // Parse JSON
  let parsedData: any = typeof msgData === 'string'
    ? JSON.parse(msgData)
    : msgData

  // Dispatch to corresponding handler
  if (parsedData?.questionId) {
    this.#onQuestion(parsedData)
  }
}
```

---

## Backend Service Specification

### 1. Directory Structure

```
alive-delivery-tools/
+-- domain/
|   +-- dto.go                      # DTO definitions
|   +-- interactive_<name>.go       # Entity definitions
|   +-- repo.go                     # Repository interface
|   +-- service/
|       +-- <name>_service.go       # Business logic
+-- infra/
|   +-- repo/
|       +-- client_service/
|           +-- alive_<name>_im.go  # IM client
+-- app/
|   +-- <name>.go                   # Application layer
+-- server/
    +-- <name>.go                   # API handlers
    +-- register.go                 # Route registration
```

### 2. DTO Naming Convention

```go
// Good practice
type <Name>Dto struct {           // Request DTO
    AppID      string `json:"app_id" binding:"required"`
    ResourceID string `json:"resource_id" binding:"required"`
}

type <Name>ResultDto struct {     // Result DTO
    Total int64 `json:"total"`
}

type Create<Name>Dto struct {     // Create request
    // ...
}
```

### 3. Service Layer Specification

```go
// Good practice
func (s *<Name>Service) Action(dto *domain.<Name>Dto) (*domain.Interactive<Name>, error) {
    // 1. Generate ID
    id := domain.Generate<Name>ID()

    // 2. Create entity
    entity := &domain.Interactive<Name>{ /* ... */ }

    // 3. Save
    if err := s.<name>Repo.Create(entity); err != nil {
        return nil, err
    }

    // 4. Send IM message asynchronously
    go s.sendIM(entity)

    return entity, nil
}
```

### 4. IM Message Format

```go
// Standard format
{
  "msg_type": 3,
  "msg_content": {
    "Data": "{\"<name>Id\":\"xxx\",\"type\":\"action\"}",
    "Desc": "<name>",                    // Message identifier (lowercase)
    "Ext": "interactive-<name>",         // Extension identifier
    "version": "1.0.0"
  },
  "msg_content_data_type": 0
}
```

### 5. API Route Specification

```go
// Standard route group
func <Name>Router(Router *gin.RouterGroup) {
    apiRouter := Router.Group("/api/v1/interactive/<name>")
    {
        apiRouter.POST("/action", <Name>Action)
        apiRouter.GET("/get", Get<Name>)
        apiRouter.GET("/list", List<Name>)
    }
}
```

---

## Gateway Configuration Specification

### 1. Route ID Naming

```yaml
# Good practice
- id: alive-interactive-<name>

# Bad practice
- id: <name>  # Too simple, may conflict
- id: alive-<name>-api  # Inconsistent
```

### 2. Path Matching

```yaml
# Standard format
predicates:
  - Path=/api/v1/interactive/<name>/**

# Wrong format
predicates:
  - Path=/api/<name>/**  # Incorrect path
```

---

## Testing Specification

### 1. Frontend Test Checklist

- [ ] Plugin installed successfully
- [ ] State subscription works correctly
- [ ] IM message reception works correctly
- [ ] UI components render correctly

### 2. Backend Test Checklist

- [ ] API endpoints are accessible
- [ ] Data persistence works correctly
- [ ] IM message sending works correctly
- [ ] Log records are complete

---

## Release Specification

### 1. Version Number Rules

```
Major.Minor.Patch
  |      |      |
  Major  Feature Fix
```

### 2. Commit Message Format

```
feat(<name>): add answer card feature

- Implement frontend plugin
- Implement backend service
- Configure gateway routing
```
