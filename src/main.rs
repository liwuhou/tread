use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::PathBuf;

mod app;
mod epub;
mod headless;
mod image;
mod parser;
mod style_serde;
mod ui;
mod web;
mod xhtml;

/// Resolve image paths: local files get absolute paths, remote URLs get downloaded to cache.
fn resolve_images(lines: &mut Vec<image::LineContent>, base_dir: Option<&std::path::Path>) {
    for line in lines.iter_mut() {
        if let image::LineContent::Image(node) = line {
            if image::is_remote_url(&node.url) {
                match image::download_image(&node.url) {
                    Ok(path) => node.local_path = Some(path),
                    Err(_) => node.download_failed = true,
                }
            } else {
                node.local_path = image::resolve_image_path(&node.url, base_dir);
                if node.local_path.is_none() {
                    node.download_failed = true;
                }
            }
        }
    }
}

fn is_epub(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("epub"))
        .unwrap_or(false)
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: tread <file.md|file.epub|url> [-r|--refresh]");
        eprintln!();
        eprintln!("A terminal Markdown, EPUB & Web reader");
        eprintln!();
        eprintln!("Keys:");
        eprintln!("  j/k/↑/↓     Scroll one line");
        eprintln!("  Ctrl+d/u    Half page down/up");
        eprintln!("  g/G         Top/bottom");
        eprintln!("  Tab         Next image/link");
        eprintln!("  Enter       Open image/link");
        eprintln!("  Ctrl+n/p    Next/prev chapter (EPUB)");
        eprintln!("  t           Table of contents (EPUB)");
        eprintln!("  ?           Help");
        eprintln!("  q           Quit");
        eprintln!();
        eprintln!("Options:");
        eprintln!("  -r, --refresh      Force refresh web page (skip cache)");
        eprintln!("  -i, --interactive  Use headless Chrome for SPA/dynamic pages");
        std::process::exit(1);
    }

    // Parse arguments
    let mut target: Option<String> = None;
    let mut refresh = false;
    let mut interactive = false;

    for arg in &args[1..] {
        match arg.as_str() {
            "-r" | "--refresh" => refresh = true,
            "-i" | "--interactive" => interactive = true,
            _ if target.is_none() => target = Some(arg.clone()),
            _ => {}
        }
    }

    let target = target.unwrap_or_else(|| {
        eprintln!("错误: 未指定文件或 URL");
        std::process::exit(1);
    });

    // Dispatch based on target type
    if web::is_url(&target) {
        run_web(&target, refresh, interactive)
    } else {
        let file_path = PathBuf::from(&target);
        let filename = file_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| file_path.display().to_string());

        if is_epub(&file_path) {
            run_epub(&file_path, &filename)
        } else {
            run_markdown(&file_path, &filename)
        }
    }
}

fn run_web(url: &str, refresh: bool, interactive: bool) -> Result<()> {
    // Try session cache first, then browser cookies
    let domain = web::extract_domain(url);
    let cookies = headless::load_session(&domain)
        .unwrap_or_else(|| headless::get_cookies_for_url(url));

    // Try cache first (unless --refresh)
    let page = if !refresh {
        web::load_web_cache(url)
    } else {
        None
    };

    let page = match page {
        Some(p) => p,
        None => {
            // Fetch HTML
            let html = if interactive {
                // Headless Chrome mode — handles SPA/dynamic content
                eprintln!("正在使用 Headless Chrome 加载页面...");
                headless::headless_fetch(url, &cookies)?
            } else {
                // Standard HTTP fetch with cookies
                let result = headless::fetch_html_with_cookies(url, &cookies);
                match result {
                    Ok(html) => html,
                    Err(_) if !cookies.is_empty() => {
                        // Cookie might be expired, retry without
                        web::fetch_html(url)?
                    }
                    Err(e) => return Err(e),
                }
            };

            let page = web::extract_content(&html, url)
                .map_err(|e| {
                    if interactive {
                        anyhow::anyhow!("无法提取正文")
                    } else {
                        anyhow::anyhow!("无法提取正文，尝试使用 -i 模式: {e}")
                    }
                })?;

            // Save to cache
            let _ = web::save_web_cache(url, &page);

            // Save session if cookies worked
            if !cookies.is_empty() {
                let _ = headless::save_session(&domain, &cookies);
            }

            page
        }
    };

    let display_title = if page.title.is_empty() {
        url.to_string()
    } else {
        page.title.clone()
    };

    // Restore progress
    let progress = web::load_web_progress(url);
    let initial_scroll = progress.as_ref().map(|p| p.scroll).unwrap_or(0);

    // Convert HTML to LineContent
    let lines = web::html_to_lines(&page.content_html, url);

    let mut app = app::App::new(lines, display_title);
    app.scroll_to_line(initial_scroll);

    // Run TUI
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let result = run_app_loop(&mut terminal, &mut app);

    // Save progress
    let _ = web::save_web_progress(url, app.scroll);

    // Teardown
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("错误: {err}");
        std::process::exit(1);
    }

    Ok(())
}

