## 1. Height-Aware Dashboard Layout

- [x] 1.1 Add a terminal-height budget to dashboard rendering instead of blindly printing every block
- [x] 1.2 Implement compact recent-reading rendering that preserves section label, current entry title, and target within 3 rows
- [x] 1.3 Add an explicit minimal fallback state for terminal heights below 3 rows

## 2. Auxiliary UI Degradation Rules

- [x] 2.1 Hide or truncate footer help before reducing the reserved recent-reading area
- [x] 2.2 Shrink or omit prompt suggestions based on remaining height budget
- [x] 2.3 Allow prompt and recoverable error text to be partially or fully obscured when needed to preserve recent-reading visibility

## 3. Verification and Documentation

- [x] 3.1 Add focused render tests for full, compact, and minimal-height dashboard states
- [x] 3.2 Add focused tests covering prompt-mode behavior under constrained height
- [x] 3.3 Update README or dashboard help text if the new constrained-height behavior needs user-visible documentation
