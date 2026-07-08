## Context

`tread` is a Rust CLI distributed as a single binary. The repository currently has no `.github/workflows` release automation, no cargo-dist configuration, and no project README at the repository root. Users who want to install it must build locally.

The desired distribution model is tag-driven release engineering:

```text
git tag vX.Y.Z
    │
    ▼
GitHub Actions / cargo-dist
    │
    ├─ build platform binaries
    ├─ package archives and checksums
    ├─ publish GitHub Release assets
    ├─ make crates.io publishing possible
    └─ update Homebrew tap formula for prebuilt macOS binaries
```

## Goals / Non-Goals

**Goals:**
- Use cargo-dist as the release orchestration tool instead of hand-written release packaging.
- Release only from version tags matching `v*`.
- Produce GitHub Release binaries for:
  - `aarch64-apple-darwin`
  - `x86_64-apple-darwin`
  - `x86_64-unknown-linux-gnu`
  - `aarch64-unknown-linux-gnu`
  - `x86_64-pc-windows-msvc`
- Configure Homebrew to install prebuilt GitHub Release binaries rather than compiling from source.
- Prepare crates.io metadata so Rust users can install with `cargo install tread`.
- Include release smoke checks that prove packaged binaries can start.

**Non-Goals:**
- Nightly builds.
- Server deployment.
- Publishing on every push to `main`.
- Linux musl targets in the first release automation pass.
- Guaranteeing full runtime parity for every optional OS integration beyond binary startup; Windows open-link/open-image behavior can be handled as a separate compatibility change if needed.

## Decisions

### Use cargo-dist for release automation

Use cargo-dist to generate and maintain the release workflow and distribution metadata.

Alternatives considered:
- Hand-written GitHub Actions: more transparent but higher maintenance for archive naming, checksums, installers, Homebrew integration, and platform matrix handling.
- release-plz/cargo-release only: useful for versioning and crates.io automation, but not focused on multi-platform binary distribution.

Rationale: cargo-dist directly matches this change: Rust CLI, GitHub Releases, target matrix, checksums, installers, and Homebrew tap integration.

### Treat GitHub Release as the canonical binary source

Release artifacts on GitHub SHALL be the source used by direct downloads and Homebrew formula updates.

Rationale: this avoids producing different binaries through different channels. Homebrew users receive the same prebuilt artifacts attached to the tag release.

### Support GNU Linux first, not musl

Linux targets SHALL start with:
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`

Rationale: GNU targets are less risky for the current dependency set. musl can be added later if static Linux binaries become a requirement.

### Keep crates.io publication readiness separate from binary release success

The implementation SHALL add required package metadata and verify `cargo publish --dry-run`. Automatic `cargo publish` may be enabled only when a crates.io token is configured.

Rationale: crates.io publishes are permanent for each version. Dry-run verification is safe by default; actual publishing should be explicitly controlled by repository secrets and release policy.

### Prefer prebuilt Homebrew binary formula

The Homebrew tap integration SHALL consume GitHub Release binaries, not source archives with a Rust build step.

Rationale: the user goal is installable binaries. Prebuilt formulae give macOS users faster installs and align Homebrew artifacts with GitHub Release artifacts.

## Risks / Trade-offs

- Cross-compiling Linux arm64 may require cargo-dist-supported runner configuration or native emulation. → Validate generated workflow and keep the target matrix explicit.
- macOS x86_64 builds may require an x86-capable runner or cross-target support from macOS runners. → Verify cargo-dist's generated plan before relying on release tags.
- Homebrew tap updates require permissions outside this repository. → Document required tap repository and token before enabling automatic formula updates.
- crates.io package metadata may require a license and README decision. → Add publication metadata as part of implementation before running `cargo publish --dry-run`.
- Windows binary may build while some optional interactions return unsupported OS errors. → Scope release smoke checks to binary startup and track Windows UX compatibility separately if needed.
- cargo-dist generated workflow may change across cargo-dist versions. → Pin or record the cargo-dist version used to generate release automation.
