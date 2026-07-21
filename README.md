# tread

`tread` is a terminal-based reader for Markdown, EPUB, and web content.

[中文指南](README-zh.md)

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
tread                         # open the reading dashboard
tread <file.md|file.epub|url> [-r|--refresh] [-i|--interactive]
```

Options:

- `-r`, `--refresh`: force refresh for web pages and skip cache.
- `-i`, `--interactive`: use Chrome for dynamic pages.

Dashboard keys:

- `j` / `k` / arrow keys: move between history entries.
- `Enter`: continue the selected URL, EPUB, or Markdown file from saved progress.
- `o`: open a new local path or URL from the dashboard prompt; URLs can include `-i`/`--interactive` and `-r`/`--refresh`, for example `https://example.com -i`.
- `s`: star or unstar the selected entry; starred entries are shown first.
- `r`: remove the selected entry from the dashboard without deleting the file.
- `q`: quit the dashboard.

The dashboard stores unified reading history in `~/.tread/history.json`. Reopening a hidden target makes it visible again and clears its starred state.
When the terminal is very short, the dashboard prioritizes the recent-reading area and may hide footer hints, prompt suggestions, or prompt/error text until more height is available.

Keys:

- `j` / `k` / arrow keys: scroll one line.
- `Ctrl+d` / `Ctrl+u`: scroll half a page.
- `g` / `G`: jump to top or bottom.
- `D`: return to the dashboard.
- `Tab`: focus the next image or link.
- `Enter`: open the focused image or link.
- `Ctrl+n` / `Ctrl+p`: next or previous EPUB chapter.
- `t`: EPUB table of contents.
- `?`: help.
- `q`: quit.

## Release process

Maintainers publish releases by pushing a version tag:

```sh
git tag v0.1.1
git push origin v0.1.1
```

The release workflow builds platform archives with cargo-dist and publishes them to GitHub Releases. Homebrew distribution is configured to consume the prebuilt GitHub Release binaries from `liwuhou/tread` and publish formula updates to the `liwuhou/homebrew-tap` repository.

Release prerequisites:

- `HOMEBREW_TAP_TOKEN`: GitHub token with permission to push to `liwuhou/homebrew-tap`.
- crates.io publishing is intentionally manual unless a maintainer chooses to add `CARGO_REGISTRY_TOKEN` and a separate publish step.

Before tagging a release, verify packaging locally:

```sh
dist plan --tag v0.1.1
cargo publish --dry-run --allow-dirty
```

After a successful crates.io publish, Rust users can install with `cargo install tread`.

## License

MIT
