## Context

`tread` has a single English `README.md` that combines user documentation with a maintainer-only release procedure. The project needs a Chinese entry point for users, but duplicating the entire file would translate content that is irrelevant to the target audience and increase synchronization burden.

## Goals / Non-Goals

**Goals:**
- Provide `README-zh.md` as a Simplified Chinese guide for installing and using `tread`.
- Keep commands, flags, paths, platform target triples, binary names, and external URLs byte-for-byte compatible with the English source where applicable.
- Make English and Chinese user documentation mutually discoverable through language links.
- Define an explicit maintenance boundary for keeping user-facing content synchronized.

**Non-Goals:**
- Translating the maintainer-only `Release process` section.
- Changing application behavior, packaging metadata, release automation, or the CLI.
- Introducing localization infrastructure or translating documentation beyond the README user guide.

## Decisions

### Separate user-guide file rather than a full translated README

Create `README-zh.md` with only the translated user-facing sections: Features, Installation, Usage, and License. Do not include `Release process`.

**Rationale:** Chinese readers receive the complete usage path without inheriting an internal release-maintenance procedure. The smaller document also has a clear, reviewable synchronization surface.

**Alternative considered:** Translate the entire README. Rejected because it expands scope to maintainer content and doubles the release-process maintenance burden.

### Bidirectional language navigation

Add a Chinese-language link near the top of `README.md` and an English-language link near the top of `README-zh.md`.

**Rationale:** Repository visitors can select their language without needing to infer filenames or search the tree.

**Alternative considered:** Add only a link from the English README. Rejected because a reader landing directly on the Chinese guide would have no equally obvious path back to the canonical English source.

### User-facing content is the synchronization contract

When Features, Installation, Usage, or License content changes in `README.md`, update the equivalent Chinese section in the same change. Changes restricted to `Release process` do not require updates to `README-zh.md`.

**Rationale:** The rule ties upkeep to semantic sections rather than an imprecise expectation that two files remain identical.

## Risks / Trade-offs

- **Translation drift:** A change may update English user content without its Chinese counterpart. **Mitigation:** make the section-level synchronization rule explicit in the guide capability and review documentation changes against it.
- **Terminology inconsistency:** CLI concepts may be translated differently over time. **Mitigation:** preserve literal CLI tokens and use stable Chinese terms for reader, dashboard, history, and shortcuts.
- **README structure changes:** Reorganizing English sections can obscure correspondence. **Mitigation:** retain matching section order and headings where practical.
