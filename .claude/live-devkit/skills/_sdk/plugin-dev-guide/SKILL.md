---
name: plugin-dev-guide
description: >
  Live room plugin integrated development guide, step-by-step completion of the full flow from creation to deployment.
  Must use this skill: when the user mentions "develop plugin", "plugin integration", "plugin development flow",
  "front-end and back-end integrated plugin", "add interactive feature", "plugin deployment",
  or any scenario requiring a complete plugin development flow.
  Also applicable when needing to reference complete plugin development specs, IM message protocol,
  or gateway configuration.
compatibility: Requires Bash tool to create files and directories, requires Go environment (backend development)
context: fork
---

# Plugin Development Guide - Integrated Plugin Development

This is a **step-by-step guided** plugin development skill, covering the full flow from creation to deployment.

## Flow Overview

```
+-------------+   +-------------+   +-------------+   +-------------+
|  Stage 1    | -> |  Stage 2    | -> |  Stage 3    | -> |  Stage 4    |
|  Frontend   |   |  Gateway    |   |  Backend    |   |  Integration |
|  Plugin     |   |  Config     |   |  Service    |   |  Test &      |
|  Creation   |   |  Routing    |   |  Development |   |  Deploy      |
+-------------+   +-------------+   +-------------+   +-------------+
```

**Rationale**: Gateway configuration comes before backend development, so that once backend development is complete, the API can be verified directly through the gateway.

## Parameters

`$ARGUMENTS` -- Format: `<plugin-name> [plugin-chinese-name]`

- **Plugin name**: kebab-case (e.g. `answer-card`, `lottery`, `gift`)
- **Plugin Chinese name**: Optional, used for documentation comments (e.g. `answer-card`, `lottery`, `gift`)

---

## Stage 1: Frontend Plugin Creation

### Step 1.1: Collect Information

> **Tip**: Before creating a plugin, it is recommended to use `/openspec-new-change` to generate a plugin requirements document to clarify the scope and API design.

**Output stage title**:
```
--------------------------------------------------------------
  Stage 1: Frontend Plugin Creation
--------------------------------------------------------------
```

Use a **terminal interactive** style output:

```
--------------------------------------------------------------
  Plugin Development Guide
--------------------------------------------------------------

Starting to create quiz-buzzer plugin

Please confirm the following information:

  1. Plugin name        quiz-buzzer
  2. Plugin description Live room quiz buzzer, supports initiating quizzes, student responses, result determination and statistics
  3. Feature description Live quiz buzzer, supports initiating quiz, student buzzer, result determination and statistics
  4. Requires backend   Yes
  5. Demo platform      Vue 3 (default)

--------------------------------------------------------------

Tip: Use /openspec-new-change to generate a plugin requirements document

Confirm to start creation? [Y/n] Or enter "plan" to do requirements analysis first
```

**After user confirms `Y`**, output:

```
--------------------------------------------------------------
  Information confirmed
--------------------------------------------------------------

Next you can choose:

  1. Skip planning, create frontend plugin directly
     -> Quick validation, suitable for simple features

  2. Do OpenSpec planning first
     -> Use /openspec-new-change to generate requirements document
     -> Clarify API interfaces, DTOs, IM message formats
     -> Provide task.md path to continue after planning is done

Please select: [1/2] Or reply "skip"/"plan" directly
```

**If user selects planning**:
- Guide user to use `/openspec-new-change` to create requirements document
- Wait for user to complete planning, then provide `openspec/changes/<name>/task.md` path
- Continue by reading task.md to understand requirements, then proceed to create the plugin

**If user selects skip**:
- Proceed directly to Step 1.2 to create the frontend plugin

### Step 1.2: Call create-plugin to Create Frontend Plugin

**Ask about Demo platform selection**:

```
--------------------------------------------------------------
  Select Demo Platforms
--------------------------------------------------------------

Please select which Demo platforms to create (multiple selection):

  1. Vue 3
  2. Vue 2
  3. WeChat mini-program

Enter options (e.g. 1 or 1,2 or 123): [1]
```

Use the `/create-plugin` skill to create the standard frontend plugin:

```bash
/create-plugin <plugin-name> packages/plugins
```

After creation, show the directory structure to the user:

```
packages/plugins/plugin-<name>/
+-- package.json           done
+-- tsconfig.json          done
+-- src/
|   +-- index.ts           done
|   +-- <Name>Plugin.ts    done  (scaffold code)
+-- demo/
    +-- vue3/              done
```

