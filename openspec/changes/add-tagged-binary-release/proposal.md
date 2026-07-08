## Why

`tread` currently has no automated release path for users who want a ready-to-run binary. Tag-triggered release automation will let each version produce installable artifacts for Rust users, macOS users, and common terminal platforms without requiring manual local builds.

## What Changes

- Add cargo-dist based release automation for tag-triggered binary releases.
- Build and publish GitHub Release artifacts for:
  - `aarch64-apple-darwin`
  - `x86_64-apple-darwin`
  - `x86_64-unknown-linux-gnu`
  - `aarch64-unknown-linux-gnu`
  - `x86_64-pc-windows-msvc`
- Configure Homebrew distribution to install prebuilt GitHub Release binaries, not build from source.
- Prepare crates.io publication metadata so Rust users can install with `cargo install tread`.
- Add release verification steps that confirm packaged binaries start successfully on each target.
- Do not add nightly builds or server deployment.
- No breaking changes.

## Capabilities

### New Capabilities
- `binary-release-distribution`: Defines tag-triggered build, packaging, GitHub Release, crates.io readiness, and Homebrew binary distribution behavior.

### Modified Capabilities
- None.

## Impact

- Adds release configuration for cargo-dist and GitHub Actions.
- May add or update package metadata in `Cargo.toml` for crates.io and cargo-dist.
- May add project documentation required for publication, such as README and license metadata.
- Requires repository secrets or tokens for optional crates.io publishing and Homebrew tap updates:
  - `CARGO_REGISTRY_TOKEN` if crates.io publishing is automated.
  - GitHub token or PAT with access to the Homebrew tap repository if formula updates are automated.
- Release trigger is limited to version tags matching `v*`.
