## Context

`tread`'s dashboard currently renders a fixed ordered stream of lines: header, section label, rows, footer help, prompt, suggestions, and errors. The renderer uses terminal width for truncation but ignores terminal height, so any short terminal simply clips whatever falls below the visible bottom edge. In practice this means the dashboard can lose the recent-reading list first, even though that list is the core reason the dashboard exists.

This change is intentionally narrow. We are not redesigning dashboard interactions, changing history semantics, or introducing a full ratatui layout tree. We only need predictable degradation rules for short terminals, with the user decision that the recent-reading area should retain at least three visible lines before auxiliary UI gets protected.

There is no Figma design file for this terminal UI change. Existing terminal styling stays in the current visual language: plain text rows, dim secondary text, and keyboard-first interactions.

## Goals / Non-Goals

**Goals:**
- Make dashboard rendering height-aware.
- Preserve at least three visible lines for the recent-reading area whenever terminal height is at least three rows.
- Prioritize recent-reading visibility over footer help, prompt suggestions, and other auxiliary text.
- Let the open prompt degrade under constrained height instead of displacing recent-reading content.
- Provide deterministic fallback behavior for extremely short terminals.

**Non-Goals:**
- Reworking dashboard navigation, storage, or history ordering.
- Guaranteeing full visibility of the open prompt or suggestions at every terminal size.
- Building a general-purpose layout engine for all TUI screens.
- Changing the reader views outside the dashboard.

## Decisions

### Treat terminal height as a render budget

Dashboard rendering should calculate terminal height up front and decide which lines fit, instead of emitting a full line stream and relying on terminal clipping.

Alternatives considered:
- Keep sequential rendering and only cap prompt/suggestion count. This still lets headers, blank lines, or footer help consume the rows needed for recent-reading content.
- Move the dashboard to a full ratatui widget tree. This would solve layout more generally but is much larger than the problem we are fixing.

Rationale: the bug is caused by missing height budgeting. Solving that directly is the smallest design that changes the UX outcome.

### Reserve a compact three-line recent-reading area

When the terminal height is constrained but at least three rows tall, the dashboard should reserve three rows for the recent-reading area before rendering any optional blocks.

The compact recent-reading area follows the user's preferred interpretation of "three lines":

```text
最近阅读
<current row title>
<current row target>
```

This is an area-level guarantee, not a per-entry expanded card. It preserves section context plus one currently relevant row.

Alternatives considered:
- Guarantee three lines for the selected entry itself, including a dedicated progress line. Current dashboard rows are two-line entries, so this would force a broader content-model change.
- Guarantee only two lines (title + target). This loses the section label and makes the compact state feel visually disconnected.

Rationale: this matches the requested UX while staying close to the existing row model.

### Degrade dashboard blocks by priority

Render blocks should degrade in this order:

1. Preserve recent-reading area.
2. Preserve essential section/header context only if it does not violate the three-line reserve.
3. Hide footer help.
4. Hide or shrink prompt suggestions.
5. Allow prompt and recoverable error lines to be partially or fully obscured under severe constraints.

A practical layout model:

```text
normal height:   header + list + footer + prompt + suggestions + error
small height:    header/list compact + maybe prompt line
very small:      compact three-line recent-reading area only
< 3 rows:        terminal-too-small fallback
```

Alternatives considered:
- Treat prompt visibility as higher priority than recent-reading content. This preserves the currently active mode but defeats the dashboard's core resume purpose.
- Always preserve the footer hint line. This consumes scarce rows with discoverability text instead of user data.

Rationale: auxiliary guidance is useful but recoverable; losing the reading list is the actual failure.

### Use explicit fallback states rather than accidental clipping

If the terminal height is below three rows, the dashboard should render a minimal fallback message rather than a partially chopped normal dashboard. The fallback can be as small as a single short line indicating that the terminal is too small.

Alternatives considered:
- Show whatever first N lines happen to fit. This is unstable because blank lines and decorative separators can become the only visible content.
- Panic or refuse to render. This is hostile for a transient resize state.

Rationale: deterministic fallback states make tests straightforward and avoid confusing half-rendered screens.

### Test rendering as pure line selection

Most of this behavior should be tested as pure rendering decisions: given dashboard state, width, and height, which lines are chosen and which blocks are omitted. Terminal cursor placement for the prompt can remain a smaller focused behavior test.

Alternatives considered:
- Rely on manual smoke testing only. This risks regressions on later dashboard tweaks.
- Test only the final printed buffer through end-to-end terminal integration. This is heavier than necessary for deterministic line-budget logic.

Rationale: the new behavior is mostly a selection problem, so unit-level render tests give the best signal.

## Risks / Trade-offs

- **Prompt mode may feel less visible on short terminals.** This is intentional; the trade-off favors preserving recent-reading context.
- **Compact mode shows less metadata.** Users may temporarily lose footer hints or suggestion lists until they resize taller.
- **Threshold choices can feel arbitrary.** Tests should lock the intended states so later changes stay consistent.
- **Height-aware rendering logic can sprawl inside `main.rs`.** Keep budgeting helpers small and data-driven to avoid another hard-coded render path maze.
