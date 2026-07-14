use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

// ─────────────────────────────────────────────────────────────────────────────
// Data types
// ─────────────────────────────────────────────────────────────────────────────

/// A web page with extracted content.
#[derive(Debug, Clone)]
pub struct WebPage {
    pub title: String,
    pub url: String,
    pub content_html: String,
}

/// Cache metadata for a web page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebCacheMeta {
    pub url: String,
    pub title: String,
    pub fetched_at: u64,
}

/// Reading progress for a web page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebProgress {
    pub url: String,
    pub scroll: usize,
    pub saved_at: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// HTTP fetch
// ─────────────────────────────────────────────────────────────────────────────

const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:128.0) Gecko/20100101 Firefox/128.0";

/// Fetch HTML from a URL with Firefox User-Agent.
pub fn fetch_html(url: &str) -> Result<String> {
    let response = ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .call()
        .with_context(|| format!("HTTP 请求失败: {url}"))?;

    response
        .into_string()
        .with_context(|| format!("读取响应失败: {url}"))
}

// ─────────────────────────────────────────────────────────────────────────────
// Readability extraction
// ─────────────────────────────────────────────────────────────────────────────

/// Extract readable content from HTML using Readability algorithm.
pub fn extract_content(html: &str, url: &str) -> Result<WebPage> {
    let mut readability = dom_smoothie::Readability::new(html, Some(url), None)
        .map_err(|e| anyhow::anyhow!("Readability 解析失败: {e}"))?;

    let article = readability
        .parse()
        .map_err(|e| anyhow::anyhow!("Readability 提取失败: {e}"))?;

    let title = article.title.to_string();
    let content = article.content.to_string();

    if content.trim().is_empty() {
        anyhow::bail!("无法提取正文：页面内容为空");
    }

    Ok(WebPage {
        title,
        url: url.to_string(),
        content_html: content,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Cache management
// ─────────────────────────────────────────────────────────────────────────────

const CACHE_EXPIRY_SECS: u64 = 24 * 60 * 60; // 24 hours

/// Return the web cache directory: `~/.tread/cache/web/`
pub fn web_cache_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".tread").join("cache").join("web"))
}

/// Generate a short cache key from a URL.
pub fn web_cache_key(url: &str) -> String {
    let hash = Sha256::digest(url.as_bytes());
    format!("{:x}", hash)[..16].to_string()
}

/// Save HTML and metadata to cache.
pub fn save_web_cache(url: &str, page: &WebPage) -> Result<()> {
    let dir = web_cache_dir().ok_or_else(|| anyhow::anyhow!("无法确定缓存目录"))?;
    fs::create_dir_all(&dir)?;

    let key = web_cache_key(url);
    let html_path = dir.join(format!("{key}.html"));
    let meta_path = dir.join(format!("{key}.meta.json"));

    fs::write(&html_path, &page.content_html)?;

    let meta = WebCacheMeta {
        url: url.to_string(),
        title: page.title.clone(),
        fetched_at: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };
    fs::write(&meta_path, serde_json::to_string_pretty(&meta)?)?;

    Ok(())
}

