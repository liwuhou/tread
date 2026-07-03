---
name: interaction-demo
description: Generate a low-fidelity interactive HTML demo from verbal descriptions to validate interaction requirements. Outputs a grayscale wireframe-style self-contained HTML file, focusing on interaction behavior rather than visual design.
license: MIT
compatibility: Requires openspec CLI.
metadata:
  author: openspec
  version: "1.0"
  generatedBy: "1.2.0"
---

Generate a low-fidelity interactive HTML demo from verbal descriptions. This demo only cares about interaction behavior, not visual design.

**Input**: User's verbal descriptions (functional requirements, interaction flows, page states, etc.), optionally specifying a change name.

## Design Constraints (Strictly Enforced)

### Visual Layer: Deliberately Restrained

```
Colors:  Grayscale primarily (#111, #333, #666, #999, #ddd, #f5f5f5)
         Single accent color (default #3F6BFF, can be used for buttons, links, and other interactive elements)
         No gradients, no multiple colors

Fonts:   system-ui, -apple-system, sans-serif (do not import any web fonts)
         Only use font weights 400 / 600

Effects: No shadows, no blur, no gradients
         Border radius only 4px / 8px
         Borders 1px solid #ddd

Icons:   No emoji, use plain text or simple CSS shapes instead
```

### Interaction Layer: Fully Expressed

```
States that must be covered:
  [ ] Initial state   — Appearance when first opened
  [ ] Loading state   — Data loading (skeleton / spinner)
  [ ] Empty state     — Prompt when no data
  [ ] Error state     — Error prompt + retry entry
  [ ] Ideal state     — Normal display when data is present
  [ ] Hover           — Mouse hover feedback
  [ ] Active          — Click/press feedback
  [ ] Disabled        — Disabled state
  [ ] Transitions     — Transitions during state changes (expand/collapse, fade, slide, etc.)

Edge cases that must be handled:
  [ ] Rapid double-click — Debounce
  [ ] Long lists         — Scroll behavior, virtual list demo
  [ ] Keyboard           — Tab navigation, Enter/Escape
```

### Code Layer: Zero Dependencies

```
Single HTML file with inline CSS + JS
Do not use any frameworks (Vue/React/jQuery)
Do not use external CDN resources
Open in browser and run immediately
```

## Output Location

```
openspec/changes/<change-name>/interaction-demo.html
```

If the change has not been created yet, save to a temporary location first and move it in after the change is created.

## Generation Process

### Step 1: Understand Interaction Requirements

Extract from user descriptions:

1. **Core flow**: What is the user's primary operation path?
2. **Pages/Components**: Which UI modules are involved?
3. **State list**: What states does each component have?
4. **Interaction details**: Click, expand, drag, input, toggle, etc.
5. **Edge cases**: Empty data, errors, extreme values, concurrent operations

If any interaction details are unclear, confirm with **AskUserQuestion** — do not guess.

### Step 2: Draw the State Machine

Before generating code, draw the core interaction state machine using an ASCII diagram:

```
         ┌─────────┐
         │ loading │
         └────┬────┘
              │
    ┌─────────┼─────────┐
    ▼         ▼         ▼
┌──────┐ ┌──────┐ ┌──────┐
│ empty│ │ data │ │error │
└──┬───┘ └──┬───┘ └──┬───┘
   │        │        │
   │   ┌────▼───┐    │
   │   │expanded│    │
   │   └────────┘    │
   └─────────────────┘
         │
         ▼
      retry → loading
```

Show it to the user for confirmation.

### Step 3: Generate HTML Demo

Generate a single HTML file with the following structure:

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>[Feature Name] - Interaction Demo</title>
<style>
  /* === Base Reset === */
  /* === Layout === */
  /* === Component Styles (Grayscale) === */
  /* === State Styles === */
  /* === Transition Animations === */

  /*
   * Design Notes:
   * - This demo is a low-fidelity wireframe, only for validating interaction behavior
   * - Colors, fonts, and spacing do not reflect the final visual effect
   * - For final visuals, refer to the Figma design file (if available)
   */
</style>
</head>
<body>
  <!-- State switch controls (for development) -->
  <div class="demo-controls">
    <button onclick="setState('loading')">Loading</button>
    <button onclick="setState('empty')">Empty</button>
    <button onclick="setState('error')">Error</button>
    <button onclick="setState('data')">Data</button>
  </div>

  <!-- Main content area -->
  <main id="app"></main>

<script>
  /*
   * State Management
   *
   * All interaction logic is centralized here.
   * Avoid scattered event handlers; keep the data flow clear.
   */

  const state = {
    current: 'loading',
    data: [],
    error: null
  };

  function setState(s) { /* ... */ }
  function render() { /* ... */ }

  // Initialize
  render();
</script>
</body>
</html>
```

### Step 4: Self-Check

After generation, confirm each of the following:

- [ ] Can be opened directly in a browser
- [ ] All state switch buttons work properly
- [ ] Interactions have visual feedback (hover/active/transition)
- [ ] No emoji used
- [ ] No web fonts used
- [ ] Colors do not exceed: black/white/gray + one accent color
- [ ] demo-controls are convenient for demonstration: one-click switch for each state

## Common Scenario Templates

### List + Detail

```
States: loading → empty / data / error
Interactions: Click list item → expand detail (accordion or side panel)
     SlideDown transition on expand
     Only one can be expanded at a time
Edge cases: Empty list, load failure, rapid clicks
```

### Form + Submit

```
States: idle → filling → validating → submitting → success / error
Interactions: Real-time validation on input, full validation on blur
     Prevent duplicate submission on submit
     Error messages appear below the field on error
Edge cases: Network disconnect, timeout, duplicate submission
```

### Modal / Dialog

```
States: hidden → opening → visible → closing → hidden
Interactions: Click overlay to close, ESC to close
     Fade + scale transition on opening/closing
     Button actions within the modal
Edge cases: Scrolling inside modal, mobile adaptation, multiple modal stacking
```

## Relationship with Figma design-spec

```
interaction-demo.html          design-spec.md (Figma)
─────────────────────          ─────────────────────
Interaction behavior  ✅ Authoritative     ❌ Static
Visual design         ❌ Deliberate grayscale ✅ Authoritative
State coverage        ✅ Complete           ❌ Partial only
Transition animations ✅ Complete           ❌ Not visible
Colors/Fonts          ❌ Not involved       ✅ Precise values
```

The two are complementary, not overlapping. Use each as needed when writing specs.

## Guardrails

- **Do not do visual design** — Grayscale wireframes, not final UI
- **Do not omit states** — Loading/empty/error are core value, must be implemented
- **Do not use frameworks** — Pure vanilla HTML, zero dependencies
- **Do not import external resources** — No CDN, web fonts, or icon libraries
- **Stop and ask when interaction is ambiguous** — Do not guess what interaction the user wants
- **Draw the state machine before writing code** — Let the user confirm the interaction flow before generating
