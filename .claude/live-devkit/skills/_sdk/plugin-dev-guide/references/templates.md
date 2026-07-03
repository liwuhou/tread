# Plugin Dev Guide - Templates

Backend code template files for the `plugin-dev-guide` skill.

---

## 1. DTO Template (`domain/dto.go`)

```go
package domain

// <Name>Dto description
type <Name>Dto struct {
	AppID      string `json:"app_id" binding:"required"`
	ResourceID string `json:"resource_id" binding:"required"`
	// TODO: Add other fields based on business requirements
}

// <Name>ResultDto result DTO
type <Name>ResultDto struct {
	// TODO: Add fields based on business requirements
}
```

---

## 2. Entity Template (`domain/interactive_<name>.go`)

```go
package domain

import "time"

// Interactive<Name> <plugin-chinese-name> entity
type Interactive<Name> struct {
	ID           int64  `json:"id"`
	AppID        string `json:"app_id"`
	ResourceID   string `json:"resource_id"`
	<Name>ID     string `json:"<name>_id"`  // Plugin ID
	<Name>No     int64  `json:"<name>_no"`  // Number
	State        int32  `json:"state"`      // 0-not started 1-in progress 2-ended
	IsDeleted    int32  `json:"is_deleted"`
	CreatedAt    string `json:"created_at"`
	UpdatedAt    string `json:"updated_at"`
	CreatedBy    string `json:"created_by"`
}

// Create<Name>Dto create request
type Create<Name>Dto struct {
	AppID      string `json:"app_id" binding:"required"`
	ResourceID string `json:"resource_id" binding:"required"`
	// TODO: Add business fields
}

// Generate<Name>ID generate ID (16 digits)
func Generate<Name>ID() string {
	now := time.Now()
	timestamp := fmt.Sprintf("%02d%02d%02d%02d%02d",
		now.Year()%100,
		now.Month(),
		now.Day(),
		now.Hour(),
		now.Minute())
	milli := now.Nanosecond() / 10000000
	return "<name>" + timestamp + fmt.Sprintf("%02d", milli)
}
```

---

## 3. Service Template (`domain/service/<name>_service.go`)

```go
package service

import (
    "context"
    "encoding/json"
    "talkcheap.xiaoeknow.com/AliveDev/alive-delivery-tools/domain"
    "time"
)

// <Name>Service <plugin-chinese-name> service
type <Name>Service struct {
    ctx      context.Context
    <name>Repo   domain.<Name>Repo
    imClient ImClient
}

type ImClient interface {
    Send<Name>Msg(dto *domain.SendImReqDto)
}

func New<Name>Service(
    ctx context.Context,
    <name>Repo domain.<Name>Repo,
    imClient ImClient,
) *<Name>Service {
    return &<Name>Service{
        ctx:      ctx,
        <name>Repo:   <name>Repo,
        imClient: imClient,
    }
}

// <Name>Action <plugin-chinese-name> action
func (s *<Name>Service) <Name>Action(dto *domain.<Name>Dto) (*domain.Interactive<Name>, error) {
    <name>ID := domain.Generate<Name>ID()

    entity := &domain.Interactive<Name>{
        AppID:      dto.AppID,
        ResourceID: dto.ResourceID,
        <Name>ID:   <name>ID,
        <Name>No:   time.Now().Unix(),
        State:      1,  // In progress
        CreatedAt:  time.Now().Format("2006-01-02 15:04:05"),
    }

    if err := s.<name>Repo.Create(entity); err != nil {
        return nil, err
    }

    // Send IM message
    go s.send<Name>IM(entity)

    return entity, nil
}

// send<Name>IM send IM message
func (s *<Name>Service) send<Name>IM(entity *domain.Interactive<Name>) {
    imCtx := context.Background()

    msgData := map[string]interface{}{
        "<name>Id":    entity.<Name>ID,
        "type":        "action",
        // TODO: Add data based on business requirements
    }

    s.sendIM(imCtx, entity.AppID, entity.GroupId, "<name>:action", msgData)
}

// sendIM send IM message
func (s *<Name>Service) sendIM(ctx context.Context, appID, groupID, msgType string, content map[string]interface{}) {
    contentBytes, _ := json.Marshal(content)

    imBodyMap := []map[string]interface{}{
        {
            "msg_type":              3,  // Custom message
            "msg_content":           content,
            "msg_content_data_type": 0,
        },
    }

    s.imClient.Send<Name>Msg(&domain.SendImReqDto{
        AppId:       appID,
        GroupId:     groupID,
        FromAccount: "system",
        MsgPriority: "high",
        ImBody: []*domain.SendImReqImBodyDto{
            {
                MsgType:              3,
                MsgContent:           domain.MsgContent{Data: string(contentBytes)},
                MsgContentDataType: 0,
            },
        },
    })
}
```

