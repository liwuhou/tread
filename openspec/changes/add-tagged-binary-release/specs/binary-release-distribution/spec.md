## ADDED Requirements

### Requirement: Tag-triggered binary release
The release system SHALL publish binary releases only when a version tag matching `v*` is pushed.

#### Scenario: Version tag creates release
- **WHEN** a maintainer pushes a tag named `v0.1.0`
- **THEN** the release workflow SHALL build release artifacts and publish them to the corresponding GitHub Release.

#### Scenario: Branch push does not publish release
- **WHEN** a maintainer pushes commits to `main` without a version tag
- **THEN** the release system SHALL NOT publish GitHub Release artifacts.

### Requirement: Supported binary targets
The release system SHALL build installable `tread` binaries for macOS arm64, macOS x86_64, Linux x86_64, Linux arm64, and Windows x86_64.

#### Scenario: macOS arm64 artifact is produced
- **WHEN** a version tag release runs
- **THEN** a GitHub Release artifact for `aarch64-apple-darwin` SHALL be produced.

#### Scenario: macOS x86_64 artifact is produced
- **WHEN** a version tag release runs
- **THEN** a GitHub Release artifact for `x86_64-apple-darwin` SHALL be produced.

#### Scenario: Linux x86_64 artifact is produced
- **WHEN** a version tag release runs
- **THEN** a GitHub Release artifact for `x86_64-unknown-linux-gnu` SHALL be produced.

#### Scenario: Linux arm64 artifact is produced
- **WHEN** a version tag release runs
- **THEN** a GitHub Release artifact for `aarch64-unknown-linux-gnu` SHALL be produced.

#### Scenario: Windows x86_64 artifact is produced
- **WHEN** a version tag release runs
- **THEN** a GitHub Release artifact for `x86_64-pc-windows-msvc` SHALL be produced.

### Requirement: GitHub Release artifact packaging
The release system SHALL package each platform binary as a downloadable archive and SHALL include checksum data for release integrity verification.

#### Scenario: Release assets are downloadable
- **WHEN** a version tag release completes
- **THEN** the GitHub Release SHALL contain compressed archives for all supported targets.

#### Scenario: Checksums are available
- **WHEN** a version tag release completes
- **THEN** checksum data for the release artifacts SHALL be available from the GitHub Release assets or cargo-dist generated release metadata.

### Requirement: cargo-dist orchestration
The release system SHALL use cargo-dist to define the release target matrix, generated GitHub Actions workflow, release artifacts, and installer metadata.

#### Scenario: Release workflow is generated from cargo-dist configuration
- **WHEN** release automation is configured
- **THEN** the repository SHALL contain cargo-dist configuration and a generated release workflow consistent with that configuration.

#### Scenario: cargo-dist plan includes all supported targets
- **WHEN** the cargo-dist release plan is inspected
- **THEN** it SHALL include macOS arm64, macOS x86_64, Linux x86_64, Linux arm64, and Windows x86_64 targets.

### Requirement: Homebrew prebuilt binary distribution
The release system SHALL support Homebrew installation on macOS using prebuilt GitHub Release binaries rather than building from source.

#### Scenario: Homebrew installs prebuilt macOS binary
- **WHEN** a macOS user installs `tread` from the configured Homebrew tap
- **THEN** Homebrew SHALL download the matching prebuilt GitHub Release artifact for the user's macOS architecture.

#### Scenario: Homebrew formula is updated from release metadata
- **WHEN** a version tag release completes with Homebrew tap publishing enabled
- **THEN** the Homebrew tap formula SHALL reference the new GitHub Release artifact URLs and checksums.

### Requirement: crates.io publication readiness
The crate package metadata SHALL be sufficient for crates.io publication, and release verification SHALL include a dry-run package publish check.

#### Scenario: Crate package dry-run succeeds
- **WHEN** release readiness is verified
- **THEN** `cargo publish --dry-run` SHALL complete successfully for the package.

#### Scenario: Rust users can install from crates.io after publish
- **WHEN** a version has been published to crates.io
- **THEN** Rust users SHALL be able to install the binary with `cargo install tread`.

### Requirement: Release smoke verification
Each supported release binary SHALL be smoke-tested enough to prove that the packaged executable starts on its target platform.

#### Scenario: Packaged executable starts
- **WHEN** a supported target build completes
- **THEN** the release workflow SHALL run the packaged `tread` executable in a startup or usage-display mode appropriate for that platform.

#### Scenario: Failed smoke test blocks release success
- **WHEN** a packaged executable fails its startup smoke test
- **THEN** the release workflow SHALL fail rather than publishing a successful release for that target.
