## 1. Release Tooling Setup

- [x] 1.1 Install or invoke a pinned cargo-dist version for release configuration generation
- [x] 1.2 Add cargo-dist configuration for the `tread` binary and supported targets
- [x] 1.3 Generate the cargo-dist GitHub Actions release workflow
- [x] 1.4 Confirm the generated release workflow triggers on `v*` tags and not ordinary branch pushes

## 2. Package Metadata Readiness

- [x] 2.1 Add crates.io-required package metadata to `Cargo.toml`
- [x] 2.2 Add repository-root README content required for package publication and installation guidance
- [x] 2.3 Add or reference license metadata required for crates.io and release distribution
- [x] 2.4 Run `cargo publish --dry-run` and resolve packaging issues

## 3. Target Matrix and Binary Packaging

- [x] 3.1 Configure release target `aarch64-apple-darwin`
- [x] 3.2 Configure release target `x86_64-apple-darwin`
- [x] 3.3 Configure release target `x86_64-unknown-linux-gnu`
- [x] 3.4 Configure release target `aarch64-unknown-linux-gnu`
- [x] 3.5 Configure release target `x86_64-pc-windows-msvc`
- [x] 3.6 Verify cargo-dist release planning includes all supported targets and archives

## 4. GitHub Release Distribution

- [x] 4.1 Configure GitHub Release asset publishing through cargo-dist
- [x] 4.2 Verify generated artifact names include package version and target platform
- [x] 4.3 Verify checksum or cargo-dist release metadata is generated for release artifacts
- [x] 4.4 Document tag-based release procedure for maintainers

## 5. Homebrew and crates.io Channels

- [x] 5.1 Configure Homebrew tap integration to use prebuilt GitHub Release binaries
- [x] 5.2 Document the required Homebrew tap repository and token permissions
- [x] 5.3 Configure crates.io publish behavior only when `CARGO_REGISTRY_TOKEN` is available, or document manual publish if automation is intentionally deferred
- [x] 5.4 Verify documented install commands cover GitHub Release download, Homebrew, and `cargo install tread`

## 6. Verification

- [x] 6.1 Run the local cargo-dist generation or plan command and inspect the target matrix
- [x] 6.2 Run Rust checks required before release: format check, tests, and release build where locally supported
- [x] 6.3 Verify release workflow contains startup smoke checks for packaged executables
- [x] 6.4 Validate OpenSpec requirements for `binary-release-distribution` are satisfied
