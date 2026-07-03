# Figma Design File Reproduction Workflow

## Overview

This workflow is used in the OpenSpec development process to automatically capture and parse Figma design files, generate detailed UI specification documents, and archive them in the change directory as part of the project knowledge base.

---

## Complete Workflow

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Figma Reproduction Workflow                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  1️⃣  /opsx:explore                                                  │
│      ├─ Explore requirements, clarify ambiguities                   │
│      ├─ AI proactively asks: "Do you have a Figma design file?"     │
│      ├─ User provides a link                                        │
│      └─ Save to .claude/figma-links.json                            │
│                                                                     │
│  2️⃣  /opsx:propose <change-name>                                    │
│      ├─ Check .claude/figma-links.json                              │
│      ├─ If pending links exist → auto-invoke /figma-spec (skip asking)│
│      ├─ If no pending links → ask user if they have a Figma design file│
│      ├─ Create change directory                                     │
│      ├─ Generate proposal.md, design.md, specs, tasks.md            │
│      └─ Auto-invoke /figma-spec                                     │
│                                                                     │
│  3️⃣  /figma-spec <link> --change <change-name>                      │
│      ├─ Call Figma MCP to fetch design file data                    │
│      ├─ Parse layout, colors, fonts, spacing                        │
│      ├─ Generate design-spec.md into the change directory            │
│      ├─ Update design.md to add reference                           │
│      └─ Clean up processed links in .claude/figma-links.json        │
│                                                                     │
│  4️⃣  /opsx:continue (spec generation)                               │
│      ├─ Detect whether design-spec.md exists                        │
│      ├─ If exists + UI capability → inject Design Tokens into spec  │
│      └─ Design Tokens include colors/fonts/spacing/layout constraints│
│                                                                     │
│  5️⃣  /opsx:apply                                                    │
│      ├─ Read design-spec.md to build design reproduction checklist   │
│      ├─ Pixel-perfect UI reproduction                               │
│      ├─ Output Design Fidelity Checklist after each UI component     │
│      └─ Implement all tasks                                         │
│                                                                     │
│  6️⃣  /opsx:verify                                                   │
│      ├─ If design-spec.md exists → add UI Fidelity dimension        │
│      ├─ Extract color/font-size/spacing values from design-spec     │
│      ├─ Search for matches in CSS/SCSS/Vue files                    │
│      ├─ Not found → WARNING (may be referenced via CSS variable)    │
│      └─ Complete mismatch → CRITICAL                                │
│                                                                     │
│  7️⃣  /opsx:archive                                                   │
│      ├─ Archive change directory                                     │
│      └─ design-spec.md archived together, persisted as knowledge base│
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## File Structure

```
.claude/
├── figma-links.json          # Figma link staging area
│   {
│     "pending": [
│       {
│         "url": "https://figma.com/...?node-id=100-200",
│         "description": "1v1 live not-started state",
│         "capturedAt": "2026-04-15T10:00:00Z"
│       }
│     ]
│   }
│
└── skills/
    ├── figma-spec/           # Figma parsing skill
    ├── openspec-explore/     # Capture links during explore phase
    └── openspec-propose/     # Auto-invoke during propose phase

openspec/changes/<change-name>/
├── .openspec.yaml
├── proposal.md
├── design.md                 # References design-spec.md
├── design-spec.md            # Figma detailed annotations (visual truth)
├── interaction-demo.html     # Low-fidelity interaction demo (interaction truth, optional)
├── specs/
│   ├── capability-1/spec.md
│   └── capability-2/spec.md
└── tasks.md
```

---

## Usage Guide

### Scenario 1: Complete Workflow (Recommended)

```bash
# 1. Explore phase
/opsx:explore

# Provide Figma link during conversation
AI: "Do you have a Figma design file?"
You: Yes, https://www.figma.com/design/xxx/node-id=100-200

# 2. Create proposal
/opsx:propose onevone-live-scene

# Automatically invokes figma-spec to generate design-spec.md

# 3. Start implementation
/opsx:apply
```

### Scenario 2: Manual Invocation

```bash
# Change directory already exists, supplement Figma annotations
/figma-spec https://figma.com/... --change onevone-live-scene
```

### Scenario 3: Design Without Figma

```bash
# Follow the normal process directly, no Figma steps needed
/opsx:explore
/opsx:propose <change-name>
/opsx:apply
```

---

## Design File Output Content

`design-spec.md` contains:

1. **Layout Structure** - ASCII layout diagram + element position and dimensions
2. **Color Summary** - All used colors and their purposes
3. **Font Summary** - Font family, weight, size, line height
4. **Spacing & Border Radius** - padding, margin, borderRadius
5. **Effects Summary** - Shadows, blur, gradients

Example:

