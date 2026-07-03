---
name: openspec-propose
description: Propose a new change with all artifacts generated in one step. Use when the user wants to quickly describe what they want to build and get a complete proposal with design, specs, and tasks ready for implementation.
license: MIT
compatibility: Requires openspec CLI.
metadata:
  author: openspec
  version: "1.0"
  generatedBy: "1.2.0"
---

Propose a new change - create the change and generate all artifacts in one step.

I'll create a change with artifacts:
- proposal.md (what & why)
- design.md (how)
- tasks.md (implementation steps)

When ready to implement, run /opsx:apply

---

**Input**: The user's request should include a change name (kebab-case) OR a description of what they want to build.

**Steps**

1. **If no clear input provided, ask what they want to build**

   Use the **AskUserQuestion tool** (open-ended, no preset options) to ask:
   > "What change do you want to work on? Describe what you want to build or fix."

   From their description, derive a kebab-case name (e.g., "add user authentication" → `add-user-auth`).

   **IMPORTANT**: Do NOT proceed without understanding what the user wants to build.

2. **Figma design check (MANDATORY — do NOT skip this step)**

   Before creating the change, perform two checks IN ORDER:

   a. **FIRST**: Check `.claude/figma-links.json` for pending Figma links:
   ```bash
   cat .claude/figma-links.json 2>/dev/null || echo '{"pending":[]}'
   ```

   b. **If pending links exist**: Skip the user question (step c) — auto-process the pending links.
      - Inform the user: "Detected {N} pending Figma design file links, auto-parsing..."
      - Mark all pending links for `/figma-spec` invocation (will be processed in step 4)
      - Proceed directly to step 3

   c. **If NO pending links exist**: Ask the user about Figma designs:
   Use the **AskUserQuestion tool** to ask:
   > "Do you have a Figma design for this change? If so, please provide the link (format: https://www.figma.com/design/xxx?node-id=xxx). I'll auto-parse it and generate design-spec.md after creating the change."

   - If user provides a Figma link → record it, mark for `/figma-spec` invocation
   - If user confirms no Figma → skip Figma step, continue normal flow

3. **Create the change directory**
   ```bash
   openspec new change "<name>"
   ```
   This creates a scaffolded change at `openspec/changes/<name>/` with `.openspec.yaml`.

4. **Invoke /figma-spec to parse design (if Figma links exist)**

   If step 2 detected Figma links (from figma-links.json or user-provided):
   - For each link, invoke the `figma-spec` skill with the link and change name
   - This generates `design-spec.md` into the change directory
   - After processing, clean up handled links from `.claude/figma-links.json`

5. **Get the artifact build order**
   ```bash
   openspec status --change "<name>" --json
   ```
   Parse the JSON to get:
   - `applyRequires`: array of artifact IDs needed before implementation (e.g., `["tasks"]`)
   - `artifacts`: list of all artifacts with their status and dependencies

