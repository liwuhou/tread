use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// An image node extracted from Markdown.
#[derive(Debug, Clone)]
pub struct ImageNode {
    /// Alt text (may be empty).
    pub alt: String,
    /// Original URL or path from Markdown source.
    pub url: String,
    /// Resolved local path (after caching or direct reference).
    /// `None` if download failed or not yet resolved.
    pub local_path: Option<PathBuf>,
    /// Unique index for focus navigation.
    #[allow(dead_code)]
    pub id: usize,
    /// Whether a remote download was attempted and failed.
    pub download_failed: bool,
}

/// A hyperlink node extracted from Markdown/XHTML.
#[derive(Debug, Clone)]
pub struct LinkNode {
    /// Display text of the link.
    pub text: String,
    /// The URL / href value.
    pub url: String,
    /// Whether this is an external (http/https) link.
    pub is_external: bool,
}

/// A content line — either styled text, an image placeholder, or a link.
#[derive(Debug, Clone)]
pub enum LineContent {
    /// Normal styled text line.
    Styled(Vec<(String, ratatui::style::Style)>),
    /// Image placeholder (occupies one visual line).
    Image(ImageNode),
    /// Hyperlink (occupies one visual line, rendered as styled text).
    Link(LinkNode),
}

// ─────────────────────────────────────────────────────────────────────────────
// Cache helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Return the cache directory: `~/.tread/cache/`
pub fn cache_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".tread").join("cache"))
}

/// Ensure the cache directory exists. Idempotent.
pub fn ensure_cache_dir() -> std::io::Result<()> {
    if let Some(dir) = cache_dir() {
        fs::create_dir_all(&dir)?;
    }
    Ok(())
}

/// Compute the cache file path for a given URL.
/// Returns `<cache_dir>/<sha256(url)>.<ext>`
pub fn url_to_cache_path(url: &str) -> Option<PathBuf> {
    let dir = cache_dir()?;
    let hash = Sha256::digest(url.as_bytes());
    let hex = format!("{:x}", hash);
    let ext = guess_extension(url).unwrap_or_else(|| "png".to_string());
    Some(dir.join(format!("{hex}.{ext}")))
}

/// Guess file extension from URL path.
fn guess_extension(url: &str) -> Option<String> {
    // Strip query string and fragment
    let path = url.split('?').next().unwrap_or(url);
    let path = path.split('#').next().unwrap_or(path);
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());
    ext.filter(|e| !e.is_empty())
}

/// Resolve a local image path (relative or absolute).
/// Returns the absolute path if the file exists, or `None`.
pub fn resolve_image_path(url: &str, base_dir: Option<&Path>) -> Option<PathBuf> {
    let path = Path::new(url);
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else if let Some(base) = base_dir {
        base.join(path)
    } else {
        std::env::current_dir().ok()?.join(path)
    };
    if absolute.exists() {
        Some(absolute)
    } else {
        None
    }
}

/// Check if a URL is remote (http/https).
pub fn is_remote_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

/// Download a remote image to the cache. Returns the cached path on success.
/// Uses Content-Type header to determine file extension when URL has none.
pub fn download_image(url: &str) -> Result<PathBuf, String> {
    let mut cache_path = url_to_cache_path(url).ok_or("无法确定缓存目录")?;

    // Cache hit
    if cache_path.exists() {
        return Ok(cache_path);
    }

    ensure_cache_dir().map_err(|e| format!("创建缓存目录失败: {e}"))?;

    // Download
    let response = ureq::get(url)
        .call()
        .map_err(|e| format!("下载失败: {e}"))?;

    // Use Content-Type to determine extension if URL has a generic one
    let content_type = response.content_type().to_string();
    let mut bytes = Vec::new();
    let mut reader = response.into_reader();
    std::io::Read::read_to_end(&mut reader, &mut bytes)
        .map_err(|e| format!("读取响应失败: {e}"))?;

    // If the cached path has a generic extension (.img), try Content-Type
    if cache_path.extension().and_then(|e| e.to_str()) == Some("img") {
        if let Some(ext) = content_type_to_ext(&content_type) {
            cache_path.set_extension(ext);
        }
    }

    fs::write(&cache_path, &bytes).map_err(|e| format!("写入缓存失败: {e}"))?;

    Ok(cache_path)
}

/// Map a Content-Type string to a file extension.
fn content_type_to_ext(ct: &str) -> Option<&str> {
    match ct.split(';').next().unwrap_or(ct).trim() {
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/gif" => Some("gif"),
        "image/webp" => Some("webp"),
        "image/svg+xml" => Some("svg"),
        "image/bmp" => Some("bmp"),
        "image/tiff" => Some("tiff"),
        "image/x-icon" | "image/vnd.microsoft.icon" => Some("ico"),
        "image/avif" => Some("avif"),
        _ => None,
    }
}

/// Open a file with the system default image viewer.
pub fn open_with_viewer(path: &Path) -> Result<(), String> {
    let cmd = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "linux") {
        "xdg-open"
    } else {
        return Err("不支持的操作系统".to_string());
    };

    std::process::Command::new(cmd)
        .arg(path)
        .spawn()
        .map_err(|e| format!("打开图片失败: {e}"))?;

    Ok(())
}

