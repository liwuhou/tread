# tread

`tread` is a terminal-based reader for Markdown, EPUB, and web content.

## Features

- Read local Markdown files in a terminal UI.
- Read EPUB files with table-of-contents navigation.
- Fetch and render web pages from URLs.
- Optionally use Chrome-powered interactive fetching for dynamic pages.
- Open links and images from the reader interface where supported by the operating system.

## Installation

### Cargo

```sh
cargo install tread
```

### Homebrew

```sh
brew install liwuhou/tap/tread
```

### GitHub Releases

Download the archive for your platform from the GitHub Releases page:

- `aarch64-apple-darwin` for macOS Apple Silicon
- `x86_64-apple-darwin` for macOS Intel
- `x86_64-unknown-linux-gnu` for Linux x86_64
- `aarch64-unknown-linux-gnu` for Linux arm64
- `x86_64-pc-windows-msvc` for Windows x86_64

## Usage

```sh
tread <file.md|file.epub|url> [-r|--refresh] [-i|--interactive]
```

Options:

- `-r`, `--refresh`: force refresh for web pages and skip cache.
- `-i`, `--interactive`: use Chrome for dynamic pages.

Keys:

- `j` / `k` / arrow keys: scroll one line.
- `Ctrl+d` / `Ctrl+u`: scroll half a page.
- `g` / `G`: jump to top or bottom.
- `Tab`: focus the next image or link.
- `Enter`: open the focused image or link.
- `Ctrl+n` / `Ctrl+p`: next or previous EPUB chapter.
- `t`: EPUB table of contents.
- `?`: help.
- `q`: quit.

## Release process

Maintainers publish releases by pushing a version tag:

```sh
git tag v0.1.0
git push origin v0.1.0
```

The release workflow builds platform archives with cargo-dist and publishes them to GitHub Releases. Homebrew distribution is configured to consume the prebuilt GitHub Release binaries from `liwuhou/tread` and publish formula updates to the `liwuhou/homebrew-tap` repository.

Release prerequisites:

- `HOMEBREW_TAP_TOKEN`: GitHub token with permission to push to `liwuhou/homebrew-tap`.
- crates.io publishing is intentionally manual unless a maintainer chooses to add `CARGO_REGISTRY_TOKEN` and a separate publish step.

Before tagging a release, verify packaging locally:

```sh
dist plan --tag v0.1.0
cargo publish --dry-run --allow-dirty
```

After a successful crates.io publish, Rust users can install with `cargo install tread`.

## License

MIT
