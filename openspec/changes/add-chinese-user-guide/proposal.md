## Why

`tread` currently offers user-facing documentation only in English, creating unnecessary friction for Chinese-speaking users despite the reader supporting CJK content. A concise Chinese guide makes installation and everyday operation discoverable without translating maintainer-only release procedures.

## What Changes

- Add `README-zh.md` as a Simplified Chinese user guide and cross-link it with the English `README.md`.
- Translate the English README's user-facing Features, Installation, Usage, and License content while preserving commands, option names, platform target identifiers, and destination links.
- Clearly exclude the maintainer-only `Release process` section from `README-zh.md`.
- Establish that future changes to user-facing content in `README.md` must be reflected in `README-zh.md`; changes limited to `Release process` do not require Chinese-guide updates.

## Capabilities

### New Capabilities
- `chinese-user-guide`: A maintained Simplified Chinese guide for installing and using `tread`, with a defined translation boundary and synchronization rule.

### Modified Capabilities
- None.

## Impact

- Affected documentation: `README.md` and new `README-zh.md`.
- No application code, CLI behavior, public APIs, dependencies, packaging, or release automation changes.