6. **Create artifacts in sequence until apply-ready**

   Use the **TodoWrite tool** to track progress through the artifacts.

   Loop through artifacts in dependency order (artifacts with no pending dependencies first):

   a. **For each artifact that is `ready` (dependencies satisfied)**:
      - Get instructions:
        ```bash
        openspec instructions <artifact-id> --change "<name>" --json
        ```
      - The instructions JSON includes:
        - `context`: Project background (constraints for you - do NOT include in output)
        - `rules`: Artifact-specific rules (constraints for you - do NOT include in output)
        - `template`: The structure to use for your output file
        - `instruction`: Schema-specific guidance for this artifact type
        - `outputPath`: Where to write the artifact
        - `dependencies`: Completed artifacts to read for context
      - Read any completed dependency files for context
      - Create the artifact file using `template` as the structure
      - Apply `context` and `rules` as constraints - but do NOT copy them into the file

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
         - What component states exist? (loading / empty / error / data / hover / active / disabled / ...)
         - What triggers state transitions? (click / input / timer / API response / ...)
         - What transition animations are used? (fade / slide / expand / collapse / ...)
         - What edge cases are handled? (debounce / double-click / empty value / concurrent operations / ...)
         - What keyboard interactions exist? (Tab / Enter / Escape / arrow keys / ...)
      2. Inject an "Interaction Patterns" section into the spec requirements:
         ```markdown
         ### Requirement: Interaction Patterns
         The UI SHALL implement the following interaction patterns verified in the interaction demo:

         #### Component States
         | State | Trigger | Visual Result |
         |-------|---------|---------------|
         | loading | Page mount / retry action | Skeleton / spinner placeholder |
         | empty | Data is empty (length === 0) | Empty state message + suggested action |
         | error | API failure / network error | Error message + retry button |
         | data | API success with data | Render item list / content |
         | hover | Mouse enters interactive element | Background / border color change |
         | active | Mouse down / touch start | Scale or color change |
         | disabled | Missing permission / in-progress | Grayed out, pointer-events: none |
         | expanded | Click on collapsed item | Detail content revealed |

         #### State Transitions
         | From | To | Transition | Duration |
         |------|----|-----------|----------|
         | loading | data | fade out skeleton, fade in content | 300ms |
         | collapsed | expanded | slideDown | 250ms |
         | expanded | collapsed | slideUp | 200ms |
         | idle | submitting | button → spinner, disable button | 100ms |

         #### Edge Cases
         - **Rapid double-click**: Debounce 300ms on actionable buttons
         - **Concurrent toggle**: Only one item expanded at a time; expanding another collapses the previous
         - **Network timeout**: Show timeout error after 10s, offer retry
         - **Long content**: Truncate with ellipsis, show tooltip on hover

         #### Keyboard Support
         | Key | Context | Action |
         |-----|---------|--------|
         | Tab | Global | Navigate between focusable elements |
         | Enter | On focused button / link | Trigger click |
         | Escape | Modal / popover open | Close modal / popover |
         | Arrow keys | List focused | Navigate between list items |

         #### Scenario: All interaction states are correctly implemented
         - **WHEN** the component transitions between any two states listed above
         - **THEN** the transition animation, timing, and visual result SHALL match the behavior demonstrated in the interaction demo
         ```
      If `interaction-demo.html` does NOT exist, or the spec is for a non-UI capability, skip this injection step.

      - Show brief progress: "Created <artifact-id>"

   b. **Continue until all `applyRequires` artifacts are complete**
      - After creating each artifact, re-run `openspec status --change "<name>" --json`
      - Check if every artifact ID in `applyRequires` has `status: "done"` in the artifacts array
      - Stop when all `applyRequires` artifacts are done

   c. **If an artifact requires user input** (unclear context):
      - Use **AskUserQuestion tool** to clarify
      - Then continue with creation

7. **Show final status**
   ```bash
   openspec status --change "<name>"
   ```

**Output**

After completing all artifacts, summarize:
- Change name and location
- List of artifacts created with brief descriptions
- What's ready: "All artifacts created! Ready for implementation."
- Prompt: "Run `/opsx:apply` or ask me to implement to start working on the tasks."

**Artifact Creation Guidelines**

- Follow the `instruction` field from `openspec instructions` for each artifact type
- The schema defines what each artifact should contain - follow it
- Read dependency artifacts for context before creating new ones
- Use `template` as the structure for your output file - fill in its sections
- **IMPORTANT**: `context` and `rules` are constraints for YOU, not content for the file
  - Do NOT copy `<context>`, `<rules>`, `<project_context>` blocks into the artifact
  - These guide what you write, but should never appear in the output

**Guardrails**
- Create ALL artifacts needed for implementation (as defined by schema's `apply.requires`)
- Always read dependency artifacts before creating a new one
- If context is critically unclear, ask the user - but prefer making reasonable decisions to keep momentum
- If a change with that name already exists, ask if user wants to continue it or create a new one
- Verify each artifact file exists after writing before proceeding to next
