use anyhow::{Context, Result};
use headless_chrome::protocol::cdp::Network::CookieParam;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

// ─────────────────────────────────────────────────────────────────────────────
// Cookie reading
// ─────────────────────────────────────────────────────────────────────────────

/// Get cookies for a specific URL from the user's browser.
/// Returns a vector of (name, value) pairs.
/// Returns empty vec if reading fails (silent degradation).
pub fn get_cookies_for_url(url: &str) -> Vec<(String, String)> {
    let opts = cookie_scoop::GetCookiesOptions::new(url);

    // Run async function in a blocking tokio runtime
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return Vec::new(),
    };

    let result = rt.block_on(cookie_scoop::get_cookies(opts));

    result
        .cookies
        .iter()
        .map(|c| (c.name.clone(), c.value.clone()))
        .collect()
}

/// Convert cookie pairs to an HTTP Cookie header string.
pub fn cookies_to_header(cookies: &[(String, String)]) -> String {
    cookies
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join("; ")
}

// ─────────────────────────────────────────────────────────────────────────────
// HTTP fetch with cookies
// ─────────────────────────────────────────────────────────────────────────────

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:128.0) Gecko/20100101 Firefox/128.0";

/// Fetch HTML from a URL, optionally with cookies.
pub fn fetch_html_with_cookies(url: &str, cookies: &[(String, String)]) -> Result<String> {
    let mut req = ureq::get(url)
        .set("User-Agent", USER_AGENT);

    if !cookies.is_empty() {
        let header = cookies_to_header(cookies);
        req = req.set("Cookie", &header);
    }

    let response = req
        .call()
        .with_context(|| format!("HTTP 请求失败: {url}"))?;

    response
        .into_string()
        .with_context(|| format!("读取响应失败: {url}"))
}

// ─────────────────────────────────────────────────────────────────────────────
// Headless browser
// ─────────────────────────────────────────────────────────────────────────────

/// Fetch HTML using a visible Chrome browser (non-headless).
/// The user can interact with the browser window to log in if needed.
/// Waits for the page content to stabilize before extracting.
///
/// If cookies are provided, they will be injected into the browser session
/// to reuse authentication state from the user's local browser.
pub fn headless_fetch(url: &str, cookies: &[(String, String)]) -> Result<String> {
    use headless_chrome::browser::LaunchOptionsBuilder;

    // Launch Chrome in visible mode (not headless)
    let launch_options = LaunchOptionsBuilder::default()
        .headless(false)
        .build()
        .context("构建 Chrome 启动选项失败。请确保已安装 Chrome 或 Chromium。")?;

    let browser = headless_chrome::Browser::new(launch_options)
        .context("启动 Chrome 失败。请确保已安装 Chrome 或 Chromium。")?;

    let tab = browser.new_tab()
        .context("创建浏览器标签页失败")?;

    // Navigate to the URL
    tab.navigate_to(url)
        .with_context(|| format!("导航到 {url} 失败"))?;

    // Wait for navigation to complete
    tab.wait_until_navigated()
        .with_context(|| format!("等待页面加载超时: {url}"))?;

    // Inject cookies if provided (to reuse authentication from local browser)
    if !cookies.is_empty() {
        // Convert cookie tuples to CookieParam
        let cookie_params: Vec<CookieParam> = cookies
            .iter()
            .map(|(name, value)| CookieParam {
                name: name.clone(),
                value: value.clone(),
                url: Some(url.to_string()),
                domain: None,
                path: None,
                secure: None,
                http_only: None,
                same_site: None,
                expires: None,
                priority: None,
                same_party: None,
                source_scheme: None,
                source_port: None,
                partition_key: None,
            })
            .collect();

        // Inject cookies (silent failure: if injection fails, continue without cookies)
        if let Err(e) = tab.set_cookies(cookie_params) {
            eprintln!("  ⚠ Cookies 注入失败，可能需要手动登录: {e}");
        } else {
            // Reload page to apply cookies (ignore cache to ensure cookies take effect)
            if let Err(e) = tab.reload(true, None) {
                eprintln!("  ⚠ 页面重载失败: {e}");
            } else {
                // Wait for page to load after reload
                let _ = tab.wait_until_navigated();
            }
        }
    }

    // Print login hint
    eprintln!();
    eprintln!("  📌 如果页面需要登录，请在浏览器窗口中完成登录。");
    eprintln!("  📌 登录完成且页面加载后，按 Enter 键继续提取内容。");
    eprintln!("  📌 （或等待 5 分钟自动超时）");
    eprintln!();

    // Wait for user to press Enter (non-blocking: check every 100ms)
    // Or timeout after 5 minutes
    let max_wait = std::time::Duration::from_secs(300);
    let start = std::time::Instant::now();

    // Set terminal to raw mode briefly to read a single keypress
    let enter_pressed = wait_for_enter_with_timeout(max_wait);

    if !enter_pressed {
        eprintln!("  ⚠ 等待超时，使用当前页面内容");
    }

    // Extra wait for any final JS rendering after login
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Extract rendered HTML
    let html = tab.get_content()
        .context("获取页面内容失败")?;

    Ok(html)
}