fn run_markdown(file_path: &PathBuf, filename: &str) -> Result<()> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("无法打开文件: {}", file_path.display()))?;

    let initial_line = if env::args().len() > 2 {
        env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(0usize).saturating_sub(1)
    } else {
        0
    };

    let mut content_lines = parser::parse_markdown(&content);

    let base_dir = file_path.parent().map(|p| p.to_path_buf());
    resolve_images(&mut content_lines, base_dir.as_deref());

    let mut app = app::App::new(content_lines, filename.to_string());
    app.scroll_to_line(initial_line);

    run_tui(app)
}

fn run_epub(file_path: &PathBuf, filename: &str) -> Result<()> {
    let book = epub::EpubBook::open(file_path)?;
    let display_title = if book.metadata.title.is_empty() {
        filename.to_string()
    } else {
        book.metadata.title.clone()
    };

    let progress = epub::load_progress(&book.book_hash());
    let initial_chapter = progress.as_ref().map(|p| p.chapter).unwrap_or(0);
    let initial_scroll = progress.as_ref().map(|p| p.scroll).unwrap_or(0);

    let book_hash = book.book_hash();
    let chapter_count = book.chapter_count();
    let toc = book.toc.clone();
    let spine_hrefs: Vec<String> = book.spine.iter().map(|s| s.href.clone()).collect();

    // Load initial chapter with cache support
    let lines = load_chapter_with_cache(&book, initial_chapter)?;

    let mut app = app::App::new(lines, display_title);
    app.set_epub_mode(Some(book_hash.clone()), chapter_count, initial_chapter, toc, spine_hrefs);
    app.scroll_to_line(initial_scroll);

    // Terminal setup
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    // Main loop with chapter switching
    let result = run_epub_loop(&mut terminal, &mut app, &book);

    // Save progress
    let _ = epub::save_progress(&book_hash, &epub::ReadingProgress {
        chapter: app.current_chapter,
        scroll: app.scroll,
    });

    // Teardown
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("错误: {err}");
        std::process::exit(1);
    }

    Ok(())
}

/// Load a chapter, using cache if available.
fn load_chapter_with_cache(book: &epub::EpubBook, chapter_idx: usize) -> Result<Vec<image::LineContent>> {
    let book_hash = book.book_hash();

    // Try cache first
    if let Some(lines) = epub::load_chapter_cache(&book_hash, chapter_idx) {
        return Ok(lines);
    }

    // Cache miss: parse XHTML
    let html = book.read_chapter(chapter_idx)?;
    let image_extractor = |href: &str| -> Option<std::path::PathBuf> {
        book.extract_image(href).ok()
    };
    let lines = xhtml::xhtml_to_lines(&html, Some(&image_extractor));

    // Merge paragraphs
    let merged = epub::merge_paragraphs(lines);

    // Save to cache
    let _ = epub::save_chapter_cache(&book_hash, chapter_idx, &merged);

    Ok(merged)
}

fn run_tui(mut app: app::App) -> Result<()> {
    // Terminal setup
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let result = run_app_loop(&mut terminal, &mut app);

    // Save progress before exit (EPUB only)
    if let Some(ref hash) = app.epub_hash {
        let _ = epub::save_progress(hash, &epub::ReadingProgress {
            chapter: app.current_chapter,
            scroll: app.scroll,
        });
    }

    // Terminal teardown
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("错误: {err}");
        std::process::exit(1);
    }

    Ok(())
}

fn run_epub_loop(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    app: &mut app::App,
    book: &epub::EpubBook,
) -> Result<()> {
    loop {
        terminal.draw(|frame| ui::draw(frame, app))?;

        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            if key.kind != crossterm::event::KeyEventKind::Press {
                continue;
            }
            match app.handle_key(key) {
                app::Action::Continue => {}
                app::Action::Quit => return Ok(()),
                app::Action::LoadChapter(idx, scroll_to_bottom) => {
                    let lines = load_chapter_with_cache(book, idx)?;
                    app.current_chapter = idx;
                    app.replace_content(lines);
                    if scroll_to_bottom {
                        // Jump to bottom of previous chapter
                        let max = app.total_lines().saturating_sub(app.height.unwrap_or(0) as usize);
                        app.scroll_to_line(max);
                    }
                }
            }
        }
    }
}

fn run_app_loop(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    app: &mut app::App,
) -> Result<()> {
    loop {
        terminal.draw(|frame| ui::draw(frame, app))?;

        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            if key.kind != crossterm::event::KeyEventKind::Press {
                continue;
            }
            match app.handle_key(key) {
                app::Action::Continue => {}
                app::Action::Quit => return Ok(()),
                app::Action::LoadChapter(_, _) => {} // Only handled in EPUB loop
            }
        }
    }
}
