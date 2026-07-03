---
name: openspec-apply-change
description: Implement tasks from an OpenSpec change. Use when the user wants to start implementing, continue implementation, or work through tasks.
license: MIT
compatibility: Requires openspec CLI.
metadata:
  author: openspec
  version: "1.0"
  generatedBy: "1.2.0"
---

Implement tasks from an OpenSpec change.

**Input**: Optionally specify a change name. If omitted, check if it can be inferred from conversation context. If vague or ambiguous you MUST prompt for available changes.

**Steps**

1. **Select the change**

   If a name is provided, use it. Otherwise:
   - Infer from conversation context if the user mentioned a change
   - Auto-select if only one active change exists
   - If ambiguous, run `openspec list --json` to get available changes and use the **AskUserQuestion tool** to let the user select

   Always announce: "Using change: <name>" and how to override (e.g., `/opsx:apply <other>`).

2. **Check status to understand the schema**
   ```bash
   openspec status --change "<name>" --json
   ```
   Parse the JSON to understand:
   - `schemaName`: The workflow being used (e.g., "spec-driven")
   - Which artifact contains the tasks (typically "tasks" for spec-driven, check status for others)

3. **Get apply instructions**

   ```bash
   openspec instructions apply --change "<name>" --json
   ```

   This returns:
   - Context file paths (varies by schema - could be proposal/specs/design/tasks or spec/tests/implementation/docs)
   - Progress (total, complete, remaining)
   - Task list with status
   - Dynamic instruction based on current state

   **Handle states:**
   - If `state: "blocked"` (missing artifacts): show message, suggest using openspec-continue-change
   - If `state: "all_done"`: congratulate, suggest archive
   - Otherwise: proceed to implementation

4. **Read context files**

   Read the files listed in `contextFiles` from the apply instructions output.
   The files depend on the schema being used:
   - **spec-driven**: proposal, specs, design, tasks
   - Other schemas: follow the contextFiles from CLI output

   **Design-spec bridge (MANDATORY for UI changes)**:
   After reading context files, check if `design-spec.md` exists in the change directory:
   ```bash
   ls openspec/changes/<name>/design-spec.md 2>/dev/null
   ```
   If `design-spec.md` exists, you MUST:
   1. Read `design-spec.md` and extract the design token reference (colors, fonts, spacing, layout)
   2. Build a **Design Fidelity Reference Table** listing all design tokens that UI tasks must satisfy
   3. Keep this reference table in context throughout the implementation loop
   4. For every UI component task completed, output a **Design Fidelity Checklist** (see step 6)

   **Interaction-demo bridge (MANDATORY for UI changes)**:
   Also check if `interaction-demo.html` exists in the change directory:
   ```bash
   ls openspec/changes/<name>/interaction-demo.html 2>/dev/null
   ```
   If `interaction-demo.html` exists, you MUST:
   1. Read `interaction-demo.html` and extract the interaction reference:
      - All component states and their triggers
      - State transition animations and timing
      - Edge case handling (debounce, concurrent ops, etc.)
      - Keyboard navigation patterns
   2. Build an **Interaction Reference Table** listing all interaction behaviors that UI tasks must satisfy
   3. Keep this reference table in context throughout the implementation loop
   4. For every UI component task completed, output an **Interaction Fidelity Checklist** (see step 6)

5. **Show current progress**

   Display:
   - Schema being used
   - Progress: "N/M tasks complete"
   - Remaining tasks overview
   - Dynamic instruction from CLI