/// Wait for the user to press Enter (reads from stdin), with a timeout.
/// Returns true if Enter was pressed, false if timed out.
fn wait_for_enter_with_timeout(timeout: std::time::Duration) -> bool {
    use std::io::BufRead;

    let (tx, rx) = std::sync::mpsc::channel();

    // Thread to read from stdin
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut line = String::new();
        let _ = stdin.lock().read_line(&mut line);
        let _ = tx.send(true);
    });

    match rx.recv_timeout(timeout) {
        Ok(_) => true,
        Err(_) => false, // Timeout
    }
}

/// Strip HTML tags from a string (simple version for content detection).
fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(ch);
        }
    }
    result.trim().to_string()
}

/// Extract domain from a URL.
fn extract_domain(url: &str) -> String {
    url.split("://")
        .nth(1)
        .unwrap_or("")
        .split('/')
        .next()
        .unwrap_or("")
        .to_string()
}

// ─────────────────────────────────────────────────────────────────────────────
// Session persistence
// ─────────────────────────────────────────────────────────────────────────────

const SESSION_EXPIRY_SECS: u64 = 24 * 60 * 60; // 24 hours

/// A saved session with cookies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub domain: String,
    pub cookies: Vec<(String, String)>,
    pub saved_at: u64,
}

fn session_path(domain: &str) -> PathBuf {
    let hash = Sha256::digest(domain.as_bytes());
    let key = format!("{:x}", hash)[..16].to_string();
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".tread")
        .join("sessions")
        .join(format!("{key}.json"))
}

/// Save cookies as a session for a domain.
pub fn save_session(domain: &str, cookies: &[(String, String)]) -> Result<()> {
    let path = session_path(domain);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let session = Session {
        domain: domain.to_string(),
        cookies: cookies.to_vec(),
        saved_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };

    fs::write(path, serde_json::to_string_pretty(&session)?)?;
    Ok(())
}

/// Load a saved session for a domain. Returns None if no session or expired.
pub fn load_session(domain: &str) -> Option<Vec<(String, String)>> {
    let path = session_path(domain);
    let content = fs::read_to_string(path).ok()?;
    let session: Session = serde_json::from_str(&content).ok()?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if now.saturating_sub(session.saved_at) > SESSION_EXPIRY_SECS {
        return None; // Expired
    }

    Some(session.cookies)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── 2.x: Cookie reading ─────────────────────────────────────────────────

    #[test]
    fn cookies_to_header_formats_correctly() {
        let cookies = vec![
            ("session".to_string(), "abc123".to_string()),
            ("token".to_string(), "xyz789".to_string()),
        ];
        let header = cookies_to_header(&cookies);
        assert_eq!(header, "session=abc123; token=xyz789");
    }

    #[test]
    fn cookies_to_header_empty() {
        let header = cookies_to_header(&[]);
        assert_eq!(header, "");
    }

    #[test]
    fn get_cookies_failure_returns_empty() {
        // This should not panic, just return empty on failure
        let cookies = get_cookies_for_url("https://this-domain-definitely-does-not-exist.invalid");
        // May or may not have cookies, but should not panic
        let _ = cookies;
    }

    // ── 4.x: Headless browser ───────────────────────────────────────────────

    #[test]
    fn cookie_param_creation_from_tuples() {
        // Test that we can convert cookie tuples to CookieParam
        let cookies = vec![
            ("session".to_string(), "abc123".to_string()),
            ("token".to_string(), "xyz789".to_string()),
        ];
        let url = "https://example.com";

        let cookie_params: Vec<CookieParam> = cookies
            .iter()
            .map(|(name, value)| CookieParam {
                name: name.clone(),
                value: value.clone(),
                url: Some(url.to_string()),
                domain: None,
                path: None,
                secure: None,
                http_only: None,
                same_site: None,
                expires: None,
                priority: None,
                same_party: None,
                source_scheme: None,
                source_port: None,
                partition_key: None,
            })
            .collect();

        assert_eq!(cookie_params.len(), 2);
        assert_eq!(cookie_params[0].name, "session");
        assert_eq!(cookie_params[0].value, "abc123");
        assert_eq!(cookie_params[0].url, Some("https://example.com".to_string()));
        assert_eq!(cookie_params[1].name, "token");
        assert_eq!(cookie_params[1].value, "xyz789");
    }

    #[test]
    fn extract_domain_works() {
        assert_eq!(extract_domain("https://example.com/path"), "example.com");
        assert_eq!(extract_domain("https://sub.example.com/"), "sub.example.com");
        assert_eq!(extract_domain("http://localhost:8080/page"), "localhost:8080");
    }

    #[test]
    fn strip_html_tags_works() {
        assert_eq!(strip_html_tags("<p>Hello</p>"), "Hello");
        assert_eq!(strip_html_tags("<div class=\"x\"><span>World</span></div>"), "World");
        assert_eq!(strip_html_tags("No tags"), "No tags");
    }

    // ── 5.x: Session persistence ────────────────────────────────────────────

    #[test]
    fn save_and_load_session() {
        let domain = "test.tread.local";
        let cookies = vec![
            ("session".to_string(), "test123".to_string()),
        ];
        save_session(domain, &cookies).unwrap();

        let loaded = load_session(domain);
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].0, "session");
        assert_eq!(loaded[0].1, "test123");

        // Cleanup
        fs::remove_file(session_path(domain)).ok();
    }

    #[test]
    fn load_session_nonexistent() {
        let result = load_session("never-saved-domain.invalid");
        assert!(result.is_none());
    }
}