### Step 1.3: Explain Frontend Plugin Structure

Explain the generated files to the user:

| File | Description |
|------|-------------|
| `src/<Name>Plugin.ts` | Plugin core implementation |
| `src/index.ts` | Public API exports |
| `demo/vue3/` | Vue 3 usage example |

### Step 1.4: Next Step Prompt

Use terminal interactive style:

```
--------------------------------------------------------------
  Stage 1 Complete - Frontend Plugin Created
--------------------------------------------------------------

Next, enter Stage 2: Gateway Configuration

Configure gateway routing rules first, so that after the backend service is developed, the API can be verified directly through the gateway.

Includes:
  - Confirm gateway configuration method (configuration file)
  - Generate gateway route configuration
  - Provide configuration flow

--------------------------------------------------------------

Continue? [Y/n]
```

---

## Stage 2: Gateway Configuration

> **Note**: Configure gateway routing first, then develop the backend service, so that after the backend is complete, the API can be verified directly through the gateway.

**Output stage title**:
```
--------------------------------------------------------------
  Stage 2: Gateway Configuration
--------------------------------------------------------------
```

### Step 2.1: Configure Project Root Path

**Explanation**:
- Gateway configuration is done in the `gateway_interact` project
- Backend service development is done in the `alive-delivery-tools` project

**Assuming all projects are under the same root directory**, only configure the root path once.

Ask the user for the project root path:

```
Please enter the project root path:

Example:
  /Users/liuwei/Documents/XETPro

Explanation:
  - gateway_interact will be at: <root-path>/gateway_interact
  - alive-delivery-tools will be at: <root-path>/alive-delivery-tools

Root path:
```

### Step 2.2: Check and Clone Projects

**Check if gateway_interact exists**:

```bash
ls -la <root-path>/gateway_interact
```

**Check if alive-delivery-tools exists**:

```bash
ls -la <root-path>/alive-delivery-tools
```

**If a project does not exist**, prompt the user:

```
Warning: The following projects do not exist:
  - gateway_interact: Needs manual configuration or clone
  - alive-delivery-tools: Use /alive-system-bridge to clone

Do you need to clone alive-delivery-tools? [Y/n]
```

If the user confirms, use the `/alive-system-bridge` skill to clone:

```bash
/alive-system-bridge clone alive-delivery-tools --target <root-path>
```

**If projects already exist**, continue to the next step.

### Step 2.3: Switch Development Branch

Switch or create a branch based on user needs:

```bash
cd <root-path>/alive-delivery-tools

# View current branch
git branch

# Let user choose branch name
```

**Ask the user**:
```
Branch Name Selection

Current branch: <current-branch-name>

Enter the development branch name to use:
- Recommended format: dev/feature-<plugin-name>
- Example: dev/feature-quiz-buzzer

Branch name: [dev/feature-<name>]
```

```bash
# Switch/create branch
git checkout <branch-name>
# Or create a new branch
git checkout -b <branch-name>
```

### Step 2.4: Confirm Environment Configuration

Check and configure necessary environment variables:

```bash
# Check .env.develop file
cat <root-path>/alive-delivery-tools/.env.develop
```

Ensure it includes the following configuration:

```bash
# IM service address
LB_ALIVE_IM_IN=http://localhost:8080

# Database configuration (if needed)
DB_DATABASE_<NAME>=db_test
```

### Step 2.5: Explain Gateway Configuration Method

```
Gateway Configuration (Simple)

Add a routing rule in the gateway_interact service configuration file
to forward plugin API requests to the alive-delivery-tools service.
```

### Step 2.6: Generate Gateway Route Configuration

Generate gateway route configuration for you:

```yaml
# gateway_interact route configuration
# Add to the routes list in gateway-{env}.yaml

# ======================================= [<plugin-chinese-name>] ========================================
- id: alive-interactive-<name>
  uri: http://alive-delivery-tools-master.alive-delivery-tools.svc.cluster.local
  predicates:
    path: /api/v1/interactive/<name>/*
  filters:
    - AuthFilter=1
    - CrossFilter=1
```

**Note**: The production network service load address is `http://alive-delivery-tools-master.alive-delivery-tools.svc.cluster.local`

### Step 2.7: Configuration Flow