---

## 4. IM Client Template (`infra/repo/client_service/alive_<name>_im.go`)

```go
package client_service

import (
    "context"
    "encoding/json"
    "net/http"
    "talkcheap.xiaoeknow.com/AliveDev/alive-delivery-tools/domain"
    "time"
)

var (
    _<name>SceneCode = "im-xxxxxxxxxxxxx"  // TODO: Apply for scene Code
    _send<Name>Msg   = "/tim/send_msg"
    _version         = "1.0.0"
)

// Alive<Name>ImClient <plugin-chinese-name> IM client
type Alive<Name>ImClient struct {
    *httpBase
}

func NewAlive<Name>ImClient(ctx context.Context, host string, client *http.Client) *Alive<Name>ImClient {
    return &Alive<Name>ImClient{httpBase: newHttpBase(ctx, host, client)}
}

// Send<Name>Msg send <plugin-chinese-name> IM message
func (c *Alive<Name>ImClient) Send<Name>Msg(dto *domain.SendImReqDto) {
    // Process message content, add version field, and set Desc and Ext
    for k, v := range dto.ImBody {
        var g map[string]interface{}
        json.Unmarshal([]byte(v.MsgContent.Data), &g)
        g["version"] = _version

        // Set Desc and Ext for frontend identification
        dto.ImBody[k].MsgContent.Desc = "<name>"           // <plugin-chinese-name> identifier
        dto.ImBody[k].MsgContent.Ext = "interactive-<name>" // Extension identifier
        dto.ImBody[k].MsgContent.Data = marshal(g)
    }

    req := &requestData{
        header:  map[string]string{"app_id": dto.AppId},
        path:    _send<Name>Msg,
        timeOut: time.Second * 3,
        queryParams: map[string]string{"app_id": dto.AppId},
        postParams: map[string]interface{}{
            "group_id":     dto.GroupId,
            "from_account": dto.FromAccount,
            "scene_code":   _<name>SceneCode,
            "im_body":      dto.ImBody,
        },
    }

    resp, err := c.httpPostV2(req)
    // TODO: Handle response
}
```

---

## 5. API Handler Template (`server/<name>.go`)

```go
package server

import (
    "github.com/gin-gonic/gin"
    "talkcheap.xiaoeknow.com/AliveDev/alive-delivery-tools/app"
    "talkcheap.xiaoeknow.com/AliveDev/alive-delivery-tools/domain"
)

// <Name>Router register <plugin-chinese-name> routes
func <Name>Router(Router *gin.RouterGroup) {
    apiRouter := Router.Group("/api/v1/interactive/<name>")
    {
        apiRouter.POST("/action", <Name>Action)    // TODO: Define endpoints
        apiRouter.GET("/get", Get<Name>)           // Get information
        apiRouter.GET("/list", List<Name>)         // Get list
    }
}

// <Name>Action <plugin-chinese-name> action
// @Summary <plugin-chinese-name> action
// @Tags Interactive Q&A - <plugin-chinese-name>
// @version 1.0
// @Accept json
// @Produce json
// @Param body body domain.<Name>Dto{} true "Input parameters"
// @Success 200 object Response{}
// @Router /api/v1/interactive/<name>/action [post]
func <Name>Action(c *gin.Context) {
    var dto domain.<Name>Dto
    panicValidateError(c.ShouldBindJSON(&dto))

    result, err := app.New<Name>(c.Request.Context()).<Name>Action(&dto)
    if err != nil {
        WriteFailedResp(c, err)
        return
    }

    WriteOKResp(c, result)
}

// Get<Name> get <plugin-chinese-name> information
// @Summary Get <plugin-chinese-name> information
// @Tags Interactive Q&A - <plugin-chinese-name>
// @version 1.0
// @Produce json
// @Param <name>_id query string true "<plugin-chinese-name> ID"
// @Success 200 object Response{Data=domain.Interactive<Name>}
// @Router /api/v1/interactive/<name>/get [get]
func Get<Name>(c *gin.Context) {
    <name>ID := c.Query("<name>_id")

    entity := app.New<Name>(c.Request.Context()).Get<Name>ByID(<name>ID)
    WriteOKResp(c, entity)
}

// List<Name> get <plugin-chinese-name> list
// @Summary Get <plugin-chinese-name> list
// @Tags Interactive Q&A - <plugin-chinese-name>
// @version 1.0
// @Produce json
// @Param resource_id query string true "Live room ID"
// @Success 200 object Response{Data=[]domain.Interactive<Name>}
// @Router /api/v1/interactive/<name>/list [get]
func List<Name>(c *gin.Context) {
    resourceID := c.Query("resource_id")

    entities := app.New<Name>(c.Request.Context()).List<Name>ByResourceID(resourceID)
    WriteOKResp(c, entities)
}
```

