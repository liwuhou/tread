---
name: figma-use
description: Isolate Figma MCP calls via sub-agent to prevent the main agent context from being filled with design data. All Figma operations (downloading icons, generating specs, extracting information) use this skill uniformly.
type: user-invocable
context: fork
---

# Figma Use — Sub-Agent Isolation Mode

All Figma MCP calls are executed in isolation through a sub-agent, protecting the main agent context.

## Core Principle

**Figma raw data must not enter the main agent context.** All MCP calls, data parsing, and file generation are completed within the sub-agent; the main agent only receives concise results.

## Workflow

### Step 1: Parse User Request

Extract from user input:
- **Task type**: `download` (download icons) | `spec` (generate spec document) | `extract` (extract information)
- **Figma link**: extract `fileKey` and `nodeId`
- **Additional parameters**: output path, change name, etc.

### Step 2: Launch Sub-Agent

Use the `Agent` tool (`subagent_type: "general-purpose"`) to launch a sub-agent with the following prompt:

```
You are a Figma design data operations assistant. Complete the following task using Figma MCP and REST API.

**Task type**: <download|spec|extract>
**Task description**: <specific description>
**Figma link**: <figma-link>
**fileKey**: <fileKey>
**nodeId**: <nodeId>
**Output path**: <output-path>

**Core rules (must be followed)**:
1. All Figma MCP calls are completed inside the sub-agent; do not return raw data
2. Preserve original Figma color values, viewBox, and file structure; do not make any subjective replacements
3. Only remove CSS variable wrappers (var(--fill-0, X) → keep X); do not change original colors
4. If Figma has N independent assets, keep N independent files; do not merge
5. Each SVG keeps its original viewBox; do not perform coordinate transformations
6. Use CSS to compose multiple SVG assets; do not modify SVG internal structure
7. **Do not add files that do not exist in the Figma design file**

**SVG download strategy (try in priority order)**:

**Strategy 1: Figma MCP download_figma_images (preferred)**
- Use `mcp__figma__download_figma_images` to download SVG
- If 0 images are returned, proceed to Strategy 2

**Strategy 2: Figma REST API export instance nodes (fallback when MCP fails)**

Figma MCP SVG export returns 0 images for COMPONENT type nodes. This is because COMPONENT is a component definition; you need to find its INSTANCE nodes on the page to export.

Steps:
a) Read the Figma Personal Access Token from the `.mcp.json` file in the project root (in the `--figma-api-key=` parameter within `args`)
b) Use the REST API to query the design page node tree and find INSTANCE nodes of the target COMPONENT:
   ```bash
   curl -s -H "X-Figma-Token: <TOKEN>" \
     "https://api.figma.com/v1/files/<FILEKEY>/nodes?ids=<PAGE_NODE_ID>&depth=8" | python3 -c "
   import sys,json
   d=json.load(sys.stdin)
   target_components = {<COMPONENT_ID>: '<FILENAME>', ...}  # Mapping from target component IDs to filenames
   def find_instances(node, results):
       cid = node.get('componentId','')
       if cid in target_components:
           results[target_components[cid]] = node.get('id','')
       for child in node.get('children',[]):
           find_instances(child, results)
   results = {}
   for nid, info in d.get('nodes',{}).items():
       find_instances(info.get('document',{}), results)
   for name, iid in results.items():
       print(name + '=' + iid)
   "
   ```
c) Get the SVG download URL using the instance node ID:
   ```bash
   curl -s -H "X-Figma-Token: <TOKEN>" \
     "https://api.figma.com/v1/images/<FILEKEY>?ids=<INSTANCE_IDS>&format=svg"
   ```
d) Download SVG files using Python (node IDs contain special characters; must use Python instead of bash associative arrays):
   ```python
   import urllib.request, json, os
   # ... Get images URL mapping, download each via urllib.request.urlretrieve
   ```
e) Verify each file starts with `<svg`

**Strategy 3: If the REST API also fails, return an error message; do not use any alternative approaches**
- Do not use potrace tracing, hand-written SVG, PNG-to-SVG conversion, or other alternatives
- Return specific error information for the main agent to report to the user

**Return format**:
Return concise results; do not include complete React code or design context:
- Download task: list of file paths + strategy number used
- Spec task: generated file path + key data summary
- Extract task: structured summary
- Error: specific error information + strategies already attempted

Please execute the task and return the results.
```

