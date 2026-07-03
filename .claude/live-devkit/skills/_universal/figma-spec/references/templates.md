# Figma Spec Templates

## Output Document Template

```markdown
## Figma Design File Annotation

**Design file source**: <filename>
**Design file link**: <figma-link>
**Node ID**: <nodeId>
**Canvas size**: <width>x<height>

---

## Layout Structure

```
[ASCII layout diagram - generated from actual node tree]
```

### Key Element Analysis

<Analyze each top-level Frame and important child elements>

| Element name | Type | Position (x,y) | Size (w x h) | Layout properties | Notes |
|---------|------|-----------|-----------|---------|------|
| Frame1 | FRAME | x, y | w x h | row/center | Container description |
| Text1 | TEXT | x, y | w x h | - | Text content |

---

## Color Summary

| Usage | Color value | Figma variable name |
|------|------|-------------|
| Main background | #FFFFFF | fill_white |
| Primary text color | #333333 | titleBlack |
| Accent color | #0066FF | primary |
| ... | ... | ... |

---

## Font Summary

| Usage | Font family | Weight | Size | Line height | Letter spacing | Color | Alignment |
|------|--------|------|------|------|--------|------|------|
| Button text | PingFang SC | 400 | 12px | 1.33 | 0 | #FFFFFF | center |
| Title | PingFang SC | 600 | 18px | 1.11 | 0 | #333333 | left |
| ... | ... | ... | ... | ... | ... | ... | ... |

---

## Spacing and Border Radius

| Element | Type | Value |
|------|------|-----|
| Card border radius | borderRadius | 8px |
| Button spacing | gap | 24px |
| Container padding | padding | 16px |
| ... | ... | ... |

---

## Effects Summary

| Element | Effect type | Value |
|------|---------|-----|
| Card shadow | boxShadow | 2px 4px 12px 0px rgba(0,0,0,0.08) |
| Navigation bar blur | backdropFilter | blur(50px) |
| ... | ... | ... |

---

## Component References

<If the design file uses components, list component information>

| Component name | Component ID | Usage location |
|--------|--------|---------|
| Button/Default | 12345:67890 | Frame1 > Button1 |
| Icon/Close | 12345:67891 | Frame1 > CloseIcon |

---

## Responsive/Adaptation Information

<If there is Auto Layout or constraint information>

| Element | Layout mode | Adaptation behavior |
|------|---------|---------|
| Main container | Auto Layout row | Fixed width, adaptive height |
| Button | Auto Layout column | Fixed size |

```

---

## ASCII Layout Diagram Generation Rules

```
For FRAME type nodes:
┌─────────────────────────────────────┐
│  Frame Name (w x h)                │
│  ┌────────┐ ┌────────             │
│  │ Child1 │ │ Child2 │ ...        │
│  └────────┘ └────────┘            │
└─────────────────────────────────────┘

For TEXT type nodes:
[Text: "content"] <fontSize>px

For IMAGE/RECTANGLE type:
[Image/Color: #xxxxxx]
```

---

## Variable Replacement Rules

| Placeholder | Replacement source |
|--------|---------|
| `<filename>` | metadata.name |
| `<figma-link>` | Original link entered by user |
| `<nodeId>` | nodes[0].id |
| `<width>` | nodes[0].dimensions.width |
| `<height>` | nodes[0].dimensions.height |
| `<layout diagram>` | Recursively generated from nodes tree |

---

## Color Format Conversion

```javascript
// Figma RGB → CSS
{ r: 0.5, g: 0.5, b: 0.5, a: 1 } → rgb(128, 128, 128) or #808080

// Figma Gradient → CSS
{
  gradient: [
    { position: 0, color: { r: 0, g: 0, b: 0, a: 0 } },
    { position: 1, color: { r: 0.1, g: 0.1, b: 0.1, a: 1 } }
  ]
} → linear-gradient(180deg, rgba(0,0,0,0) 0%, rgba(26,26,26,1) 100%)
```

---

## Notes

1. **Color deduplication**: Merge identical color values, distinguish by usage
2. **Font deduplication**: Merge identical font combinations
3. **Hierarchy simplification**: ASCII diagrams show at most 3 levels of nesting
4. **Unit consistency**: Use px as the unit throughout
5. **Naming**: Tables and headings use English
