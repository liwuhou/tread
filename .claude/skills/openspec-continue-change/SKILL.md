---
name: openspec-continue-change
description: Continue working on an OpenSpec change by creating the next artifact. Use when the user wants to progress their change, create the next artifact, or continue their workflow.
license: MIT
compatibility: Requires openspec CLI.
metadata:
  author: openspec
  version: "1.0"
  generatedBy: "1.2.0"
---

Continue working on a change by creating the next artifact.

**Input**: Optionally specify a change name. If omitted, check if it can be inferred from conversation context. If vague or ambiguous you MUST prompt for available changes.

**Steps**

1. **If no change name provided, prompt for selection**

   Run `openspec list --json` to get available changes sorted by most recently modified. Then use the **AskUserQuestion tool** to let the user select which change to work on.

   Present the top 3-4 most recently modified changes as options, showing:
   - Change name
   - Schema (from `schema` field if present, otherwise "spec-driven")
   - Status (e.g., "0/5 tasks", "complete", "no tasks")
   - How recently it was modified (from `lastModified` field)

   Mark the most recently modified change as "(Recommended)" since it's likely what the user wants to continue.

   **IMPORTANT**: Do NOT guess or auto-select a change. Always let the user choose.

2. **Check current status**
   ```bash
   openspec status --change "<name>" --json
   ```
   Parse the JSON to understand current state. The response includes:
   - `schemaName`: The workflow schema being used (e.g., "spec-driven")
   - `artifacts`: Array of artifacts with their status ("done", "ready", "blocked")
   - `isComplete`: Boolean indicating if all artifacts are complete

3. **Act based on status**:

   ---

   **If all artifacts are complete (`isComplete: true`)**:
   - Congratulate the user
   - Show final status including the schema used
   - Suggest: "All artifacts created! You can now implement this change or archive it."
   - STOP

   ---

   **If artifacts are ready to create** (status shows artifacts with `status: "ready"`):
   - Pick the FIRST artifact with `status: "ready"` from the status output
   - Get its instructions:
     ```bash
     openspec instructions <artifact-id> --change "<name>" --json
     ```
   - Parse the JSON. The key fields are:
     - `context`: Project background (constraints for you - do NOT include in output)
     - `rules`: Artifact-specific rules (constraints for you - do NOT include in output)
     - `template`: The structure to use for your output file
     - `instruction`: Schema-specific guidance
     - `outputPath`: Where to write the artifact
     - `dependencies`: Completed artifacts to read for context
   - **Create the artifact file**:
     - Read any completed dependency files for context
     - Use `template` as the structure - fill in its sections
     - Apply `context` and `rules` as constraints when writing - but do NOT copy them into the file
     - Write to the output path specified in instructions

     **If creating a spec artifact (MANDATORY design-spec bridge)**:
     Before writing the spec, check if `design-spec.md` exists in the change directory:
     ```bash
     ls openspec/changes/<name>/design-spec.md 2>/dev/null
     ```
     If `design-spec.md` exists AND the spec is for a UI-related capability:
     1. Read `design-spec.md`
     2. Extract design tokens from the following sections:
        - **Color Summary** table → color tokens (color value + usage)
        - **Font Summary** table → font tokens (family + weight + size + line height + usage)
        - **Spacing & Border Radius Summary** table → spacing/border-radius tokens (value + usage)
        - **Key Elements** table → layout constraints (element + size + position)
     3. Inject a "Design Tokens" section into the spec requirements:
        ```markdown
        ### Requirement: Design Tokens
        The UI SHALL implement the following design tokens from Figma design-spec:

        #### Colors
        | Token | Value | Usage |
        |-------|-------|-------|
        | ... | ... | ... |

        #### Fonts
        | Token | Family | Weight | Size | Line Height | Usage |
        |-------|--------|--------|------|-------------|-------|
        | ... | ... | ... | ... | ... | ... |

        #### Spacing & Border Radius
        | Token | Value | Usage |
        |-------|-------|-------|
        | ... | ... | ... |

        #### Scenario: Design tokens are correctly applied
        - **WHEN** UI components are rendered
        - **THEN** colors, fonts, spacing, and border-radius values SHALL match the design tokens listed above
        ```
     If `design-spec.md` does NOT exist, or the spec is for a non-UI capability (state machine, data model, etc.), skip this injection step.

     **If creating a spec artifact (interaction-demo bridge)**:
     Also check if `interaction-demo.html` exists in the change directory:
     ```bash
     ls openspec/changes/<name>/interaction-demo.html 2>/dev/null
     ```
     If `interaction-demo.html` exists AND the spec is for a UI-related capability:
     1. Read `interaction-demo.html` and extract interaction patterns:
        - Component states and triggers
        - State transition animations and timing
        - Edge cases handled
        - Keyboard interactions
     2. Inject an "Interaction Patterns" section into the spec requirements (see openspec-propose skill for the full template).
     If `interaction-demo.html` does NOT exist, or the spec is for a non-UI capability, skip this injection step.

   - Show what was created and what's now unlocked
   - STOP after creating ONE artifact

   ---

   **If no artifacts are ready (all blocked)**:
   - This shouldn't happen with a valid schema
   - Show status and suggest checking for issues

4. **After creating an artifact, show progress**
   ```bash
   openspec status --change "<name>"
   ```

**Output**

After each invocation, show:
- Which artifact was created
- Schema workflow being used
- Current progress (N/M complete)
- What artifacts are now unlocked
- Prompt: "Want to continue? Just ask me to continue or tell me what to do next."

**Artifact Creation Guidelines**

The artifact types and their purpose depend on the schema. Use the `instruction` field from the instructions output to understand what to create.

Common artifact patterns:

**spec-driven schema** (proposal → specs → design → tasks):
- **proposal.md**: Ask user about the change if not clear. Fill in Why, What Changes, Capabilities, Impact.
  - The Capabilities section is critical - each capability listed will need a spec file.
- **specs/<capability>/spec.md**: Create one spec per capability listed in the proposal's Capabilities section (use the capability name, not the change name).
- **design.md**: Document technical decisions, architecture, and implementation approach.
- **tasks.md**: Break down implementation into checkboxed tasks.

For other schemas, follow the `instruction` field from the CLI output.

**Guardrails**
- Create ONE artifact per invocation
- Always read dependency artifacts before creating a new one
- Never skip artifacts or create out of order
- If context is unclear, ask the user before creating
- Verify the artifact file exists after writing before marking progress
- Use the schema's artifact sequence, don't assume specific artifact names
- **IMPORTANT**: `context` and `rules` are constraints for YOU, not content for the file
  - Do NOT copy `<context>`, `<rules>`, `<project_context>` blocks into the artifact
  - These guide what you write, but should never appear in the output