### Step 3: Wait for Sub-Agent to Return Results

Sub-agent result example:

```
## Sub-Agent Result

**Task**: Download hangup icon
**Status**: Success (Strategy 2: REST API)

### File List
- `apps/scenes/1v1/h5/src/assets/icons/hangup.svg` (1085 bytes, Figma native SVG)
- `apps/scenes/1v1/h5/src/assets/icons/mic.svg` (1080 bytes, Figma native SVG)
```

### Step 4: Execute Follow-up Actions

Based on the sub-agent results:

1. **After a download task completes, you must generate `.figma-manifest.json`**:
   Run the generate-figma-manifest script, locating it by searching in order:
   - `.claude/hooks/generate-figma-manifest.sh`
   - `.claude/live-devkit/hooks/generate-figma-manifest.sh`
   - `./node_modules/@xiaoe/live-devkit/hooks/generate-figma-manifest.sh`
   Use whichever exists first: `<resolved-path> <icons-dir> <figma-fileKey>`
   This generates a manifest file in the icons directory, recording SHA256 hashes of all exported SVGs,
   and sets the files to read-only (chmod 444).

2. Update reference files (e.g., design.md)
3. Notify the user of completion status
4. If there are errors, report them to the user

## Task Types in Detail

### download (Download Icons)

**Input**: Node ID + output directory
**Sub-agent operations**:
1. Try Strategy 1: `download_figma_images` to download assets
2. If it fails, try Strategy 2: REST API to find instance nodes and export SVG
3. If both fail, try Strategy 3: return error
4. Clean up CSS variable wrappers (preserve original color values)
5. Return file list + strategy used

### spec (Generate Spec Document)

**Input**: Node ID + change name
**Sub-agent operations**:
1. `get_design_context` to get node tree, styles, components
2. Parse layout structure, colors, fonts, spacing, effects
3. Generate `design-spec.md` file
4. Return file path + key data summary (element count, color varieties, etc.)

### extract (Extract Information)

**Input**: Node ID + extraction target
**Sub-agent operations**:
1. `get_design_context` to get node information
2. Extract specified information (dimensions, colors, spacing, etc.)
3. Return structured summary

## Error Handling

When the sub-agent encounters errors:
1. Report specific error information (MCP connection failure, node does not exist, etc.)
2. Do not guess or attempt to fix (to avoid generating incorrect files)
3. **Do not use potrace, manual drawing, PNG-to-SVG, or other alternatives to generate SVG**
4. The main agent reports the error information to the user

## Figma Asset Protection Rules

**NEVER write SVG code yourself.** All SVG assets must be downloaded from Figma via the download strategies above. If download fails, report the error to the user — do not fall back to hand-writing SVG, potrace tracing, PNG-to-SVG conversion, or any other alternative.

Protection layers for downloaded SVG assets:

1. **Manifest tracking**: After downloading, run the generate-figma-manifest script (found by searching `.claude/hooks/`, `.claude/live-devkit/hooks/`, `./node_modules/@xiaoe/live-devkit/hooks/` in order) to generate `.figma-manifest.json` recording SHA256 hashes of all SVG files, and set files to read-only (chmod 444)
2. **OS-level read-only protection**: chmod 444 makes the file unwritable by default
3. **Modification requires explicit authorization**: To modify a Figma-exported SVG, you must manually delete the entry from `.figma-manifest.json` + manually run chmod 644

## Known Issues

- **Figma MCP SVG export fails for COMPONENT nodes**: Returns 0 images. Reason: COMPONENT is a component definition (remote=true); its visual content is not in the current file. Solution: Find INSTANCE nodes on the page via REST API to export
- **REST API also returns null for COMPONENT nodes**: Same reason as MCP. You must use INSTANCE instance node IDs
- **Figma Personal Access Token location**: In the project root `.mcp.json` file, within the `--figma-api-key=` parameter of `args`

## Applicable Scenarios

- Downloading icons and assets
- Generating design spec documents
- Extracting layout, color, and font information
- Batch processing multiple nodes
- Verifying design file reproduction accuracy

## Not Applicable Scenarios

- Simple Figma link validation
- Only need to confirm whether a design file exists
- Design file information is already known; only need to reference the link
