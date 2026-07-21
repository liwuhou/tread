## ADDED Requirements

### Requirement: Simplified Chinese user guide
The repository SHALL provide `README-zh.md` as a Simplified Chinese user guide for `tread`. The guide SHALL contain the user-facing equivalents of the English README's Features, Installation, Usage, and License sections.

The guide SHALL preserve literal commands, CLI option names, paths, binary names, platform target identifiers, and external destination URLs from the corresponding English user-facing content.

#### Scenario: Chinese-speaking user opens the guide
- **WHEN** a repository visitor opens `README-zh.md`
- **THEN** they can learn `tread`'s features, install it, and use its commands and keyboard controls in Simplified Chinese

#### Scenario: Installation identifiers remain actionable
- **WHEN** the Chinese guide describes an installation command or release archive target
- **THEN** the command and target identifier match their English README counterparts exactly

### Requirement: Bidirectional language navigation
The English `README.md` SHALL link to `README-zh.md` near its beginning, and `README-zh.md` SHALL link to `README.md` near its beginning.

#### Scenario: User selects Chinese from English documentation
- **WHEN** a visitor opens `README.md`
- **THEN** they can navigate directly to the Simplified Chinese guide

#### Scenario: User returns to English documentation
- **WHEN** a visitor opens `README-zh.md`
- **THEN** they can navigate directly to the English README

### Requirement: Defined translation boundary and synchronization
`README-zh.md` SHALL NOT include the maintainer-only `Release process` section. When the English README's Features, Installation, Usage, or License content changes, the corresponding Chinese user-guide content SHALL be updated in the same change. A change limited to `Release process` SHALL NOT require an update to `README-zh.md`.

#### Scenario: User-facing English documentation changes
- **WHEN** a change modifies Features, Installation, Usage, or License content in `README.md`
- **THEN** the same change updates the corresponding content in `README-zh.md`

#### Scenario: Release maintenance instructions change
- **WHEN** a change modifies only `Release process` in `README.md`
- **THEN** `README-zh.md` remains outside the required update scope
