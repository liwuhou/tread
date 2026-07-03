---
name: figma-spec
description: >
  Parse Figma design files and generate UI specification documents.
  Must use this skill when the user mentions "Figma design file", "design file annotation", "pixel-perfect reproduction", "UI spec",
  "develop from design file", or any scenario requiring detailed annotation information from Figma.
  Also applicable when design file information needs to be persisted into the project knowledge base.
compatibility: Requires Figma MCP and Agent tools
---

# Figma Spec Generator

Parse Figma design files and generate detailed UI specification documents to support pixel-perfect reproduction.

**Core principle: All Figma MCP calls are executed in isolation via sub-agent to prevent the main agent context from being filled with design data.**

## Parameters

`$ARGUMENTS` — Format: `<figma-link> [--change <change-name>]`

- **figma-link**: Figma design file link (must include node-id parameter)
- **--change <change-name>**: Optional, associates with an existing OpenSpec change

**If the `--change` parameter is not provided, the user will be prompted to create a change first or specify an output path.**

---

## Workflow

### Step 1: Parse Figma Link

Extract from the user-provided link:
- `fileKey`: Figma file ID
- `nodeId`: Design file node ID

**Link format**:
```
https://www.figma.com/design/<fileKey>/<filename>?node-id=<nodeId>&m=dev
```

### Step 2: Launch Sub-Agent to Fetch Figma Data

Use the `Agent` tool (`subagent_type: "general-purpose"`) to launch a sub-agent with the following prompt:

```
You are a Figma design data extraction assistant. Please use Figma MCP to complete the following task:

**Task type**: spec (generate design specification document)
**Figma link**: <figma-link>
**fileKey**: <fileKey>
**nodeId**: <nodeId>
**Change name**: <change-name>

**Core rules (must be followed)**:
1. Use mcp__figma__get_figma_data to get node tree structure, global styles, and component definitions
2. All Figma MCP calls are completed inside the sub-agent; do not return raw data
3. Parse layout structure, colors, fonts, spacing, effects, and other key information
4. Generate design-spec.md file at openspec/changes/<change-name>/design-spec.md
5. Preserve original Figma color values, viewBox, and file structure; do not make any subjective replacements

**design-spec.md template**:
```markdown
## Figma Design File Annotation

**Source**: <filename>
**Node**: <nodeId>
**Canvas size**: <width>x<height>

---

## Layout Structure

```
[ASCII layout diagram]
```

### Key Elements

| Element name | Type | Position | Size | Notes |
|---------|------|------|------|------|
| ... | ... | ... | ... | ... |

---

## Color Summary

| Usage | Color value |
|------|------|
| ... | ... |

---

## Font Summary

| Usage | Font | Weight | Size | Line height | Color |
|------|------|------|------|------|------|
| ... | ... | ... | ... | ... | ... |

---

## Spacing and Border Radius

| Element | Spacing/Radius value |
|------|-------------|
| ... | ... |

---

## Effects Summary

| Element | Shadow/Blur effect |
|------|--------------|
| ... | ... |
```

**Return format**:
Return concise results; do not include complete React code or design context:
- File path: openspec/changes/<change-name>/design-spec.md
- Key data summary: element count, color varieties, font varieties, etc.
- Error: specific error information

Please execute the task and return the results.
```

### Step 3: Wait for Sub-Agent to Return Results

Sub-agent result example:

```
## Sub-Agent Result

**Task**: Generate design specification document
**Status**: Success

### File List
- `openspec/changes/onevone-live-scene/design-spec.md` (Elements: 25, Colors: 8, Fonts: 3)
```

### Step 4: Update design.md

Add to the `## Figma Design File` section of `design.md`:

```markdown
For detailed annotation information, see: [`design-spec.md`](./design-spec.md)

**Key pages:**
- **<Page name>**: [Figma](<original-link>) - <brief description>
```

### Step 5: Clean Up Processed Links

Clean up processed links in `.claude/figma-links.json`.

---

## Rules

- **Must confirm change directory exists first** - If the `--change` parameter is provided, check whether the directory exists
- **Link must include node-id** - Otherwise prompt the user to provide it
- **Color extraction requires format conversion** - Convert Figma rgba to CSS format
- **Layout analysis must include ASCII diagram** - For quick structural understanding
- **Font information must be complete** - Include font family, weight, size, line height, and color
- **Do not overwrite existing files** - If `design-spec.md` already exists, prompt the user
- **Figma data must not enter main context** - All MCP calls are completed in the sub-agent
- **NEVER write SVG code yourself** - All SVG assets must be downloaded from Figma (use `/figma-use download`), never hand-written or converted from other formats

---

## Integration with OpenSpec

### During `/opsx:explore` Phase

If the user mentions having a Figma design file:
1. Record the design file link
2. Prompt the user to call `/figma-spec` after `/opsx:propose`

### After `/opsx:propose`

Auto-prompt:
> "A Figma design file was mentioned during the explore phase. Would you like to call `/figma-spec` to parse the design file?"

### During `/opsx:apply` Phase

Automatically read `design-spec.md` (if it exists) as an implementation reference.

---

## Examples

### Basic Usage

```
/figma-spec https://www.figma.com/design/ABC123/file?node-id=100-200
```

### Associated with a Change

```
/figma-spec https://www.figma.com/design/ABC123/file?node-id=100-200 --change onevone-live-scene
```

### Multi-Node Parsing

```
/figma-spec https://www.figma.com/design/ABC123/file?node-id=100-200,100-300 --change onevone-live-scene
```

---

## Output File Structure

```
openspec/changes/<change-name>/
├── proposal.md
├── design.md          ← Auto-updated, adds reference to design-spec.md
├── design-spec.md     ← Newly generated Figma spec document
├── specs/
└── tasks.md
```