/// Load a web page from cache. Returns None if cache doesn't exist or is expired.
pub fn load_web_cache(url: &str) -> Option<WebPage> {
    let dir = web_cache_dir()?;
    let key = web_cache_key(url);
    let html_path = dir.join(format!("{key}.html"));
    let meta_path = dir.join(format!("{key}.meta.json"));

    if !html_path.exists() || !meta_path.exists() {
        return None;
    }

    let meta: WebCacheMeta = serde_json::from_str(&fs::read_to_string(&meta_path).ok()?).ok()?;

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if now.saturating_sub(meta.fetched_at) > CACHE_EXPIRY_SECS {
        return None; // Expired
    }

    let content_html = fs::read_to_string(&html_path).ok()?;

    Some(WebPage {
        title: meta.title,
        url: url.to_string(),
        content_html,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// URL utilities
// ─────────────────────────────────────────────────────────────────────────────

/// Extract the domain from a URL (e.g., "https://example.com/path" → "example.com").
pub fn extract_domain(url: &str) -> String {
    url.split("://")
        .nth(1)
        .unwrap_or("")
        .split('/')
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("")
        .to_string()
}

// ─────────────────────────────────────────────────────────────────────────────
// Image URL resolution
// ─────────────────────────────────────────────────────────────────────────────

/// Resolve a potentially relative image URL against a base URL.
pub fn resolve_image_url(src: &str, base_url: &str) -> String {
    // Already absolute
    if src.starts_with("http://") || src.starts_with("https://") || src.starts_with("//") {
        if src.starts_with("//") {
            // Protocol-relative
            let base_scheme = base_url.split("://").next().unwrap_or("https");
            return format!("{base_scheme}:{src}");
        }
        return src.to_string();
    }

    // Parse base URL
    let base_parts: Vec<&str> = base_url.splitn(3, "://").collect();
    if base_parts.len() < 2 {
        return src.to_string();
    }
    let scheme = base_parts[0];
    let rest = base_parts[1];
    let (host, base_path) = match rest.find('/') {
        Some(pos) => (&rest[..pos], &rest[pos..]),
        None => (rest, "/"),
    };

    if src.starts_with('/') {
        // Absolute path on same host
        format!("{scheme}://{host}{src}")
    } else {
        // Relative path — resolve against base_path
        let mut base_dir: String = base_path.to_string();
        // Remove filename (last segment)
        if let Some(pos) = base_dir.rfind('/') {
            base_dir = base_dir[..=pos].to_string();
        }

        // Handle ../
        let mut combined = format!("{base_dir}{src}");
        while combined.contains("/../") {
            if let Some(pos) = combined.find("/../") {
                let before = &combined[..pos];
                let after = &combined[pos + 4..];
                let parent = match before.rfind('/') {
                    Some(p) => &before[..p],
                    None => "",
                };
                combined = format!("{parent}/{after}");
            } else {
                break;
            }
        }

        // Handle ./
        combined = combined.replace("/./", "/");

        format!("{scheme}://{host}{combined}")
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Progress persistence
// ─────────────────────────────────────────────────────────────────────────────

fn progress_path(url: &str) -> PathBuf {
    let key = web_cache_key(url);
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".tread")
        .join("progress")
        .join(format!("web_{key}.json"))
}

/// Save web reading progress.
pub fn save_web_progress(url: &str, scroll: usize) -> Result<()> {
    let path = progress_path(url);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let progress = WebProgress {
        url: url.to_string(),
        scroll,
        saved_at: chrono_now(),
    };
    fs::write(path, serde_json::to_string_pretty(&progress)?)?;
    Ok(())
}

/// Load web reading progress. Returns None if no progress file exists.
pub fn load_web_progress(url: &str) -> Option<WebProgress> {
    let path = progress_path(url);
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Simple ISO 8601 timestamp without chrono dependency.
fn chrono_now() -> String {
    use std::time::SystemTime;
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple epoch → "2026-06-30T10:00:00Z" approximation
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let mins = (time_secs % 3600) / 60;
    let sec = time_secs % 60;

    // Days since 1970-01-01 → year/month/day (simplified)
    let (y, m, d) = days_to_ymd(days);
    format!("{y:04}-{m:02}-{d:02}T{hours:02}:{mins:02}:{sec:02}Z")
}

fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    // Algorithm from https://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Check if a string is a URL.
pub fn is_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

/// Convert extracted HTML content to LineContent for terminal rendering.
/// Resolves relative image URLs against the base URL.
pub fn html_to_lines(html: &str, base_url: &str) -> Vec<crate::image::LineContent> {
    // Wrap the content fragment in a minimal XHTML document
    let wrapped = format!(
        r#"<html xmlns="http://www.w3.org/1999/xhtml"><body>{}</body></html>"#,
        html
    );

    // Create an image resolver that resolves relative URLs
    let base = base_url.to_string();
    let resolver = |href: &str| -> Option<std::path::PathBuf> {
        let resolved = resolve_image_url(href, &base);
        // Try to download/cache the image
        crate::image::download_image(&resolved).ok()
    };

    crate::xhtml::xhtml_to_lines(&wrapped, Some(&resolver))
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── 2.x: HTTP fetch ─────────────────────────────────────────────────────

    #[test]
    fn fetch_html_invalid_url_returns_error() {
        let result = fetch_html("http://this-domain-does-not-exist-12345.invalid/page");
        assert!(result.is_err());
    }

    // ── 3.x: Readability extraction ─────────────────────────────────────────

    #[test]
    fn extract_content_from_article() {
        let html = r#"<html><head><title>Test Article</title></head>
        <body>
        <nav>Navigation</nav>
        <article>
        <h1>Test Article</h1>
        <p>This is the main content of the article. It has multiple sentences.</p>
        <p>Another paragraph with more text content for readability scoring.</p>
        <p>Yet another paragraph to ensure the content is substantial enough for extraction.</p>
        <p>More content here to boost the score above the threshold.</p>
        <p>Final paragraph with additional text.</p>
        </article>
        <footer>Footer content</footer>
        </body></html>"#;

        let result = extract_content(html, "https://example.com/article");
        assert!(result.is_ok(), "extract_content failed: {:?}", result.err());
        let page = result.unwrap();
        assert!(!page.content_html.is_empty());
        assert!(page.content_html.contains("main content"));
    }

    #[test]
    fn extract_content_empty_html_returns_error() {
        let html = "<html><body></body></html>";
        let result = extract_content(html, "https://example.com");
        assert!(result.is_err());
    }

    // ── 4.x: Cache management ───────────────────────────────────────────────

    #[test]
    fn web_cache_dir_path() {
        let dir = web_cache_dir().unwrap();
        assert!(dir.to_str().unwrap().ends_with(".tread/cache/web"));
    }

    #[test]
    fn web_cache_key_deterministic() {
        let k1 = web_cache_key("https://example.com/a");
        let k2 = web_cache_key("https://example.com/a");
        let k3 = web_cache_key("https://example.com/b");
        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
        assert_eq!(k1.len(), 16);
    }

    #[test]
    fn save_and_load_web_cache() {
        let url = "https://test.tread.local/cache_test";
        let page = WebPage {
            title: "Test".to_string(),
            url: url.to_string(),
            content_html: "<p>Cached content</p>".to_string(),
        };
        save_web_cache(url, &page).unwrap();
        let loaded = load_web_cache(url);
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.title, "Test");
        assert_eq!(loaded.content_html, "<p>Cached content</p>");

        // Cleanup
        let dir = web_cache_dir().unwrap();
        let key = web_cache_key(url);
        fs::remove_file(dir.join(format!("{key}.html"))).ok();
        fs::remove_file(dir.join(format!("{key}.meta.json"))).ok();
    }

    #[test]
    fn load_web_cache_nonexistent() {
        let result = load_web_cache("https://never-cached-before.local/page");
        assert!(result.is_none());
    }

    // ── 5.x: Image URL resolution ───────────────────────────────────────────

    #[test]
    fn resolve_absolute_url() {
        let result = resolve_image_url(
            "https://cdn.example.com/pic.jpg",
            "https://example.com/article/",
        );
        assert_eq!(result, "https://cdn.example.com/pic.jpg");
    }

    #[test]
    fn resolve_root_relative() {
        let result = resolve_image_url("/images/pic.jpg", "https://example.com/article/page.html");
        assert_eq!(result, "https://example.com/images/pic.jpg");
    }

    #[test]
    fn resolve_relative_path() {
        let result = resolve_image_url("pic.jpg", "https://example.com/articles/page.html");
        assert_eq!(result, "https://example.com/articles/pic.jpg");
    }

    #[test]
    fn resolve_parent_relative() {
        let result = resolve_image_url(
            "../assets/pic.jpg",
            "https://example.com/articles/deep/page.html",
        );
        assert_eq!(result, "https://example.com/articles/assets/pic.jpg");
    }

    #[test]
    fn resolve_protocol_relative() {
        let result = resolve_image_url("//cdn.example.com/pic.jpg", "https://example.com/");
        assert_eq!(result, "https://cdn.example.com/pic.jpg");
    }

    // ── 7.x: Progress persistence ───────────────────────────────────────────

    #[test]
    fn save_and_load_web_progress() {
        let url = "https://test.tread.local/progress_test";
        save_web_progress(url, 42).unwrap();
        let loaded = load_web_progress(url);
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.scroll, 42);
        assert_eq!(loaded.url, url);

        // Cleanup
        fs::remove_file(progress_path(url)).ok();
    }

    #[test]
    fn load_web_progress_nonexistent() {
        let result = load_web_progress("https://never-progressed-before.local/page");
        assert!(result.is_none());
    }

    // ── is_url ──────────────────────────────────────────────────────────────

    #[test]
    fn is_url_detects_http() {
        assert!(is_url("http://example.com"));
        assert!(is_url("https://example.com"));
        assert!(!is_url("./local.md"));
        assert!(!is_url("file.epub"));
        assert!(!is_url("/absolute/path"));
    }
}
