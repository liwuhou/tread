## Why

The dashboard currently writes every visual block sequentially without budgeting for terminal height, so short terminals can hide the most important recent-reading content behind headers, help text, and the open prompt. We need constrained-height behavior now that the dashboard is the default no-argument entry point, because a reader home screen that loses the reading list at small sizes breaks the primary resume flow.

## What Changes

- Change dashboard rendering to treat terminal height as a first-class layout constraint instead of blindly streaming all lines.
- Preserve a minimum three-line recent-reading area whenever the terminal is tall enough to show it.
- Prefer hiding or truncating footer help, prompt suggestions, and other auxiliary dashboard text before hiding recent-reading content.
- Allow the dashboard open prompt to become partially or fully obscured under constrained height rather than forcing it to displace recent-reading content.
- Add an explicit minimal fallback for extremely short terminals that cannot fit the compact recent-reading layout.
- No breaking CLI behavior changes beyond the already-shipped dashboard entry behavior.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `reading-dashboard`: Dashboard rendering and prompt visibility rules change under constrained terminal heights so recent-reading content remains visible.

## Impact

- Affected code:
  - `src/main.rs`: dashboard line construction and render flow need height-aware budgeting and compact fallback behavior.
  - `src/dashboard.rs`: may need compact-row helpers or explicit recent-reading slice selection for constrained layouts.
  - `README.md`: dashboard behavior docs may need a short note if footer/prompt visibility rules are user-visible enough to document.
- Tests will need explicit small-height coverage for dashboard rendering decisions.
- No new storage, dependencies, or external integrations are required.