```markdown
## Figma Design File Annotations

**Source**: [Admin Console 3.0] Industry_v20260227
**Node**: 20063-17752
**Canvas Size**: 375x812

---

## Layout Structure

┌─────────────────────────────────────┐
│ 9:41  [Signal Bar]           [Close]│
├─────────────────────────────────────┤
│    [Live Preview Card]              │
│     Consultation Starting Soon      │
│   00 Days 02 Hours 02 Mins 08 Secs │
│                                     │
├─────────────────────────────────────┤
│ [Mic][Camera][Flip][Mirror][Beauty] │
└─────────────────────────────────────┘

---

## Color Summary

| Purpose          | Color Value              |
|------------------|--------------------------|
| Button text      | #FFFFFF                  |
| Countdown digits | #333333                  |
| Preview card bg  | rgba(255,255,255,0.7)    |

---

## Font Summary

| Purpose          | Font        | Weight | Size  |
|------------------|-------------|--------|-------|
| Button text      | PingFang SC | 400    | 12px  |
| Countdown digits | PingFang SC | 600    | 18px  |
```

---

## Design File Width Conversion Rules

**Core Principle: Source code is uniformly based on 375px baseline, postcss-pxtorem rootValue: 37.5**

The project rootValue is fixed at 37.5 (corresponding to a 375px design file). When the Figma design file width is not 375px, the agent must automatically convert px values:

| Design File Width | Conversion Ratio      | Example                                                          |
|-------------------|-----------------------|------------------------------------------------------------------|
| 375px             | 1:1 use directly      | Figma 16px → source 16px                                        |
| 750px             | ÷2                    | Figma 32px → source 16px                                        |
| Other widths      | ÷(width/375)          | 20px in a 414px Figma design → source 20÷(414/375)≈18.12px     |

**Conversion Process:**

1. When reading Figma data, check the root frame width (i.e., canvas/page width)
2. Calculate conversion ratio: `ratio = frameWidth / 375`
3. All px values ÷ ratio before writing to source code
4. 1px borders are not converted (preserve the 1px thin line effect)
5. Annotate the design file width and conversion ratio at the top of design-spec.md

**Example design-spec.md Header:**

```markdown
**Source**: [Project Name]
**Node**: 100-200
**Canvas Size**: 750x1624
**Conversion Ratio**: 750/375 = 2 (all px values below have been ÷2 to convert to 375px baseline)
```

---

## Best Practices

### Recommended Practices

1. **Provide Figma links during the explore phase** — Ensure complete design file information
2. **Provide independent node links for each core page** — Easier to parse separately
3. **Supplement design decisions in design.md** — Figma data + technical decisions
4. **Preserve design-spec.md when archiving** — Persist as knowledge base

### Practices to Avoid

1. **Do not provide only Figma Home links** — Must include a specific node-id
2. **Do not remember Figma after propose** — The workflow becomes fragmented
3. **Do not manually copy Figma annotations** — Use /figma-spec for automatic parsing
4. **Do not forget to clean up figma-links.json** — Avoid duplicate processing

---

## Troubleshooting

### Issue: figma-links.json Not Being Read

**Check**:
1. Is the file path correct: `.claude/figma-links.json`
2. Is the JSON format valid
3. Does `openspec-propose/skill.md` include the check logic

### Issue: /figma-spec Invocation Fails

**Check**:
1. Is the Figma MCP available
2. Does the link include a node-id parameter
3. Does the change directory exist

### Issue: design-spec.md Not Read by apply

**Check**:
1. Does design.md have the correct reference
2. Is the file path relative and correct
3. Does tasks.md need updating

---

## Future Extensions

1. **Support multiple design tools** — Instant Design, MasterGo, Pixso
2. **Auto-generate color variables** — Output SCSS/CSS variable files
3. **Component mapping** — Figma components → code component mapping table
4. **Responsive analysis** — Auto Layout → Tailwind class names

---

## Collaboration with Interaction Demo

Figma design-spec and interaction demo are complementary, not overlapping:

| Dimension        | Figma design-spec          | Interaction Demo (interaction-demo.html) |
|------------------|----------------------------|-----------------------------------------|
| Colors           | Authoritative source       | Not involved (grayscale)                |
| Fonts            | Authoritative source       | Not involved (system font)              |
| Spacing          | Authoritative source       | Approximate                             |
| Interaction states | Static                   | Authoritative source                    |
| Transition animations | Not visible           | Authoritative source                    |
| Edge cases       | Not visible                | Authoritative source                    |
| Keyboard operations | Not visible             | Authoritative source                    |

When both exist, the apply phase outputs both a Design Fidelity Checklist (visual) and an Interaction Fidelity Checklist (interaction), each governing its own dimension.

---

## Related Files

- `.claude/skills/figma-spec/skill.md` - Figma parsing skill
- `.claude/skills/openspec-explore/skill.md` - Capture during explore phase
- `.claude/skills/openspec-propose/skill.md` - Invoke during propose phase
- `openspec/changes/<name>/design-spec.md` - Output file
