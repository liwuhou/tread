# Plugin Dev Guide - Plugin Development Guided Skill

> Step-by-step guided plugin development skill, covering the full flow from creation to deployment.

## Usage

### Basic Usage

```bash
/plugin-dev-guide <plugin-name> [plugin-chinese-name]
```

**Examples**:
```bash
# Create answer card plugin
/plugin-dev-guide answer-card Answer Card

# Create lottery plugin
/plugin-dev-guide lottery Lottery

# Create gift plugin
/plugin-dev-guide gift Gift
```

### Shortcut Commands

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

---

## Stage 1: Frontend Plugin Creation

### Deliverables

- Create standard plugin package structure
- Generate TypeScript scaffold code
- Create multi-platform demos (optional)
- Add build scripts

### Output Directory

```
packages/plugins/plugin-<name>/
+-- package.json
+-- tsconfig.json
+-- src/
|   +-- index.ts
|   +-- <Name>Plugin.ts
+-- demo/
    +-- vue3/
    +-- vue2/
    +-- miniprogram/
```

---

## Stage 2: Gateway Configuration

### Prerequisites (Backend Environment Preparation)

Since gateway configuration and backend service development are both done in the same project (alive-delivery-tools),
first prepare the project environment:

1. Configure project path
2. Find/Clone backend system (using alive-system-bridge)
3. Switch development branch (user selects branch name)
4. Confirm environment configuration

### Deliverables

- Generate gateway route configuration (YAML)
- Provide configuration flow instructions

### Configuration Method

Add routing rules in the `gateway_interact` service configuration file:

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

### Configuration File Locations

| Environment | Configuration File |
|-------------|-------------------|
| Development | `gateway-develop.yaml` |
| Testing | `gateway-test.yaml` |
| Production | `gateway-production.yaml` |
| Docker deployment | `gateway-docker.yaml` |
| K8s deployment | `gateway-k8s.yaml` |

### Configuration Path

```
gateway_interact/cmd/gateway/resources/gateway-{env}.yaml
```

### Common Filters

| Filter | Description |
|--------|-------------|
| `AuthFilter=1` | Enable authentication |
| `CrossFilter=1` | Enable CORS |
| `StripPrefix=N` | Strip first N path prefixes |
| `AliveLimiterFilter=1` | Rate limiting |
| `UserLimiterFilter=1` | User-level rate limiting |
| `IpLimiterFilter=1` | IP-level rate limiting |

---

## Stage 3: Backend Service Development

> **Prerequisite**: Stage 2 has completed backend project environment preparation (project path, branch switch, environment check)

### Deliverables

- Define DTOs and entities
- Implement business logic (Service)
- Implement IM client
- Implement API handlers
- Register routes

### Output Files

```
alive-delivery-tools/
+-- domain/
|   +-- dto.go
|   +-- interactive_<name>.go
|   +-- repo.go
|   +-- service/<name>_service.go
+-- infra/repo/client_service/
|   +-- alive_<name>_im.go
+-- app/<name>.go
+-- server/<name>.go
```

---

## Stage 4: Integration Testing & Deployment

### Deliverables

- Provide local/online integration method selection
- Provide startup commands
- Provide test checklist
- Provide debugging tips
- Build and publish
- Deploy to MCP Bus

### Integration Method Selection

| Method | Applicable Scenario | Advantages | Disadvantages |
|--------|-------------------|------------|---------------|
| **Local integration** | Full local environment available | Easy debugging, fast response | Need to start multiple services |
| **Online integration** | Insufficient local resources or backend already deployed | Only run frontend locally, lightweight | Depends on online environment, slightly slower debugging |

### Integration Address

```
Interactive gateway domain: https://alive-interactive.xiaoeknow.com

API address: https://alive-interactive.xiaoeknow.com/api/v1/interactive/<name>/<endpoint>
```

### Test Checklist

- [ ] Frontend plugin installed successfully
- [ ] API endpoints are accessible
- [ ] IM message send/receive works correctly
- [ ] Data persistence works correctly
- [ ] Gateway routing works correctly
- [ ] MCP Bus deployment successful

---

## Reference Documentation

| Document | Location |
|----------|----------|
| Full integration guide | `.claude/specs/plugin-integration-guide.md` |
| Development specification | `.claude/skills/plugin-dev-guide/references/style-guide.md` |
| Code templates | `.claude/skills/plugin-dev-guide/references/templates.md` |

---

## FAQ

### Q: How to plan plugin requirements?

It is recommended to use `/openspec-new-change` to generate a plugin requirements document, clarifying:
- Feature scope and API design
- DTO and entity definitions
- API endpoint list
- IM message format

