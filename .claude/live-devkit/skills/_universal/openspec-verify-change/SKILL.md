---
name: openspec-verify-change
description: Verify implementation matches change artifacts. Use when the user wants to validate that implementation is complete, correct, and coherent before archiving.
license: MIT
compatibility: Requires openspec CLI.
metadata:
  author: openspec
  version: "1.0"
  generatedBy: "1.2.0"
---

Verify that an implementation matches the change artifacts (specs, tasks, design).

**Input**: Optionally specify a change name. If omitted, check if it can be inferred from conversation context. If vague or ambiguous you MUST prompt for available changes.

**Steps**

1. **If no change name provided, prompt for selection**

   Run `openspec list --json` to get available changes. Use the **AskUserQuestion tool** to let the user select.

   Show changes that have implementation tasks (tasks artifact exists).
   Include the schema used for each change if available.
   Mark changes with incomplete tasks as "(In Progress)".

   **IMPORTANT**: Do NOT guess or auto-select a change. Always let the user choose.

2. **Check status to understand the schema**
   ```bash
   openspec status --change "<name>" --json
   ```
   Parse the JSON to understand:
   - `schemaName`: The workflow being used (e.g., "spec-driven")
   - Which artifacts exist for this change

3. **Get the change directory and load artifacts**

   ```bash
   openspec instructions apply --change "<name>" --json
   ```

   This returns the change directory and context files. Read all available artifacts from `contextFiles`.

4. **Initialize verification report structure**

   Create a report structure with five dimensions:
   - **Completeness**: Track tasks and spec coverage
   - **Correctness**: Track requirement implementation and scenario coverage
   - **Coherence**: Track design adherence and pattern consistency
   - **UI Fidelity**: Track design-spec token matching (only when design-spec.md exists)
   - **Interaction Fidelity**: Track interaction pattern matching (only when interaction-demo.html exists)

   Each dimension can have CRITICAL, WARNING, or SUGGESTION issues.

5. **Verify Completeness**

   **Task Completion**:
   - If tasks.md exists in contextFiles, read it
   - Parse checkboxes: `- [ ]` (incomplete) vs `- [x]` (complete)
   - Count complete vs total tasks
   - If incomplete tasks exist:
     - Add CRITICAL issue for each incomplete task
     - Recommendation: "Complete task: <description>" or "Mark as done if already implemented"

   **Spec Coverage**:
   - If delta specs exist in `openspec/changes/<name>/specs/`:
     - Extract all requirements (marked with "### Requirement:")
     - For each requirement:
       - Search codebase for keywords related to the requirement
       - Assess if implementation likely exists
     - If requirements appear unimplemented:
       - Add CRITICAL issue: "Requirement not found: <requirement name>"
       - Recommendation: "Implement requirement X: <description>"

6. **Verify Correctness**

   **Requirement Implementation Mapping**:
   - For each requirement from delta specs:
     - Search codebase for implementation evidence
     - If found, note file paths and line ranges
     - Assess if implementation matches requirement intent
     - If divergence detected:
       - Add WARNING: "Implementation may diverge from spec: <details>"
       - Recommendation: "Review <file>:<lines> against requirement X"

   **Scenario Coverage**:
   - For each scenario in delta specs (marked with "#### Scenario:"):
     - Check if conditions are handled in code
     - Check if tests exist covering the scenario
     - If scenario appears uncovered:
       - Add WARNING: "Scenario not covered: <scenario name>"
       - Recommendation: "Add test or implementation for scenario: <description>"

7. **Verify Coherence**

   **Design Adherence**:
   - If design.md exists in contextFiles:
     - Extract key decisions (look for sections like "Decision:", "Approach:", "Architecture:")
     - Verify implementation follows those decisions
     - If contradiction detected:
       - Add WARNING: "Design decision not followed: <decision>"
       - Recommendation: "Update implementation or revise design.md to match reality"
   - If no design.md: Skip design adherence check, note "No design.md to verify against"

   **Code Pattern Consistency**:
   - Review new code for consistency with project patterns
   - Check file naming, directory structure, coding style
   - If significant deviations found:
     - Add SUGGESTION: "Code pattern deviation: <details>"
     - Recommendation: "Consider following project pattern: <example>"