```
Configuration Steps

1. Find the gateway_interact service configuration file
   Path: gateway_interact/cmd/gateway/resources/gateway-{env}.yaml

   Environment file mapping:
   - Development: gateway-develop.yaml
   - Testing: gateway-test.yaml
   - Production: gateway-production.yaml

2. Add the above configuration to the end of the routes list

3. Restart the gateway_interact service
   cd gateway_interact && make run

   After gateway starts successfully, routes take effect automatically. No additional verification needed.
```

### Step 2.8: Next Step Prompt

Use terminal interactive style:

```
--------------------------------------------------------------
  Stage 2 Complete - Backend Environment Ready, Gateway Route Generated
--------------------------------------------------------------

Environment preparation checklist:
  - Project root path: <root-path>
  - gateway_interact: <root-path>/gateway_interact
  - alive-delivery-tools: <root-path>/alive-delivery-tools
  - Development branch: <branch-name>
  - Environment configuration: Checked

Gateway configuration checklist:
  - Route ID: alive-interactive-<name>
  - Backend address: http://alive-delivery-tools:18383
  - Match path: /api/v1/interactive/<name>/*
  - Filters: AuthFilter(auth), CrossFilter(CORS)

--------------------------------------------------------------

Next, enter Stage 3: Backend Service Development

Backend service includes:
  - Define DTOs and entities (domain layer)
  - Implement business logic (service layer)
  - Implement IM client (infra layer)
  - Implement API handlers (app/server layer)

Continue? [Y/n]
```

---

## Stage 3: Backend Service Development

> **Prerequisite**: Stage 2 has completed backend project environment preparation (project path, branch switch, environment check)

**Output stage title**:
```
--------------------------------------------------------------
  Stage 3: Backend Service Development
--------------------------------------------------------------
```

### Step 3.1: Confirm Whether New Database Tables Are Needed

**Ask the user**:
```
Database Table Confirmation

Does your plugin need new database tables?

1. Yes, need new tables -> Output table structure design (refer to live marketing delivery database specification)
2. No, use existing tables -> Continue development directly

Your choice: [1/2]
```

**If option 1 (need new tables)**:
- Output table structure design first (see Stage 4.1 template)
- Continue development after user confirms
- Remind user to contact abner to execute the table creation SQL first

**If option 2 (use existing tables)**:
- Proceed directly to Step 3.2

### Step 3.2: Create Domain Layer