### Q: How to skip a stage?

Use the `--stage` parameter to execute only the specified stage:
```bash
/plugin-dev-guide <name> --stage 1  # Frontend creation only
```

### Q: What if the backend project path is not the default?

You will be asked during Stage 2, and you can customize the path.

### Q: Can the branch name be customized?

Yes, Stage 2 will let you enter the branch name. The recommended format is `dev/feature-<plugin-name>`.

### Q: How to add custom logic?

The scaffold code has `TODO` comments marking where you should implement based on business requirements.

### Q: How to exit the guide?

Enter `n` or `cancel` at any time to exit the guide. Progress is saved automatically.

### Q: How to deploy the backend service?

Use the `/alive-system-bridge` skill to create a xiaoe-bus deployment plan:

1. **Create system**
   ```
   /alive-system-bridge create system
   ```
   Fill in information:
   - System name: alive-delivery-tools
   - Branch name: dev/feature-<name> (the branch you created in Stage 2)
   - Service port: 18383

2. **Add system to xiaoe-bus plan**
   Add alive-delivery-tools in the xiaoe-bus MCP system

3. **Verify deployment**
   After xiaoe-bus deployment completes, the service goes online automatically

**Important: Items requiring abner's assistance**

The following items require contacting abner for creation/configuration:
1. **Database (DB)**
   - Create new database
   - Database permission configuration

2. **Backend Configuration (Apollo/Nacos)**
   - IM service address
   - Other environment variable configuration

3. **IM Scene Registration**
   - Apply for IM scene Code (e.g. im-3e5b0683400ceef96)
   - Register scene with IM service

---

## Changelog

### v1.7.2
- Fixed xiaoe-bus system domain: `https://ops.xiaoe-tools.com` (previously written as ops.xiaoebus.xiaoeknow.com)

### v1.7.1
- Fixed xiaoe-bus deployment URL: `https://ops.xiaoebus.xiaoeknow.com/#/xiaoe_bus/plans`
- Fixed deployment plan details URL: `https://ops.xiaoebus.xiaoeknow.com/#/xiaoe_bus/workorders/detail/{iterationId}`
- Fixed system details page URL: `https://ops.xiaoebus.xiaoeknow.com/#/xiaoe_bus/systems`

### v1.7.0
- Stage 3 adds database table confirmation step, first confirm whether new tables are needed
- Stage 4 adds table structure design output (referencing live marketing delivery database specification)
- Stage 4 adds backend configuration checklist (Apollo/Nacos, IM scene registration)
- Stage 4 integrates xiaoe-bus MCP deployment plan creation capability
- Added xiaoe-bus deployment plan URL and configuration instructions
- Completed abner assistance checklist (database, configuration, IM scene, xiaoe-bus deployment)

### v1.6.4
- Output stage title at the beginning of each Stage (e.g. "Stage 1: Frontend Plugin Creation")
- Enhanced UI progress display, clearer step-by-step guidance

### v1.6.3
- Optimized Stage 1.1 information confirmation flow, more concise output
- Added OpenSpec planning guidance (optional skip)
- Support user providing task.md path to continue after planning

### v1.6.0
- Stage 4.1 adds integration method selection (local integration / online integration)
- Online integration suitable for insufficient local resources or when backend is already deployed online

### v1.5.0
- Updated production network service load address: `http://alive-delivery-tools-master.alive-delivery-tools.svc.cluster.local`
- Changed abner assistance items to: Database (DB), Backend Configuration, IM Scene Registration

### v1.4.0
- Added interactive gateway domain: https://alive-interactive.xiaoeknow.com
- Recommended using `/openspec-new-change` to generate plugin requirements document

### v1.3.0
- Removed gateway verification step, routes take effect automatically after gateway starts
- Changed MCP deployment to use /alive-system-bridge to create xiaoe-bus plan
- Added abner assistance configuration instructions (backend configuration, IM scene registration)

### v1.2.0
- Adjusted Stage 2 to "Gateway Configuration + Backend Environment Preparation"
- Moved backend project preparation earlier: project path -> Find/Clone -> Switch branch (user choice) -> Environment check
- Gateway configuration and backend development are in the same project, environment preparation is reused

### v1.1.0
- Added backend prerequisite steps: project path configuration, system clone, branch switch, environment verification
- Added MCP Bus deployment step (Stage 4.5)
- Fixed step numbering duplication issue

### v1.0.0
- Initial version
- Supports 4-stage full flow
- Provides backend code templates
- Provides gateway configuration templates