7.5. **Verify UI Fidelity** (only when design-spec.md exists)

   If `design-spec.md` exists in the change directory:

   **Design Token Extraction**:
   - Read `design-spec.md` and extract:
     - All color values from the **Color Summary** section (hex codes and rgba values)
     - All font sizes from the **Font Summary** section (px values)
     - Key spacing/border-radius values from the **Spacing & Border Radius Summary** section
   - Build a list of expected design tokens with their usage context

   **CSS Token Matching**:
   - Search all CSS/SCSS/Vue files related to the change for each extracted token:
     - Colors: search for hex values (e.g., `#FF4747`), rgba values, and CSS variable references
     - Font sizes: search for the px values (e.g., `18px`, `font-size: 18px`)
     - Spacing/border-radius: search for the specified values (e.g., `gap: 24px`, `border-radius: 8px`)
   - For each token:
     - If found in CSS → mark as "matched"
     - If NOT found but likely referenced via CSS variable → Add WARNING: "Design token not found in CSS: <value> (used for: <usage>). May be referenced via CSS variable — verify manually."
     - If NOT found and no CSS variable equivalent → Add CRITICAL: "Design token missing from implementation: <value> (used for: <usage>)"

   **Fidelity Score**:
   - Count matched vs total tokens
   - Report as: "UI Fidelity: N/M tokens matched"

   If `design-spec.md` does NOT exist:
   - Skip this dimension
   - Note in report: "No design-spec.md to verify UI fidelity against"

7.6. **Verify Interaction Fidelity** (only when interaction-demo.html exists)

   If `interaction-demo.html` exists in the change directory:

   **Interaction Pattern Extraction**:
   - Read `interaction-demo.html` and extract:
     - All component states and their visual representation
     - State transition triggers and animations
     - Edge case handling patterns (debounce, double-click guard, etc.)
     - Keyboard navigation support
   - Build a list of expected interaction behaviors

   **Interaction Pattern Matching**:
   - For each interaction pattern, check the implementation:
     - States: Does the implementation cover all states from the demo? (loading / empty / error / data / hover / active / disabled)
     - Transitions: Are transition animations present with similar timing?
     - Edge cases: Are debounce guards, concurrent operation checks, etc. implemented?
     - Keyboard: Is Tab / Enter / Escape support present?
   - For each pattern:
     - If matched → mark as "matched"
     - If partially implemented → Add WARNING: "Interaction pattern partially implemented: <pattern>. Demo shows <expected>, implementation has <actual>."
     - If missing → Add CRITICAL: "Interaction pattern missing: <pattern>. Demo demonstrates this behavior but it's not found in implementation."

   **Fidelity Score**:
   - Count matched vs total interaction patterns
   - Report as: "Interaction Fidelity: N/M patterns matched"

   If `interaction-demo.html` does NOT exist:
   - Skip this dimension
   - Note in report: "No interaction-demo.html to verify interaction fidelity against"

8. **Generate Verification Report**

   **Summary Scorecard**:
   ```
   ## Verification Report: <change-name>

   ### Summary
   | Dimension    | Status           |
   |--------------|------------------|
   | Completeness | X/Y tasks, N reqs|
   | Correctness  | M/N reqs covered |
   | Coherence    | Followed/Issues  |
   | UI Fidelity  | N/M tokens matched (or "N/A - no design-spec") |
   | Interaction Fidelity | N/M patterns matched (or "N/A - no interaction-demo") |
   ```

   **Issues by Priority**:

   1. **CRITICAL** (Must fix before archive):
      - Incomplete tasks
      - Missing requirement implementations
      - Each with specific, actionable recommendation

   2. **WARNING** (Should fix):
      - Spec/design divergences
      - Missing scenario coverage
      - Each with specific recommendation

   3. **SUGGESTION** (Nice to fix):
      - Pattern inconsistencies
      - Minor improvements
      - Each with specific recommendation

   **Final Assessment**:
   - If CRITICAL issues: "X critical issue(s) found. Fix before archiving."
   - If only warnings: "No critical issues. Y warning(s) to consider. Ready for archive (with noted improvements)."
   - If all clear: "All checks passed. Ready for archive."

**Verification Heuristics**

- **Completeness**: Focus on objective checklist items (checkboxes, requirements list)
- **Correctness**: Use keyword search, file path analysis, reasonable inference - don't require perfect certainty
- **Coherence**: Look for glaring inconsistencies, don't nitpick style
- **False Positives**: When uncertain, prefer SUGGESTION over WARNING, WARNING over CRITICAL
- **Actionability**: Every issue must have a specific recommendation with file/line references where applicable

**Graceful Degradation**

- If only tasks.md exists: verify task completion only, skip spec/design/UI fidelity checks
- If tasks + specs exist: verify completeness and correctness, skip design and UI fidelity
- If full artifacts + design-spec.md + interaction-demo.html: verify all five dimensions
- If full artifacts + design-spec.md but no interaction-demo.html: verify four dimensions, note "No interaction-demo.html — Interaction Fidelity skipped"
- If full artifacts + interaction-demo.html but no design-spec.md: verify four dimensions, note "No design-spec.md — UI Fidelity skipped"
- If full artifacts but no design-spec.md and no interaction-demo.html: verify three dimensions, note both skipped
- Always note which checks were skipped and why

**Output Format**

Use clear markdown with:
- Table for summary scorecard
- Grouped lists for issues (CRITICAL/WARNING/SUGGESTION)
- Code references in format: `file.ts:123`
- Specific, actionable recommendations
- No vague suggestions like "consider reviewing"