---

## 6. Route Registration Template (`server/register.go`)

```go
func RegisterHttpServer(s *gin.Engine) error {
    // pprof
    RegisterPprof(s)
    // Register health check
    RegisterHealth(s)

    // Main routes
    sGroup := s.Group("",
        plugins.XeSpecificContextSetMiddleware,
        plugins.RequestLogMiddleware(alive_log.AccessLog()),
        plugins.AliveErrorRecoverMiddleware(alive_log.BusinessLog()),
    )

    // Module route groups
    UserRouter(sGroup)
    // TODO: Add new plugin routes
    <Name>Router(sGroup)  // <- Add this line

    return nil
}
```

---

## 7. App Layer Template (`app/<name>.go`)

```go
package app

import (
    "context"
    "talkcheap.xiaoeknow.com/AliveDev/alive-delivery-tools/domain"
    "talkcheap.xiaoeknow.com/AliveDev/alive-delivery-tools/domain/service"
    "talkcheap.xiaoeknow.com/AliveDev/alive-delivery-tools/infra/common"
    "talkcheap.xiaoeknow.com/AliveDev/alive-delivery-tools/infra/repo/client_service"
    "net/http"
)

type <Name> struct {
    ctx           context.Context
    <name>Service *service.<Name>Service
}

func New<Name>(ctx context.Context) *<Name> {
    imClient := client_service.NewAlive<Name>ImClient(ctx, common.ImAddr, &http.Client{})
    return &<Name>{
        ctx:           ctx,
        <name>Service: service.New<Name>Service(ctx, <name>Repo, imClient),
    }
}

// <Name>Action <plugin-chinese-name> action
func (n *<Name>) <Name>Action(dto *domain.<Name>Dto) (*domain.Interactive<Name>, error) {
    return n.<name>Service.<Name>Action(dto)
}

// Get<Name>ByID get by ID
func (n *<Name>) Get<Name>ByID(id string) *domain.Interactive<Name> {
    // TODO: Implement
    return nil
}

// List<Name>ByResourceID get list by resource ID
func (n *<Name>) List<Name>ByResourceID(resourceID string) []*domain.Interactive<Name> {
    // TODO: Implement
    return nil
}
```

---

## 8. Gateway Configuration Template

### gateway_interact Service Configuration Format

Add route configuration in `gateway_interact/cmd/gateway/resources/gateway-{env}.yaml`:

```yaml
routes:
  # ======================================= [Plugin Name] ========================================
  - id: alive-interactive-<name>
    uri: http://alive-delivery-tools:18383
    predicates:
      path: /api/v1/interactive/<name>/*
    filters:
      - AuthFilter=1
      - CrossFilter=1
```

### Configuration File Locations

| Environment | Configuration File |
|-------------|-------------------|
| Development | `gateway-develop.yaml` |
| Testing | `gateway-test.yaml` |
| Production | `gateway-production.yaml` |
| Docker deployment | `gateway-docker.yaml` |
| K8s deployment | `gateway-k8s.yaml` |

### Configuration Notes

| Field | Description | Example |
|-------|-------------|---------|
| `id` | Route unique identifier | `alive-interactive-answer-card` |
| `uri` | Backend service address | `http://alive-delivery-tools:18383` |
| `predicates.path` | Match path (use `*` wildcard) | `/api/v1/interactive/<name>/*` |
| `filters` | Filter chain (optional) | `AuthFilter`, `CrossFilter`, `StripPrefix` |

### Common Filters

- `AuthFilter=1` - Enable authentication
- `CrossFilter=1` - Enable CORS
- `StripPrefix=N` - Strip first N path prefixes
- `AliveLimiterFilter=1` - Rate limiting
- `UserLimiterFilter=1` - User-level rate limiting
- `IpLimiterFilter=1` - IP-level rate limiting

### Full Example (Answer Card Plugin)

```yaml
routes:
  # ======================================= [Answer Card Plugin] ========================================
  - id: alive-interactive-answer-card
    uri: http://alive-delivery-tools:18383
    predicates:
      path: /api/v1/interactive/answer-card/*
    filters:
      - AuthFilter=1
      - CrossFilter=1
```

---

## Variable Reference

| Variable | Description | Example |
|----------|-------------|---------|
| `${name}` | Lowercase plugin name | `answer-card` |
| `${Name}` | PascalCase plugin name | `AnswerCard` |
| `${NAME}` | Uppercase plugin name | `ANSWER_CARD` |
| `${plugin-chinese-name}` | Chinese description | `Answer card` |