6. **Implement tasks (loop until done or blocked)**

   First, analyze all pending tasks to determine the best implementation approach:
   - If **2+ independent subtasks** exist → Use built-in parallel execution flow (see below)
   - If task is **feature implementation or bug fix** → Follow built-in TDD flow (see below)
   - If task is **simple change** (config, docs, single-line fixes) → Implement directly

   ### Built-in TDD Flow
   For feature implementation or bug fix tasks:
   1. **Understand the requirement** from task description and context files
   2. **Find or create the test file** for the component/function being modified
   3. **Write a failing test** that describes the expected behavior
   4. **Run the test** to confirm it fails as expected
   5. **Implement the minimal code** needed to make the test pass
   6. **Refactor** the code for quality while keeping tests passing
   7. **Run all related tests** to ensure no regressions
   8. **Mark the task as complete**

   ### Built-in Parallel Task Execution
   For multiple independent subtasks (no overlapping file modifications):
   1. **Group tasks** by independence (tasks that modify different files/domains can run in parallel)
   2. **Process independent groups sequentially**, with each group's tasks executed in parallel via Agent tool
   3. For each parallel group, create subagents using:
      ```
      Agent({
        subagent_type: "general-purpose",
        description: "<task description>",
        prompt: "<full task context + relevant file contents + project spec references>"
      })
      ```
   4. **Wait for all subagents to complete** their work
   5. **Review and merge** all changes, resolve any conflicts
   6. **Mark all tasks in the group as complete**

   ### Direct Implementation
   For simple changes (configuration updates, documentation edits, single-line bug fixes):
   1. Make the minimal required change
   2. Verify correctness
   3. Mark the task as complete

   For each pending task:
   - Show which task is being worked on
   - Follow the appropriate flow (TDD/parallel/direct) based on task type
   - Keep changes minimal and focused
   - Mark task complete in the tasks file: `- [ ]` → `- [x]`

   **Design Fidelity Checklist (MANDATORY when design-spec.md exists)**:
   After completing a UI component task, output a checklist comparing the implementation against design-spec.md:
   ```
   ## Design Fidelity Checklist: <component-name>
   | Token Type | Design-Spec Value | Implementation | Status |
   |-----------|-------------------|----------------|--------|
   | Color     | #FF4747 (leave button) | ✓ matched     | ✅     |
   | Font      | PingFang SC 600/18px (countdown) | ✓ matched | ✅ |
   | Spacing   | gap: 24px (button spacing) | ✗ used 20px | ⚠️     |
   | Radius    | 8px (card border-radius) | ✓ matched     | ✅     |
   ```
   For any ⚠️ items, add a brief reason (e.g., "adapted for CSS variable usage", "adjusted for responsive layout").
   Non-UI tasks (state logic, API integration, etc.) do NOT require a checklist.

   **Interaction Fidelity Checklist (MANDATORY when interaction-demo.html exists)**:
   After completing a UI component task, output a checklist comparing the interaction behavior against interaction-demo.html:
   ```
   ## Interaction Fidelity Checklist: <component-name>
   | Behavior | Demo Reference | Implementation | Status |
   |----------|---------------|----------------|--------|
   | Loading state | Skeleton placeholder, 3 cards pulsing | ✓ matched | ✅ |
   | Empty state | "No data" + action hint | ✗ missing hint text | ⚠️ |
   | Error state | Error msg + retry button | ✓ matched | ✅ |
   | Expand transition | slideDown 250ms ease-out | ✓ matched | ✅ |
   | Double-click guard | Debounce 300ms on submit | ✗ not implemented | ⚠️ |
   | Keyboard: Tab | Navigate list items | ✓ matched | ✅ |
   | Keyboard: Escape | Close expanded item | ✗ not handled | ⚠️ |
   ```
   For any ⚠️ items, add a brief reason (e.g., "deferred to next iteration", "framework handles this differently").
   Non-UI tasks do NOT require this checklist.

   - Continue to next task

   **Pause if:**
   - Task is unclear → ask for clarification
   - Implementation reveals a design issue → suggest updating artifacts
   - Error or blocker encountered → report and wait for guidance
   - User interrupts

7. **On completion or pause, show status**

   Display:
   - Tasks completed this session
   - Overall progress: "N/M tasks complete"
   - If all done: suggest archive
   - If paused: explain why and wait for guidance

**Output During Implementation**

```
## Implementing: <change-name> (schema: <schema-name>)

Working on task 3/7: <task description>
[...implementation happening...]
✓ Task complete

Working on task 4/7: <task description>
[...implementation happening...]
✓ Task complete
```

**Output On Completion**

```
## Implementation Complete

**Change:** <change-name>
**Schema:** <schema-name>
**Progress:** 7/7 tasks complete ✓

### Completed This Session
- [x] Task 1
- [x] Task 2
...

All tasks complete! Ready to archive this change.
```

**Output On Pause (Issue Encountered)**

```
## Implementation Paused

**Change:** <change-name>
**Schema:** <schema-name>
**Progress:** 4/7 tasks complete

### Issue Encountered
<description of the issue>

**Options:**
1. <option 1>
2. <option 2>
3. Other approach

What would you like to do?
```

**Guardrails**
- Keep going through tasks until done or blocked
- Always read context files before starting (from the apply instructions output)
- If task is ambiguous, pause and ask before implementing
- If implementation reveals issues, pause and suggest artifact updates
- Keep code changes minimal and scoped to each task
- Update task checkbox immediately after completing each task
- Pause on errors, blockers, or unclear requirements - don't guess
- Use contextFiles from CLI output, don't assume specific file names
- **TDD Compliance**: For feature/bug tasks, always write tests first before implementation, and maintain 80%+ test coverage for core modules
- **Parallel Execution**: When running tasks in parallel, ensure tasks modify separate files/domains to avoid merge conflicts
- **Agent Isolation**: Parallel tasks run in isolated subagents, no shared state between them

**Fluid Workflow Integration**

This skill supports the "actions on a change" model:

- **Can be invoked anytime**: Before all artifacts are done (if tasks exist), after partial implementation, interleaved with other actions
- **Allows artifact updates**: If implementation reveals design issues, suggest updating artifacts - not phase-locked, work fluidly