/// Open a URL with the system default browser.
pub fn open_url(url: &str) -> Result<(), String> {
    let cmd = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "linux") {
        "xdg-open"
    } else {
        return Err("不支持的操作系统".to_string());
    };

    std::process::Command::new(cmd)
        .arg(url)
        .spawn()
        .map_err(|e| format!("打开链接失败: {e}"))?;

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // ── 2.2: resolve_image_path ──────────────────────────────────────────────

    #[test]
    fn resolve_absolute_existing_path() {
        // Use a known existing file
        let path = "/etc/hosts";
        let result = resolve_image_path(path, None);
        assert!(result.is_some());
        assert!(result.unwrap().is_absolute());
    }

    #[test]
    fn resolve_nonexistent_path_returns_none() {
        let result = resolve_image_path("/nonexistent/image.png", None);
        assert!(result.is_none());
    }

    #[test]
    fn resolve_relative_path_with_base_dir() {
        // Create a temp file
        let tmp = std::env::temp_dir().join("tread_test_img.png");
        fs::write(&tmp, b"fake image").unwrap();

        let result = resolve_image_path("tread_test_img.png", Some(std::env::temp_dir().as_path()));
        assert!(result.is_some());
        assert!(result.unwrap().ends_with("tread_test_img.png"));

        fs::remove_file(&tmp).ok();
    }

    // ── 2.3: cache_dir ───────────────────────────────────────────────────────

    #[test]
    fn cache_dir_ends_with_tread_cache() {
        let dir = cache_dir();
        assert!(dir.is_some());
        let dir = dir.unwrap();
        assert!(dir.ends_with(".tread/cache"));
    }

    // ── 2.4: url_to_cache_path ───────────────────────────────────────────────

    #[test]
    fn url_to_cache_path_uses_sha256() {
        let path = url_to_cache_path("https://example.com/photo.jpg").unwrap();
        let filename = path.file_name().unwrap().to_str().unwrap();
        // Should be <hex>.jpg
        assert!(filename.ends_with(".jpg"));
        let hex_part = &filename[..filename.len() - 4];
        assert_eq!(hex_part.len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn same_url_produces_same_path() {
        let p1 = url_to_cache_path("https://example.com/a.png").unwrap();
        let p2 = url_to_cache_path("https://example.com/a.png").unwrap();
        assert_eq!(p1, p2);
    }

    #[test]
    fn different_urls_produce_different_paths() {
        let p1 = url_to_cache_path("https://example.com/a.png").unwrap();
        let p2 = url_to_cache_path("https://example.com/b.png").unwrap();
        assert_ne!(p1, p2);
    }

    #[test]
    fn url_with_query_string_ignores_query() {
        let p1 = url_to_cache_path("https://example.com/a.png?v=1").unwrap();
        let p2 = url_to_cache_path("https://example.com/a.png?v=2").unwrap();
        // Both should have .png extension (from path before query)
        assert!(p1.to_str().unwrap().ends_with(".png"));
        assert!(p2.to_str().unwrap().ends_with(".png"));
    }

    // ── 2.5: ensure_cache_dir ────────────────────────────────────────────────

    #[test]
    fn ensure_cache_dir_is_idempotent() {
        ensure_cache_dir().unwrap();
        ensure_cache_dir().unwrap(); // second call should not fail
        let dir = cache_dir().unwrap();
        assert!(dir.exists());
    }

    // ── 2.6: download_image (integration — uses real HTTP) ───────────────────

    #[test]
    fn download_image_returns_cached_path_for_existing() {
        // Pre-create a cache file
        let url = "https://test.tread.local/fake_cached.jpg";
        let cache_path = url_to_cache_path(url).unwrap();
        ensure_cache_dir().unwrap();
        fs::write(&cache_path, b"fake image data").unwrap();

        let result = download_image(url);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), cache_path);

        fs::remove_file(&cache_path).ok();
    }

    // ── is_remote_url ────────────────────────────────────────────────────────

    #[test]
    fn is_remote_url_detects_http() {
        assert!(is_remote_url("http://example.com/a.png"));
        assert!(is_remote_url("https://example.com/a.png"));
        assert!(!is_remote_url("./local.png"));
        assert!(!is_remote_url("/absolute/path.png"));
    }

    // ── LinkNode ─────────────────────────────────────────────────────────────

    #[test]
    fn link_node_external() {
        let link = super::LinkNode {
            text: "click".to_string(),
            url: "https://example.com".to_string(),
            is_external: true,
        };
        assert_eq!(link.text, "click");
        assert!(link.is_external);
    }

    #[test]
    fn link_node_internal() {
        let link = super::LinkNode {
            text: "next".to_string(),
            url: "chapter2.xhtml".to_string(),
            is_external: false,
        };
        assert!(!link.is_external);
    }

    #[test]
    fn line_content_link_variant() {
        let link = super::LinkNode {
            text: "test".to_string(),
            url: "https://example.com".to_string(),
            is_external: true,
        };
        let lc = super::LineContent::Link(link);
        match lc {
            super::LineContent::Link(l) => assert_eq!(l.text, "test"),
            _ => panic!("expected Link variant"),
        }
    }
}