Add DTO definitions in `domain/dto.go` (create the file if it doesn't exist):

**Ask user about question types**:
```
Define Data Transfer Objects (DTO)

Please describe which API interfaces your plugin needs. For example:
- Send question (SendQuestion)
- Submit answer (SubmitAnswer)
- End answering (EndAnswer)
- Get statistics (GetResult)

Or briefly describe the features, and I'll help you design the DTOs.
```

Based on user description, generate DTO code and create:
- `domain/dto.go` - Request/Response DTOs
- `domain/interactive_<name>.go` - Entity definitions

### Step 3.3: Create Repository Layer

Ask user about storage method:

```
Select data storage method:
1. MySQL (default)
2. Redis
3. In-memory (testing only)
4. External API

Your choice: [1]
```

Generate repository interface (`domain/repo.go`):

```go
type <Name>Repo interface {
    Create(entity *<EntityName>) error
    GetByID(id string) *<EntityName>
    Update(entity *<EntityName>) error
    GetByResourceID(resourceID string) []*<EntityName>
}
```

### Step 3.4: Create Service Layer

Create business logic in `domain/service/<name>_service.go`:

**Information needed from user**:
1. IM message sending scene Code (e.g. `im-3e5b0683400ceef96`)
2. Message ID generation rule (use timestamp or UUID)
3. Whether multiple IM messages need to be sent (e.g. answer:send + answer:question)

Generated Service code includes:
- Constructor
- Business methods (SendXxx, SubmitXxx, EndXxx)
- IM message sending methods
- Helper functions (type conversion, etc.)

### Step 3.5: Create IM Client

Create IM client in `infra/repo/client_service/alive_<name>_im.go`:

**IM message format explanation**:
```go
{
  "msg_type": 3,  // Custom message
  "msg_content": {
    "Data": "{...JSON data...}",
    "Desc": "<name>",           // Message identifier
    "Ext": "interactive-<name>" // Extension identifier
  }
}
```

### Step 3.6: Create API Handlers

Create HTTP handlers in `server/<name>.go`:

Generated content includes:
- Route registration function
- Handler functions (bind parameters, call Service, return response)
- Swagger annotations

Register routes in `server/register.go`:

```go
func RegisterHttpServer(s *gin.Engine) error {
    // ...
    <Name>Router(sGroup)  // <- Add this line
    // ...
}
```

### Step 3.7: Next Step Prompt

Use terminal interactive style:

```
--------------------------------------------------------------
  Stage 3 Complete - Backend Service Created
--------------------------------------------------------------

Backend file checklist:
  - domain/dto.go                          (DTO definitions)
  - domain/interactive_<name>.go           (Entity definitions)
  - domain/repo.go                         (Repository interface)
  - domain/service/<name>_service.go       (Business logic)
  - infra/repo/client_service/alive_<name>_im.go  (IM client)
  - server/<name>.go                       (API handlers)
  - server/register.go                     (Route registration - modified)

--------------------------------------------------------------

Next, enter Stage 4: Integration Testing & Deployment

Includes:
  - Database table structure design (if needed)
  - Backend configuration checklist (Apollo/Nacos, IM scene)
  - Start frontend and backend services
  - Test API and IM messages
  - Build and publish
  - xiaoe-bus MCP deployment

Continue? [Y/n]
```

---

## Stage 4: Integration Testing & Deployment

**Output stage title**:
```
--------------------------------------------------------------
  Stage 4: Integration Testing & Deployment
--------------------------------------------------------------
```

### Step 4.1: Database Table Structure Design

> **Note**: If new database tables are needed, you must output the table structure design first, following the live marketing delivery database specification.

**Output table structure design** (if new tables are needed):

```markdown
Database Table Structure Design

Reference table: t_alive_auction (Live auction table)

[Table 1] t_alive_interactive_<name> (<plugin-chinese-name> main table)

```sql
CREATE TABLE t_alive_interactive_<name> (
    id               BIGINT AUTO_INCREMENT PRIMARY KEY COMMENT 'Primary key ID',
    app_id           VARCHAR(64)  NOT NULL COMMENT 'Shop ID',
    resource_id      VARCHAR(64)  NOT NULL COMMENT 'Live room ID',
    <name>_id        VARCHAR(64)  NOT NULL COMMENT '<plugin-chinese-name> ID',
    <name>_no        BIGINT       NOT NULL COMMENT '<plugin-chinese-name> number',
    title            VARCHAR(255) DEFAULT '' COMMENT 'Title',
    state            TINYINT      NOT NULL DEFAULT 0 COMMENT 'State: 0-not started 1-in progress 2-ended',
    is_deleted       TINYINT      NOT NULL DEFAULT 0 COMMENT 'Is deleted: 0-normal 1-deleted',
    created_at       TIMESTAMP    DEFAULT CURRENT_TIMESTAMP NOT NULL COMMENT 'Created at',
    updated_at       TIMESTAMP    DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated at',
    created_by       VARCHAR(64)  DEFAULT '' COMMENT 'Creator user ID',
    UNIQUE KEY uk_app_resource_<name> (app_id, resource_id, <name>_id),
    INDEX idx_resource_id (resource_id),
    INDEX idx_<name>_no (<name>_no),
    INDEX idx_created_at (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Interactive Q&A - <plugin-chinese-name> table';
```

[Table 2] t_alive_interactive_<name>_participant (Participant table, if needed)

```sql
CREATE TABLE t_alive_interactive_<name>_participant (
    id            BIGINT AUTO_INCREMENT PRIMARY KEY COMMENT 'Primary key ID',
    app_id        VARCHAR(64)  NOT NULL COMMENT 'Shop ID',
    resource_id   VARCHAR(64)  NOT NULL COMMENT 'Live room ID',
    <name>_id     VARCHAR(64)  NOT NULL COMMENT '<plugin-chinese-name> ID',
    user_id       VARCHAR(64)  NOT NULL COMMENT 'User ID',
    user_name     VARCHAR(128) NOT NULL COMMENT 'User nickname',
    user_avatar   VARCHAR(512) DEFAULT '' COMMENT 'User avatar',
    <data_field>  <TYPE>       NOT NULL COMMENT 'Business data field',
    rank          INT          NOT NULL DEFAULT 0 COMMENT 'Rank',
    is_deleted    TINYINT      NOT NULL DEFAULT 0 COMMENT 'Is deleted: 0-normal 1-deleted',
    created_at    TIMESTAMP    DEFAULT CURRENT_TIMESTAMP NOT NULL COMMENT 'Created at',
    INDEX idx_<name>_id (<name>_id),
    INDEX idx_<name>_user (<name>_id, user_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Interactive Q&A - <plugin-chinese-name> participant table';
```

[Table Creation Notes]
- Field specification: app_id/resource_id uniformly use VARCHAR(64)
- Time fields: Use BIGINT for millisecond timestamps or TIMESTAMP
- State fields: Use TINYINT, 0 for initial state
- Index design: Unique index + business query indexes
```

### Step 4.2: Backend Configuration Checklist

**Output configuration checklist**:

```markdown
Backend Configuration Checklist (Contact abner for configuration)

[Configuration Center Apollo/Nacos]
Configuration Item           Value                              Description
---------------------------------------------------------------------------
LB_ALIVE_IM_IN              http://10.192.24.80:8080            IM service internal address
DB_DATABASE_ALIVE            db_ex_alive_market                  Live marketing delivery database
REDIS_HOST                   10.192.24.xx                        Redis address
REDIS_PORT                   6379                                Redis port

[IM Scene Registration]
Scene Name       Scene Code              Desc                    Ext
---------------------------------------------------------------------------
<plugin-chinese-name>    im-xxxxxxxxxxxxxxxx   <plugin-name>          interactive-<plugin-name>

Notes:
- Scene Code is assigned by IM service, format: im-{32-char hex}
- Desc is used by the frontend to identify message types
- Ext is used for extension identification, format: interactive-{plugin-name}
```

### Step 4.3: Select Integration Method

Ask user about integration environment:

```
Select Integration Method

Please select the integration environment:
1. Local integration (requires starting frontend and backend services locally)
2. Online integration (use online environment, only develop frontend locally)

Your choice: [1/2]
```

**Option 1: Local Integration**

Provide startup commands:

```bash
# Terminal 1: Frontend plugin dev mode
pnpm --filter @xiaoe/live-<name>-sdk watch

# Terminal 2: Frontend scene dev
pnpm dev:live-room:h5

# Terminal 3: Backend service
cd <backend-project-path> && make run
```

**Option 2: Online Integration**

```
Online Integration Configuration

1. Frontend plugin uses local dev mode
   pnpm --filter @xiaoe/live-<name>-sdk watch

2. Frontend scene configure online API address
   In .env.development:
   VITE_API_BASE_URL=https://alive-interactive.xiaoeknow.com

3. Backend service uses online environment
   - Ensure xiaoe-bus MCP is deployed
   - Ensure gateway routes are configured and active

4. Access local frontend, call online API
   http://localhost:5176?api=https://alive-interactive.xiaoeknow.com
```

**Note**:
- Online integration is suitable for scenarios with insufficient local resources or when the backend is already deployed online
- Frontend develops locally, directly calls online backend API
- Ensure gateway routes are active, otherwise API requests will return 404

### Step 4.4: Test Checklist

Provide a test checklist for the user to check off:

```
Test Checklist

Frontend tests:
[ ] Plugin successfully installed to SDK
[ ] Can subscribe to state changes
[ ] UI components render correctly

Backend tests:
[ ] API endpoints are accessible (return 200)
[ ] Data successfully saved to database
[ ] IM messages sent successfully

IM message tests:
[ ] Frontend receives GET_CONTENT_MSG event
[ ] Desc and Ext fields correctly identified
[ ] Plugin correctly processes message

Integration tests:
[ ] Send question -> Student side displays
[ ] Submit answer -> Backend saves
[ ] End answering -> Statistics displayed
```

### Step 4.5: Debugging Tips

Provide common debugging commands:

**Frontend debugging**:
```javascript
// Browser console
window.<name>Plugin  // Plugin instance
window.liveRoomSdk   // SDK instance
<name>Plugin.getState()  // View state
```

**Backend debugging**:
```bash
# View logs
tail -f logs/business.log | grep -E "<Name>|<name>"

# View IM send logs
tail -f logs/business.log | grep "sendIM"
```

**Integration address**:
```
Interactive gateway domain: https://alive-interactive.xiaoeknow.com

API address: https://alive-interactive.xiaoeknow.com/api/v1/interactive/<name>/<endpoint>
```

### Step 4.6: Build and Publish Flow

**Frontend build**:
```bash
# Build plugin
pnpm --filter @xiaoe/live-<name>-sdk build

# Check build output
ls packages/plugins/plugin-<name>/dist/
```

**Publish steps**:
1. Update `package.json` version number
2. Build the plugin
3. Commit code
4. Push to remote
5. Publish to npm (if needed)

### Step 4.7: Deploy to xiaoe-bus MCP

**Note**: Backend services need to be deployed to xiaoe-bus MCP to serve external requests.

**Use xiaoe-bus MCP to create a deployment plan**:

```
xiaoe-bus MCP Deployment

Use xiaoe-bus MCP tool to create a deployment plan:

1. Query system information
   System name: alive-delivery-tools
   System ID: 2038
   Service port: 18383
   Apollo AppID: alive_delivery_tools

2. Create deployment plan
   Use xiaoe-bus MCP: create_deployment_branch
   Parameters:
   - taskEnv: dev/test/pro (environment)
   - branchName: dev/feature-<name> (deployment branch)
   - systemIds: [2038] (system ID)

3. Get deployment plan URL
   Deployment plan details: https://ops.xiaoe-tools.com/#/xiaoe_bus/workorders/detail/{iterationId}
   System details page: https://ops.xiaoe-tools.com/#/xiaoe_bus/systems
   Deployment plan list: https://ops.xiaoe-tools.com/#/xiaoe_bus/plans

4. Verify deployment
   After xiaoe-bus deployment completes, service goes online automatically
   Verification URL: https://alive-interactive.xiaoeknow.com/api/v1/interactive/<name>/health
```

**Deployment Configuration Notes**:
| Configuration Item | Value                   | Description            |
|--------------------|-------------------------|------------------------|
| System ID          | 2038                    | Delivery tools system  |
| System name        | alive-delivery-tools    | System English name    |
| Service port       | 18383                   | Go service port        |
| Apollo AppID       | alive_delivery_tools    | Configuration center AppID |
| Deployment branch  | dev/feature-<name>      | Feature branch         |
| Deployment cluster | [1, 2]                  | 1=production 2=testing |

**Important: Items requiring abner's assistance**

The following items require contacting abner for creation/configuration:
1. **Database (DB)**
   - Create new database
   - Database permission configuration
   - Execute table creation SQL

2. **Backend Configuration (Apollo/Nacos)**
   - IM service address
   - Database connection configuration
   - Redis configuration
   - Other environment variable configuration

3. **IM Scene Registration**
   - Apply for IM scene Code
   - Register scene with IM service
   - Configure message push rules

4. **xiaoe-bus Deployment**
   - Create deployment plan
   - Configure deployment cluster
   - Set canary rules (if needed)

**Deployment methods**:
- Development environment: Deploy directly to development MCP Bus
- Testing environment: Deploy via CI/CD pipeline
- Production environment: Requires approval before deployment

### Step 4.8: Completion Summary

Use terminal interactive style:

```
--------------------------------------------------------------
  Plugin Development Complete!
--------------------------------------------------------------

Full checklist:
  Stage 1: Frontend Plugin Creation
     -> packages/plugins/plugin-<name>/

  Stage 2: Gateway Configuration
     -> Route forwarding rules generated

  Stage 3: Backend Service Development
     -> domain/dto.go
     -> domain/service/<name>_service.go
     -> server/<name>.go

  Stage 4: Integration Testing & Deployment
     -> Tests passed, build successful
     -> xiaoe-bus deployment complete

--------------------------------------------------------------

Full documentation saved to:
   .claude/skills/plugin-dev-guide/SKILL.md

Tips:
   - View README: packages/plugins/plugin-<name>/README.md
   - View examples: packages/plugins/plugin-<name>/demo/
   - View full guide: .claude/skills/plugin-dev-guide/SKILL.md
   - xiaoe-bus deployment: https://ops.xiaoe-tools.com/#/xiaoe_bus/plans
```

---

## Shortcut Commands

```bash
# One-click full flow (for experienced users)
/plugin-dev-guide <name> --full

# Execute only a specific stage
/plugin-dev-guide <name> --stage 1  # Frontend creation only
/plugin-dev-guide <name> --stage 2  # Backend development only
/plugin-dev-guide <name> --stage 3  # Gateway configuration only
/plugin-dev-guide <name> --stage 4  # Integration testing only
```

---

## Reference Documentation

- Plugin integration guide: `.claude/specs/plugin-integration-guide.md`
- Plugin development specification: `.claude/specs/plugin-development.md`
- Plugin README example: `packages/plugins/plugin-answer-card/README.md`

---

## Rules

- **Step-by-step guidance** - Ask user whether to continue after each stage
- **Do not skip steps** - Ensure critical steps are not missed even if the user requests it
- **Provide choices** - Offer options at key decision points for the user to decide
- **Save progress** - Record user's current progress for resuming later
- **Provide rollback** - User can go back to the previous step to reconfigure at any time
