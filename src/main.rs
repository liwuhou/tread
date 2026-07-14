use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

mod app;
mod dashboard;
mod epub;
mod headless;
mod history;
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum CommandIntent {
    Dashboard,
    Open(OpenRequest),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OpenRequest {
    target: ReaderTarget,
    refresh: bool,
    interactive: bool,
    initial_position_override: Option<HistoryPosition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ReaderTarget {
    Web(String),
    Epub(PathBuf),
    Markdown(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum HistoryPosition {
    WebScroll { scroll: usize },
    EpubLocation { chapter: usize, scroll: usize },
    MarkdownLine { line: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ReaderSpecificProgress {
    Web(web::WebProgress),
    Epub(epub::ReadingProgress),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReaderRestoreRequest {
    target: ReaderTarget,
    explicit_markdown_line: Option<usize>,
    unified_history_position: Option<HistoryPosition>,
    reader_specific_progress: Option<ReaderSpecificProgress>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RestoreSource {
    ExplicitCliLine,
    ReaderSpecificProgress,
    UnifiedHistory,
    DefaultStart,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RestorePosition {
    Scroll {
        scroll: usize,
        source: RestoreSource,
    },
    EpubLocation {
        chapter: usize,
        scroll: usize,
        source: RestoreSource,
    },
    MarkdownLine {
        line: usize,
        source: RestoreSource,
    },
}

fn parse_command_intent<I, S>(args: I) -> Result<CommandIntent>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut iter = args.into_iter();
    let _program = iter.next();
    let mut target: Option<String> = None;
    let mut refresh = false;
    let mut interactive = false;
    let mut explicit_line: Option<usize> = None;

    for arg in iter {
        let arg = arg.as_ref();
        match arg {
            "-r" | "--refresh" => refresh = true,
            "-i" | "--interactive" => interactive = true,
            _ if target.is_none() => target = Some(arg.to_string()),
            _ if explicit_line.is_none() => match arg.parse() {
                Ok(line) => explicit_line = Some(line),
                Err(_) => anyhow::bail!("无效参数: {arg}"),
            },
            _ => anyhow::bail!("无效参数: {arg}"),
        }
    }

    let Some(target) = target else {
        return Ok(CommandIntent::Dashboard);
    };

    let reader_target = if web::is_url(&target) {
        ReaderTarget::Web(target)
    } else {
        let path = PathBuf::from(target);
        if is_epub(&path) {
            ReaderTarget::Epub(path)
        } else {
            ReaderTarget::Markdown(path)
        }
    };
    let initial_position_override =
        explicit_line.map(|line| HistoryPosition::MarkdownLine { line });

    Ok(CommandIntent::Open(OpenRequest {
        target: reader_target,
        refresh,
        interactive,
        initial_position_override,
    }))
}

fn resolve_reader_restore(request: ReaderRestoreRequest) -> RestorePosition {
    match request.target {
        ReaderTarget::Web(_) => match request.reader_specific_progress {
            Some(ReaderSpecificProgress::Web(progress)) => RestorePosition::Scroll {
                scroll: progress.scroll,
                source: RestoreSource::ReaderSpecificProgress,
            },
            _ => match request.unified_history_position {
                Some(HistoryPosition::WebScroll { scroll }) => RestorePosition::Scroll {
                    scroll,
                    source: RestoreSource::UnifiedHistory,
                },
                _ => RestorePosition::Scroll {
                    scroll: 0,
                    source: RestoreSource::DefaultStart,
                },
            },
        },
        ReaderTarget::Epub(_) => match request.reader_specific_progress {
            Some(ReaderSpecificProgress::Epub(progress)) => RestorePosition::EpubLocation {
                chapter: progress.chapter,
                scroll: progress.scroll,
                source: RestoreSource::ReaderSpecificProgress,
            },
            _ => match request.unified_history_position {
                Some(HistoryPosition::EpubLocation { chapter, scroll }) => {
                    RestorePosition::EpubLocation {
                        chapter,
                        scroll,
                        source: RestoreSource::UnifiedHistory,
                    }
                }
                _ => RestorePosition::EpubLocation {
                    chapter: 0,
                    scroll: 0,
                    source: RestoreSource::DefaultStart,
                },
            },
        },
        ReaderTarget::Markdown(_) => {
            if let Some(line) = request.explicit_markdown_line {
                return RestorePosition::MarkdownLine {
                    line,
                    source: RestoreSource::ExplicitCliLine,
                };
            }
            match request.unified_history_position {
                Some(HistoryPosition::MarkdownLine { line }) => RestorePosition::MarkdownLine {
                    line,
                    source: RestoreSource::UnifiedHistory,
                },
                _ => RestorePosition::MarkdownLine {
                    line: 0,
                    source: RestoreSource::DefaultStart,
                },
            }
        }
    }
}
fn main() -> Result<()> {
    match parse_command_intent(env::args())? {
        CommandIntent::Dashboard => run_dashboard(),
        CommandIntent::Open(request) => run_open_request(request),
    }
}

fn run_open_request(request: OpenRequest) -> Result<()> {
    match request.target {
        ReaderTarget::Web(url) => run_web(
            &url,
            request.refresh,
            request.interactive,
            request.initial_position_override,
        ),
        ReaderTarget::Epub(file_path) => {
            let filename = display_filename(&file_path);
            run_epub(&file_path, &filename, request.initial_position_override)
        }
        ReaderTarget::Markdown(file_path) => {
            let filename = display_filename(&file_path);
            let history_store = history::HistoryStore::default();
            let initial_line = markdown_initial_line(
                &file_path,
                request.initial_position_override,
                &history_store,
            );
            run_markdown(&file_path, &filename, initial_line)
        }
    }
}

fn display_filename(file_path: &std::path::Path) -> String {
    file_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| file_path.display().to_string())
}

fn markdown_initial_line(
    file_path: &std::path::Path,
    initial_position_override: Option<HistoryPosition>,
    store: &history::HistoryStore,
) -> usize {
    match initial_position_override {
        Some(HistoryPosition::MarkdownLine { line }) => line.saturating_sub(1),
        _ => markdown_history_scroll(file_path, store).unwrap_or(0),
    }
}

fn markdown_history_scroll(
    file_path: &std::path::Path,
    store: &history::HistoryStore,
) -> Option<usize> {
    let cwd = std::env::current_dir().ok()?;
    let identity =
        history::TargetIdentity::for_local_path(file_path, history::TargetKind::Markdown, &cwd)
            .ok()?;
    let loaded = store.load().ok()?.history;
    match loaded.entry(&identity.id)?.position {
        history::ReadingPosition::Markdown { scroll } => Some(scroll),
        _ => None,
    }
}

fn progress_percent(position: usize, total: usize) -> i16 {
    if total == 0 || position == 0 {
        return 0;
    }
    ((position.min(total) * 100).div_ceil(total)) as i16
}

fn run_dashboard() -> Result<()> {
    let store = history::HistoryStore::default();
    let loaded = store.load()?;
    if let Some(recovery) = loaded.recovery {
        eprintln!(
            "历史记录已恢复为空，损坏文件已备份到 {}",
            recovery.backup_path.display()
        );
    }

    let history_entries = loaded.history.entries;
    let dashboard_entries: Vec<dashboard::DashboardEntry> = history_entries
        .iter()
        .map(dashboard_entry_from_history)
        .collect();
    let mut state = dashboard::DashboardState::from_history(dashboard_entries);
    let mut prompt = String::new();
    let mut prompt_open = false;
    let mut error: Option<String> = None;
    let mut suggestions: Vec<dashboard::PathSuggestion> = Vec::new();
    let mut selected_suggestion: usize = 0;
    let mut selected_request: Option<OpenRequest> = None;

    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::cursor::Hide
    )?;

    loop {
        render_dashboard(
            &mut stdout,
            &state,
            prompt_open.then_some(prompt.as_str()),
            if prompt_open {
                Some(suggestions.as_slice())
            } else {
                None
            },
            selected_suggestion,
            error.as_deref(),
        )?;
        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            if key.code == crossterm::event::KeyCode::Char('c')
                && key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
            {
                break;
            }
            if prompt_open {
                match key.code {
                    crossterm::event::KeyCode::Esc => {
                        prompt_open = false;
                        prompt.clear();
                        suggestions.clear();
                    }
                    crossterm::event::KeyCode::Enter => {
                        if complete_selected_suggestion(
                            &mut prompt,
                            &suggestions,
                            selected_suggestion,
                        ) {
                            suggestions = refresh_suggestions(&prompt);
                            selected_suggestion = 0;
                        } else {
                            match open_request_from_prompt(&prompt) {
                                Ok(request) => {
                                    selected_request = Some(request);
                                    break;
                                }
                                Err(err) => error = Some(err),
                            }
                        }
                    }
                    crossterm::event::KeyCode::Tab => {
                        complete_selected_suggestion(
                            &mut prompt,
                            &suggestions,
                            selected_suggestion,
                        );
                        suggestions = refresh_suggestions(&prompt);
                        selected_suggestion = 0;
                    }
                    crossterm::event::KeyCode::Down => {
                        if !suggestions.is_empty() {
                            selected_suggestion =
                                (selected_suggestion + 1).min(suggestions.len() - 1);
                        }
                    }
                    crossterm::event::KeyCode::Up => {
                        selected_suggestion = selected_suggestion.saturating_sub(1);
                    }
                    crossterm::event::KeyCode::Backspace => {
                        prompt.pop();
                        suggestions = refresh_suggestions(&prompt);
                        selected_suggestion = 0;
                        error = None;
                    }
                    crossterm::event::KeyCode::Char(ch) => {
                        prompt.push(ch);
                        suggestions = refresh_suggestions(&prompt);
                        selected_suggestion = 0;
                        error = None;
                    }
                    _ => {}
                }
                continue;
            }

            match key.code {
                crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => break,
                crossterm::event::KeyCode::Char('o') => {
                    prompt_open = true;
                    prompt.clear();
                    error = None;
                    suggestions.clear();
                    selected_suggestion = 0;
                }
                crossterm::event::KeyCode::Enter => {
                    if let Some(id) = state.selected_entry_id() {
                        if let Some(entry) = history_entries.iter().find(|entry| entry.id == id) {
                            selected_request = open_request_from_history(entry);
                            if selected_request.is_some() {
                                break;
                            }
                        }
                    }
                }
                crossterm::event::KeyCode::Char('s') => {
                    let selected = state.selected_entry_id().map(str::to_string);
                    state.handle_key(key);
                    if let Some(id) = selected {
                        if let Some(entry) = state.entry(&id) {
                            let mut loaded = store.load()?.history;
                            loaded.set_starred(&id, entry.starred, &history::now_timestamp())?;
                            store.save(&loaded)?;
                        }
                    }
                }
                crossterm::event::KeyCode::Char('r') => {
                    let selected = state.selected_entry_id().map(str::to_string);
                    state.handle_key(key);
                    if let Some(id) = selected {
                        persist_dashboard_remove(&store, &id)?;
                    }
                }
                _ => state.handle_key(key),
            }
        }
    }

    crossterm::execute!(
        stdout,
        crossterm::cursor::Show,
        crossterm::terminal::LeaveAlternateScreen
    )?;
    crossterm::terminal::disable_raw_mode()?;

    if let Some(request) = selected_request {
        run_open_request(request)
    } else {
        Ok(())
    }
}

fn dashboard_entry_from_history(entry: &history::HistoryEntry) -> dashboard::DashboardEntry {
    dashboard::DashboardEntry {
        id: entry.id.clone(),
        kind: match entry.kind {
            history::TargetKind::Web => dashboard::TargetKind::Web,
            history::TargetKind::Epub => dashboard::TargetKind::Epub,
            history::TargetKind::Markdown => dashboard::TargetKind::Markdown,
        },
        title: entry.title.clone(),
        target: entry.target.clone(),
        progress_percent: entry.progress_percent,
        updated_at: entry.updated_at.clone(),
        starred: entry.starred,
        hidden: entry.hidden,
    }
}

fn open_request_from_history(entry: &history::HistoryEntry) -> Option<OpenRequest> {
    let target = match entry.kind {
        history::TargetKind::Web => ReaderTarget::Web(entry.target.clone()),
        history::TargetKind::Epub => ReaderTarget::Epub(PathBuf::from(&entry.target)),
        history::TargetKind::Markdown => ReaderTarget::Markdown(PathBuf::from(&entry.target)),
    };
    let initial_position_override = match entry.position {
        history::ReadingPosition::Web { scroll } => Some(HistoryPosition::WebScroll { scroll }),
        history::ReadingPosition::Epub { chapter, scroll } => {
            Some(HistoryPosition::EpubLocation { chapter, scroll })
        }
        history::ReadingPosition::Markdown { scroll } => Some(HistoryPosition::MarkdownLine {
            line: scroll.saturating_add(1),
        }),
    };
    Some(OpenRequest {
        target,
        refresh: false,
        interactive: false,
        initial_position_override,
    })
}

fn persist_dashboard_remove(store: &history::HistoryStore, id: &str) -> Result<()> {
    let mut loaded = store.load()?.history;
    loaded.set_hidden(id, true, &history::now_timestamp())?;
    loaded.set_starred(id, false, &history::now_timestamp())?;
    store.save(&loaded)
}

struct DashboardPromptArgs {
    target: String,
    refresh: bool,
    interactive: bool,
}

fn parse_dashboard_prompt_args(input: &str) -> std::result::Result<DashboardPromptArgs, String> {
    let mut target: Option<String> = None;
    let mut refresh = false;
    let mut interactive = false;

    for token in input.split_whitespace() {
        match token {
            "-r" | "--refresh" => refresh = true,
            "-i" | "--interactive" => interactive = true,
            _ if token.starts_with('-') => return Err(format!("未知参数: {token}")),
            _ if target.is_none() => target = Some(token.to_string()),
            _ => return Err(format!("只能输入一个路径或 URL: {token}")),
        }
    }

    let Some(target) = target else {
        return Err("请输入路径或 URL".to_string());
    };

    Ok(DashboardPromptArgs {
        target,
        refresh,
        interactive,
    })
}

fn open_request_from_prompt(input: &str) -> std::result::Result<OpenRequest, String> {
    let args = parse_dashboard_prompt_args(input.trim())?;
    match dashboard::parse_open_prompt(&args.target) {
        dashboard::PromptTarget::Url(url) => Ok(OpenRequest {
            target: ReaderTarget::Web(url),
            refresh: args.refresh,
            interactive: args.interactive,
            initial_position_override: None,
        }),
        dashboard::PromptTarget::LocalPath(path) => {
            if args.interactive {
                return Err("-i 只适用于 URL".to_string());
            }
            if !path.exists() {
                return Err(format!("路径不存在: {}", path.display()));
            }
            if path.is_dir() {
                return Err(format!("不是文件: {}", path.display()));
            }
            if let Err(err) = fs::File::open(&path) {
                return Err(format!("无法打开: {} ({err})", path.display()));
            }
            let target = if is_epub(&path) {
                ReaderTarget::Epub(path)
            } else {
                ReaderTarget::Markdown(path)
            };
            Ok(OpenRequest {
                target,
                refresh: args.refresh,
                interactive: false,
                initial_position_override: None,
            })
        }
    }
}

fn refresh_suggestions(prompt: &str) -> Vec<dashboard::PathSuggestion> {
    if prompt.trim().is_empty() || prompt.starts_with("http://") || prompt.starts_with("https://") {
        Vec::new()
    } else {
        dashboard::suggest_local_paths(prompt).unwrap_or_default()
    }
}

fn complete_selected_suggestion(
    prompt: &mut String,
    suggestions: &[dashboard::PathSuggestion],
    selected: usize,
) -> bool {
    let Some(suggestion) = suggestions.get(selected) else {
        return false;
    };
    *prompt = suggestion.path.to_string_lossy().to_string();
    if suggestion.is_dir && !prompt.ends_with(std::path::MAIN_SEPARATOR) {
        prompt.push(std::path::MAIN_SEPARATOR);
    }
    suggestion.is_dir
}

struct DashboardVisualLine {
    text: String,
    selected: bool,
    dim: bool,
}

struct DashboardRenderPlan {
    lines: Vec<DashboardVisualLine>,
    prompt_cursor: Option<(u16, u16)>,
}

fn dashboard_entry_lines(
    entry: &dashboard::DashboardEntry,
    index: usize,
    is_selected: bool,
    width: usize,
) -> (DashboardVisualLine, DashboardVisualLine) {
    let index = format!("{:02}", index + 1);
    let kind = dashboard_kind_label(entry.kind);
    let progress = format!("{:>3}%", entry.progress_percent);
    let star = if entry.starred { "★" } else { " " };
    let prefix = format!("  {star}{index}  {kind:<4}  ");
    let suffix = format!("  {progress}");
    let title_width = width.saturating_sub(display_width(&prefix) + display_width(&suffix));
    let title = fit_width(&entry.title, title_width);
    (
        DashboardVisualLine {
            text: pad_to_width(&format!("{prefix}{title}{suffix}"), width),
            selected: is_selected,
            dim: false,
        },
        DashboardVisualLine {
            text: format!(
                "        {}",
                fit_width(&entry.target, width.saturating_sub(8))
            ),
            selected: false,
            dim: true,
        },
    )
}

fn dashboard_visual_lines(
    state: &dashboard::DashboardState,
    width: usize,
) -> Vec<DashboardVisualLine> {
    let width = width.max(40);
    let rows = state.visible_rows();
    let selected = state.selected_entry_id().map(str::to_string);
    let mut lines = vec![
        DashboardVisualLine {
            text: "tread".to_string(),
            selected: false,
            dim: false,
        },
        DashboardVisualLine {
            text: "─".repeat(width),
            selected: false,
            dim: true,
        },
        DashboardVisualLine {
            text: align_between("最近阅读", &entry_count_label(rows.len()), width),
            selected: false,
            dim: false,
        },
        DashboardVisualLine {
            text: String::new(),
            selected: false,
            dim: false,
        },
    ];

    if rows.is_empty() {
        lines.push(DashboardVisualLine {
            text: fit_width(
                "按 o 打开 Markdown、EPUB 或 URL。阅读后会出现在这里。",
                width,
            ),
            selected: false,
            dim: true,
        });
    } else {
        for (idx, row) in rows.iter().enumerate() {
            let Some(entry) = state.entry(&row.entry_id) else {
                continue;
            };
            let is_selected = selected.as_deref() == Some(entry.id.as_str());
            let (title_line, target_line) = dashboard_entry_lines(entry, idx, is_selected, width);
            lines.push(title_line);
            lines.push(target_line);
        }
    }

    lines.push(DashboardVisualLine {
        text: String::new(),
        selected: false,
        dim: false,
    });
    lines.push(DashboardVisualLine {
        text: "─".repeat(width),
        selected: false,
        dim: true,
    });
    lines.push(DashboardVisualLine {
        text: fit_width(
            "Enter open   j/k move   o new URL -i   s star   r remove   q quit",
            width,
        ),
        selected: false,
        dim: true,
    });
    lines
}

fn dashboard_compact_recent_lines(
    state: &dashboard::DashboardState,
    width: usize,
) -> Vec<DashboardVisualLine> {
    let width = width.max(40);
    let rows = state.visible_rows();
    let selected = state.selected_entry_id().map(str::to_string);
    let section_line = DashboardVisualLine {
        text: align_between("最近阅读", &entry_count_label(rows.len()), width),
        selected: false,
        dim: false,
    };

    if rows.is_empty() {
        return vec![
            section_line,
            DashboardVisualLine {
                text: fit_width("按 o 打开 Markdown、EPUB 或 URL。", width),
                selected: false,
                dim: true,
            },
            DashboardVisualLine {
                text: fit_width("也可以直接运行 tread <file.md|file.epub|url>。", width),
                selected: false,
                dim: true,
            },
        ];
    }

    let selected_idx = rows
        .iter()
        .position(|row| selected.as_deref() == Some(row.entry_id.as_str()))
        .unwrap_or(0);
    let row = &rows[selected_idx];
    let entry = state
        .entry(&row.entry_id)
        .expect("visible row should resolve to dashboard entry");
    let (title_line, target_line) = dashboard_entry_lines(entry, selected_idx, true, width);

    vec![section_line, title_line, target_line]
}

fn dashboard_aux_lines(
    width: usize,
    prompt: Option<&str>,
    suggestions: Option<&[dashboard::PathSuggestion]>,
    selected_suggestion: usize,
    error: Option<&str>,
    max_lines: usize,
) -> (Vec<DashboardVisualLine>, Option<(u16, u16)>) {
    if max_lines == 0 {
        return (Vec::new(), None);
    }

    let mut lines = Vec::new();
    let mut prompt_cursor = None;

    if let Some(prompt) = prompt {
        let prompt_text = fit_width(&format!("打开: {prompt}"), width);
        lines.push(DashboardVisualLine {
            text: prompt_text.clone(),
            selected: false,
            dim: false,
        });
        prompt_cursor = Some((display_width(&prompt_text) as u16, 0));

        if let Some(suggestions) = suggestions {
            for (idx, suggestion) in suggestions.iter().take(8).enumerate() {
                if lines.len() >= max_lines {
                    break;
                }
                let cursor = if idx == selected_suggestion {
                    "›"
                } else {
                    " "
                };
                lines.push(DashboardVisualLine {
                    text: fit_width(&format!("  {cursor} {}", suggestion.display), width),
                    selected: false,
                    dim: true,
                });
            }
        }
    }

    if let Some(error) = error {
        if lines.len() < max_lines {
            lines.push(DashboardVisualLine {
                text: fit_width(&format!("错误: {error}"), width),
                selected: false,
                dim: false,
            });
        }
    }

    lines.truncate(max_lines);
    (lines, prompt_cursor)
}

fn dashboard_render_plan(
    state: &dashboard::DashboardState,
    width: usize,
    height: u16,
    prompt: Option<&str>,
    suggestions: Option<&[dashboard::PathSuggestion]>,
    selected_suggestion: usize,
    error: Option<&str>,
) -> DashboardRenderPlan {
    let width = width.max(40);
    let height = height as usize;

    if height < 3 {
        return DashboardRenderPlan {
            lines: vec![DashboardVisualLine {
                text: fit_width("终端高度太小，请调高后再查看 Dashboard。", width),
                selected: false,
                dim: true,
            }],
            prompt_cursor: None,
        };
    }

    let full_lines = dashboard_visual_lines(state, width);
    let (mut full_aux, full_prompt_cursor) = dashboard_aux_lines(
        width,
        prompt,
        suggestions,
        selected_suggestion,
        error,
        usize::MAX,
    );

    if full_lines.len() + full_aux.len() <= height {
        let prompt_cursor =
            full_prompt_cursor.map(|(col, row)| (col, row + full_lines.len() as u16));
        let mut lines = full_lines;
        lines.append(&mut full_aux);
        return DashboardRenderPlan {
            lines,
            prompt_cursor,
        };
    }

    let mut lines = Vec::new();
    if prompt.is_none() && error.is_none() && height >= 5 {
        lines.push(DashboardVisualLine {
            text: "tread".to_string(),
            selected: false,
            dim: false,
        });
        lines.push(DashboardVisualLine {
            text: "─".repeat(width),
            selected: false,
            dim: true,
        });
    }

    lines.extend(dashboard_compact_recent_lines(state, width));
    lines.truncate(height);

    let remaining = height.saturating_sub(lines.len());
    let (mut aux_lines, prompt_cursor) = dashboard_aux_lines(
        width,
        prompt,
        suggestions,
        selected_suggestion,
        error,
        remaining,
    );
    let prompt_cursor = prompt_cursor.map(|(col, row)| (col, row + lines.len() as u16));
    lines.append(&mut aux_lines);

    DashboardRenderPlan {
        lines,
        prompt_cursor,
    }
}

fn dashboard_kind_label(kind: dashboard::TargetKind) -> &'static str {
    match kind {
        dashboard::TargetKind::Web => "WEB",
        dashboard::TargetKind::Epub => "EPUB",
        dashboard::TargetKind::Markdown => "MD",
    }
}

fn entry_count_label(count: usize) -> String {
    if count == 1 {
        "1 item".to_string()
    } else {
        format!("{count} items")
    }
}

fn align_between(left: &str, right: &str, width: usize) -> String {
    let left_width = display_width(left);
    let right_width = display_width(right);
    if left_width + 1 + right_width >= width {
        return left.to_string();
    }
    format!(
        "{}{}{}",
        left,
        " ".repeat(width - left_width - right_width),
        right
    )
}

fn display_width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

fn fit_width(text: &str, width: usize) -> String {
    if display_width(text) <= width {
        return text.to_string();
    }
    if width == 0 {
        return String::new();
    }
    let ellipsis_width = display_width("…");
    if width <= ellipsis_width {
        return "…".to_string();
    }
    let mut result = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let char_width = ch.width().unwrap_or(0);
        if used + char_width + ellipsis_width > width {
            break;
        }
        result.push(ch);
        used += char_width;
    }
    result.push('…');
    result
}

fn pad_to_width(text: &str, width: usize) -> String {
    let used = display_width(text);
    if used >= width {
        text.to_string()
    } else {
        format!("{}{}", text, " ".repeat(width - used))
    }
}

fn render_dashboard(
    stdout: &mut std::io::Stdout,
    state: &dashboard::DashboardState,
    prompt: Option<&str>,
    suggestions: Option<&[dashboard::PathSuggestion]>,
    selected_suggestion: usize,
    error: Option<&str>,
) -> Result<()> {
    crossterm::execute!(
        stdout,
        crossterm::cursor::MoveTo(0, 0),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
    )?;
    let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
    let plan = dashboard_render_plan(
        state,
        (width as usize).clamp(48, 120),
        height,
        prompt,
        suggestions,
        selected_suggestion,
        error,
    );

    if plan.prompt_cursor.is_some() {
        crossterm::execute!(stdout, crossterm::cursor::Show)?;
    } else {
        crossterm::execute!(stdout, crossterm::cursor::Hide)?;
    }

    for line in &plan.lines {
        if line.selected {
            write!(stdout, "\x1b[7m{}\x1b[0m\r\n", line.text)?;
        } else if line.dim {
            write!(stdout, "\x1b[2m{}\x1b[0m\r\n", line.text)?;
        } else {
            write!(stdout, "{}\r\n", line.text)?;
        }
    }

    if let Some((col, row)) = plan.prompt_cursor {
        crossterm::execute!(stdout, crossterm::cursor::MoveTo(col, row))?;
    }
    stdout.flush()?;
    Ok(())
}

fn run_web(
    url: &str,
    refresh: bool,
    interactive: bool,
    initial_position_override: Option<HistoryPosition>,
) -> Result<()> {
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
                // Headless Chrome mode — read fresh cookies from browser
                eprintln!("正在使用 Headless Chrome 加载页面...");
                let cookies = headless::get_cookies_for_url(url);
                headless::headless_fetch(url, &cookies)?
            } else {
                // Standard HTTP fetch with cookies
                // Try session cache first, then browser cookies
                let domain = web::extract_domain(url);
                let cookies = headless::load_session(&domain).unwrap_or_else(|| {
                    headless::cookies_to_name_value(&headless::get_cookies_for_url(url))
                });

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

            let page = web::extract_content(&html, url).map_err(|e| {
                if interactive {
                    anyhow::anyhow!("无法提取正文")
                } else {
                    anyhow::anyhow!("无法提取正文，尝试使用 -i 模式: {e}")
                }
            })?;

            // Save to cache
            let _ = web::save_web_cache(url, &page);

            // Save session if cookies worked (only for non-interactive mode)
            if !interactive {
                let domain = web::extract_domain(url);
                let cookies = headless::cookies_to_name_value(&headless::get_cookies_for_url(url));
                if !cookies.is_empty() {
                    let _ = headless::save_session(&domain, &cookies);
                }
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
    let initial_scroll = match initial_position_override {
        Some(HistoryPosition::WebScroll { scroll }) => scroll,
        _ => progress.as_ref().map(|p| p.scroll).unwrap_or(0),
    };
    let history_store = history::HistoryStore::default();
    let history_session = history_store
        .upsert_open(
            history::HistoryTarget::Web {
                url: url.to_string(),
                title: display_title.clone(),
            },
            history::ReadingPosition::Web {
                scroll: initial_scroll,
            },
            0,
            &history::now_timestamp(),
        )
        .ok();

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
    if let Some(session) = history_session {
        let _ = history_store.update_progress(
            &session,
            history::ReadingPosition::Web { scroll: app.scroll },
            progress_percent(app.scroll, app.lines.len()),
            &history::now_timestamp(),
        );
    }

    // Teardown
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    let exit_action = match result {
        Ok(action) => action,
        Err(err) => {
            eprintln!("错误: {err}");
            std::process::exit(1);
        }
    };

    maybe_open_dashboard(exit_action)
}

fn run_markdown(file_path: &PathBuf, filename: &str, initial_line: usize) -> Result<()> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("无法打开文件: {}", file_path.display()))?;
    let mut content_lines = parser::parse_markdown(&content);

    let base_dir = file_path.parent().map(|p| p.to_path_buf());
    resolve_images(&mut content_lines, base_dir.as_deref());

    let history_store = history::HistoryStore::default();
    let history_session = history_store
        .upsert_open(
            history::HistoryTarget::Markdown {
                path: file_path.clone(),
                title: filename.to_string(),
            },
            history::ReadingPosition::Markdown {
                scroll: initial_line,
            },
            0,
            &history::now_timestamp(),
        )
        .ok();

    let mut app = app::App::new(content_lines, filename.to_string());
    app.scroll_to_line(initial_line);

    let (app, exit_action) = run_tui(app)?;
    if let Some(session) = history_session {
        let _ = history_store.update_progress(
            &session,
            history::ReadingPosition::Markdown { scroll: app.scroll },
            progress_percent(app.scroll, app.lines.len()),
            &history::now_timestamp(),
        );
    }
    maybe_open_dashboard(exit_action)
}

fn run_epub(
    file_path: &PathBuf,
    filename: &str,
    initial_position_override: Option<HistoryPosition>,
) -> Result<()> {
    let book = epub::EpubBook::open(file_path)?;
    let display_title = if book.metadata.title.is_empty() {
        filename.to_string()
    } else {
        book.metadata.title.clone()
    };

    let progress = epub::load_progress(&book.book_hash());
    let (initial_chapter, initial_scroll) = match initial_position_override {
        Some(HistoryPosition::EpubLocation { chapter, scroll }) => (chapter, scroll),
        _ => progress
            .as_ref()
            .map(|p| (p.chapter, p.scroll))
            .unwrap_or((0, 0)),
    };
    let history_store = history::HistoryStore::default();
    let history_session = history_store
        .upsert_open(
            history::HistoryTarget::Epub {
                path: file_path.clone(),
                title: display_title.clone(),
            },
            history::ReadingPosition::Epub {
                chapter: initial_chapter,
                scroll: initial_scroll,
            },
            0,
            &history::now_timestamp(),
        )
        .ok();

    let book_hash = book.book_hash();
    let chapter_count = book.chapter_count();
    let toc = book.toc.clone();
    let spine_hrefs: Vec<String> = book.spine.iter().map(|s| s.href.clone()).collect();

    // Load initial chapter with cache support
    let lines = load_chapter_with_cache(&book, initial_chapter)?;

    let mut app = app::App::new(lines, display_title);
    app.set_epub_mode(
        Some(book_hash.clone()),
        chapter_count,
        initial_chapter,
        toc,
        spine_hrefs,
    );
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
    let _ = epub::save_progress(
        &book_hash,
        &epub::ReadingProgress {
            chapter: app.current_chapter,
            scroll: app.scroll,
        },
    );
    if let Some(session) = history_session {
        let _ = history_store.update_progress(
            &session,
            history::ReadingPosition::Epub {
                chapter: app.current_chapter,
                scroll: app.scroll,
            },
            progress_percent(app.current_chapter.saturating_add(1), chapter_count),
            &history::now_timestamp(),
        );
    }

    // Teardown
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    let exit_action = match result {
        Ok(action) => action,
        Err(err) => {
            eprintln!("错误: {err}");
            std::process::exit(1);
        }
    };

    maybe_open_dashboard(exit_action)
}

/// Load a chapter, using cache if available.
fn load_chapter_with_cache(
    book: &epub::EpubBook,
    chapter_idx: usize,
) -> Result<Vec<image::LineContent>> {
    let book_hash = book.book_hash();

    // Try cache first
    if let Some(lines) = epub::load_chapter_cache(&book_hash, chapter_idx) {
        return Ok(lines);
    }

    // Cache miss: parse XHTML
    let html = book.read_chapter(chapter_idx)?;
    let image_extractor =
        |href: &str| -> Option<std::path::PathBuf> { book.extract_image(href).ok() };
    let lines = xhtml::xhtml_to_lines(&html, Some(&image_extractor));

    // Merge paragraphs
    let merged = epub::merge_paragraphs(lines);

    // Save to cache
    let _ = epub::save_chapter_cache(&book_hash, chapter_idx, &merged);

    Ok(merged)
}

fn run_tui(mut app: app::App) -> Result<(app::App, app::Action)> {
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
        let _ = epub::save_progress(
            hash,
            &epub::ReadingProgress {
                chapter: app.current_chapter,
                scroll: app.scroll,
            },
        );
    }

    // Terminal teardown
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    let exit_action = match result {
        Ok(action) => action,
        Err(err) => {
            eprintln!("错误: {err}");
            std::process::exit(1);
        }
    };

    Ok((app, exit_action))
}

fn maybe_open_dashboard(exit_action: app::Action) -> Result<()> {
    if matches!(exit_action, app::Action::Dashboard) {
        run_dashboard()?;
    }
    Ok(())
}

fn run_epub_loop(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    app: &mut app::App,
    book: &epub::EpubBook,
) -> Result<app::Action> {
    loop {
        terminal.draw(|frame| ui::draw(frame, app))?;

        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            if key.kind != crossterm::event::KeyEventKind::Press {
                continue;
            }
            match app.handle_key(key) {
                app::Action::Continue => {}
                app::Action::Quit => return Ok(app::Action::Quit),
                app::Action::Dashboard => return Ok(app::Action::Dashboard),
                app::Action::LoadChapter(idx, scroll_to_bottom) => {
                    let lines = load_chapter_with_cache(book, idx)?;
                    app.current_chapter = idx;
                    app.replace_content(lines);
                    if scroll_to_bottom {
                        // Jump to bottom of previous chapter
                        let max = app
                            .total_lines()
                            .saturating_sub(app.height.unwrap_or(0) as usize);
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
) -> Result<app::Action> {
    loop {
        terminal.draw(|frame| ui::draw(frame, app))?;

        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            if key.kind != crossterm::event::KeyEventKind::Press {
                continue;
            }
            match app.handle_key(key) {
                app::Action::Continue => {}
                app::Action::Quit => return Ok(app::Action::Quit),
                app::Action::Dashboard => return Ok(app::Action::Dashboard),
                app::Action::LoadChapter(_, _) => {} // Only handled in EPUB loop
            }
        }
    }
}

#[cfg(test)]
mod cli_intent_tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static CTR: AtomicUsize = AtomicUsize::new(0);

    fn temp_dir(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "tread_main_test_{}_{}_{}",
            name,
            std::process::id(),
            CTR.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn history_path(root: &Path) -> PathBuf {
        root.join(".tread").join("history.json")
    }

    #[test]
    fn no_args_parse_to_dashboard_intent() {
        let intent = parse_command_intent(["tread"]).expect("bare tread should parse");

        assert_eq!(intent, CommandIntent::Dashboard);
    }

    #[test]
    fn url_direct_open_preserves_refresh_and_interactive_flags() {
        let intent = parse_command_intent([
            "tread",
            "https://example.test/articles/intro",
            "--refresh",
            "--interactive",
        ])
        .expect("url with web flags should parse");

        let CommandIntent::Open(request) = intent else {
            panic!("expected direct open intent for URL");
        };

        assert_eq!(
            request.target,
            ReaderTarget::Web("https://example.test/articles/intro".to_string())
        );
        assert!(request.refresh, "explicit --refresh must reach web reader");
        assert!(
            request.interactive,
            "explicit --interactive must reach web reader"
        );
    }

    #[test]
    fn epub_path_direct_open_is_open_intent() {
        let intent =
            parse_command_intent(["tread", "books/novel.epub"]).expect("epub path should parse");

        let CommandIntent::Open(request) = intent else {
            panic!("expected direct open intent for EPUB path");
        };

        assert_eq!(
            request.target,
            ReaderTarget::Epub(PathBuf::from("books/novel.epub"))
        );
    }

    #[test]
    fn invalid_extra_argument_is_rejected() {
        let err = parse_command_intent(["tread", "notes.md", "--refesh"]).unwrap_err();

        assert!(err.to_string().contains("无效参数: --refesh"));
    }

    #[test]
    fn markdown_explicit_line_override_wins_over_saved_history() {
        let path = PathBuf::from("/tmp/tread-notes.md");
        let restore = resolve_reader_restore(ReaderRestoreRequest {
            target: ReaderTarget::Markdown(path.clone()),
            explicit_markdown_line: Some(50),
            unified_history_position: Some(HistoryPosition::MarkdownLine { line: 12 }),
            reader_specific_progress: None,
        });

        assert_eq!(
            restore,
            RestorePosition::MarkdownLine {
                line: 50,
                source: RestoreSource::ExplicitCliLine
            }
        );
    }

    #[test]
    fn web_restore_uses_reader_specific_progress_before_unified_history() {
        let restore = resolve_reader_restore(ReaderRestoreRequest {
            target: ReaderTarget::Web("https://example.test/articles/intro".to_string()),
            explicit_markdown_line: None,
            unified_history_position: Some(HistoryPosition::WebScroll { scroll: 8 }),
            reader_specific_progress: Some(ReaderSpecificProgress::Web(web::WebProgress {
                url: "https://example.test/articles/intro".to_string(),
                scroll: 42,
                saved_at: "2026-07-09T00:00:00Z".to_string(),
            })),
        });

        assert_eq!(
            restore,
            RestorePosition::Scroll {
                scroll: 42,
                source: RestoreSource::ReaderSpecificProgress
            }
        );
    }

    #[test]
    fn epub_restore_uses_reader_specific_progress_before_unified_history() {
        let restore = resolve_reader_restore(ReaderRestoreRequest {
            target: ReaderTarget::Epub(PathBuf::from("/tmp/tread-book.epub")),
            explicit_markdown_line: None,
            unified_history_position: Some(HistoryPosition::EpubLocation {
                chapter: 1,
                scroll: 9,
            }),
            reader_specific_progress: Some(ReaderSpecificProgress::Epub(epub::ReadingProgress {
                chapter: 3,
                scroll: 77,
            })),
        });

        assert_eq!(
            restore,
            RestorePosition::EpubLocation {
                chapter: 3,
                scroll: 77,
                source: RestoreSource::ReaderSpecificProgress
            }
        );
    }

    #[test]
    fn dashboard_history_open_preserves_epub_position_override() {
        let entry = history::HistoryEntry {
            id: "epub:book".to_string(),
            kind: history::TargetKind::Epub,
            title: "Book".to_string(),
            target: "/tmp/book.epub".to_string(),
            position: history::ReadingPosition::Epub {
                chapter: 22,
                scroll: 7,
            },
            progress_percent: 70,
            updated_at: "2026-07-09T00:00:00Z".to_string(),
            starred: false,
            hidden: false,
        };

        let request = open_request_from_history(&entry).expect("history entry should open");

        assert_eq!(
            request.target,
            ReaderTarget::Epub(PathBuf::from("/tmp/book.epub"))
        );
        assert_eq!(
            request.initial_position_override,
            Some(HistoryPosition::EpubLocation {
                chapter: 22,
                scroll: 7,
            })
        );
    }

    #[test]
    fn dashboard_prompt_url_accepts_interactive_and_refresh_flags() {
        let request = open_request_from_prompt("https://example.com/app -i --refresh").unwrap();

        assert_eq!(
            request.target,
            ReaderTarget::Web("https://example.com/app".to_string())
        );
        assert!(request.interactive);
        assert!(request.refresh);
        assert_eq!(request.initial_position_override, None);
    }

    #[test]
    fn dashboard_prompt_url_accepts_flags_before_target() {
        let request = open_request_from_prompt("--interactive https://example.com/app").unwrap();

        assert_eq!(
            request.target,
            ReaderTarget::Web("https://example.com/app".to_string())
        );
        assert!(request.interactive);
        assert!(!request.refresh);
    }

    #[test]
    fn dashboard_prompt_rejects_interactive_for_local_files() {
        let root = temp_dir("prompt_local_interactive");
        let file = root.join("notes.md");
        fs::write(&file, "# Notes\n").unwrap();

        let err = open_request_from_prompt(&format!("{} -i", file.display())).unwrap_err();

        assert!(err.contains("-i 只适用于 URL"));
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn markdown_without_line_override_restores_from_unified_history() {
        let restore = resolve_reader_restore(ReaderRestoreRequest {
            target: ReaderTarget::Markdown(PathBuf::from("/tmp/tread-notes.md")),
            explicit_markdown_line: None,
            unified_history_position: Some(HistoryPosition::MarkdownLine { line: 34 }),
            reader_specific_progress: None,
        });

        assert_eq!(
            restore,
            RestorePosition::MarkdownLine {
                line: 34,
                source: RestoreSource::UnifiedHistory
            }
        );
    }

    #[test]
    fn markdown_direct_open_uses_saved_history_scroll_unless_cli_line_is_explicit() {
        let root = temp_dir("markdown_restore");
        let file = root.join("notes.md");
        fs::write(&file, "# Notes\n\nSaved position").unwrap();
        let store = history::HistoryStore::at(history_path(&root));
        let identity =
            history::TargetIdentity::for_local_path(&file, history::TargetKind::Markdown, &root)
                .unwrap();
        store
            .save(&history::HistoryFile {
                entries: vec![history::HistoryEntry {
                    id: identity.id,
                    kind: history::TargetKind::Markdown,
                    title: "Notes".to_string(),
                    target: identity.identity_source,
                    position: history::ReadingPosition::Markdown { scroll: 41 },
                    progress_percent: 40,
                    updated_at: "2026-07-09T00:00:00Z".to_string(),
                    starred: false,
                    hidden: false,
                }],
            })
            .unwrap();

        assert_eq!(markdown_initial_line(&file, None, &store), 41);
        assert_eq!(
            markdown_initial_line(
                &file,
                Some(HistoryPosition::MarkdownLine { line: 7 }),
                &store,
            ),
            6
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn dashboard_remove_persistence_hides_entry_and_clears_starred() {
        let root = temp_dir("dashboard_remove_persist");
        let store = history::HistoryStore::at(history_path(&root));
        let id = history::TargetIdentity::for_web_url("https://example.test/favorite").id;
        store
            .save(&history::HistoryFile {
                entries: vec![history::HistoryEntry {
                    id: id.clone(),
                    kind: history::TargetKind::Web,
                    title: "Favorite".to_string(),
                    target: "https://example.test/favorite".to_string(),
                    position: history::ReadingPosition::Web { scroll: 12 },
                    progress_percent: 20,
                    updated_at: "2026-07-09T00:00:00Z".to_string(),
                    starred: true,
                    hidden: false,
                }],
            })
            .unwrap();

        persist_dashboard_remove(&store, &id).unwrap();

        let loaded = store.load().unwrap().history;
        let entry = loaded.entry(&id).unwrap();
        assert!(entry.hidden);
        assert!(!entry.starred);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn dashboard_prompt_directory_returns_error_instead_of_markdown_open_request() {
        let root = temp_dir("prompt_directory");

        let err = open_request_from_prompt(&root.to_string_lossy()).unwrap_err();

        assert!(err.contains("不是文件"));
        assert!(err.contains(&root.to_string_lossy().to_string()));

        fs::remove_dir_all(root).ok();
    }

    fn sample_dashboard_state() -> dashboard::DashboardState {
        dashboard::DashboardState::from_history(vec![dashboard::DashboardEntry {
            id: "epub:book".to_string(),
            kind: dashboard::TargetKind::Epub,
            title: "境界-吴军".to_string(),
            target: "/Users/awu/Data/frontend/github/tread/test.epub".to_string(),
            progress_percent: 1,
            updated_at: "2026-07-09T00:00:00Z".to_string(),
            starred: false,
            hidden: false,
        }])
    }

    #[test]
    fn dashboard_render_plan_keeps_full_layout_when_height_allows() {
        let state = sample_dashboard_state();

        let plan = dashboard_render_plan(&state, 72, 12, None, None, 0, None);
        let text: Vec<&str> = plan.lines.iter().map(|line| line.text.as_str()).collect();

        assert_eq!(text[0], "tread");
        assert!(text.iter().any(|line| line.contains("最近阅读")));
        assert!(text.iter().any(|line| line.contains("Enter open")));
        assert!(text.iter().any(|line| line.contains("境界-吴军")));
    }

    #[test]
    fn dashboard_render_plan_uses_compact_recent_area_when_height_is_constrained() {
        let state = sample_dashboard_state();

        let plan = dashboard_render_plan(&state, 72, 5, None, None, 0, None);
        let text: Vec<&str> = plan.lines.iter().map(|line| line.text.as_str()).collect();

        assert_eq!(text.len(), 5);
        assert_eq!(text[0], "tread");
        assert!(text[2].contains("最近阅读"));
        assert!(text[3].contains("EPUB"));
        assert!(text[4].contains("test.epub"));
        assert!(!text.iter().any(|line| line.contains("Enter open")));
    }

    #[test]
    fn dashboard_render_plan_uses_minimal_fallback_below_three_rows() {
        let state = sample_dashboard_state();

        let plan = dashboard_render_plan(&state, 72, 2, None, None, 0, None);

        assert_eq!(plan.lines.len(), 1);
        assert!(plan.lines[0].text.contains("终端高度太小"));
    }

    #[test]
    fn dashboard_render_plan_truncates_prompt_suggestions_to_height_budget() {
        let state = sample_dashboard_state();
        let suggestions = vec![
            dashboard::PathSuggestion {
                path: PathBuf::from("/tmp/alpha.md"),
                display: "alpha.md".to_string(),
                is_dir: false,
            },
            dashboard::PathSuggestion {
                path: PathBuf::from("/tmp/beta.md"),
                display: "beta.md".to_string(),
                is_dir: false,
            },
        ];

        let plan = dashboard_render_plan(
            &state,
            72,
            4,
            Some("~/Books/"),
            Some(suggestions.as_slice()),
            0,
            None,
        );
        let text: Vec<&str> = plan.lines.iter().map(|line| line.text.as_str()).collect();

        assert_eq!(text.len(), 4);
        assert!(text[3].starts_with("打开: ~/Books/"));
        assert!(!text.iter().any(|line| line.contains("alpha.md")));
        assert_eq!(
            plan.prompt_cursor,
            Some((display_width("打开: ~/Books/") as u16, 3))
        );
    }

    #[test]
    fn dashboard_render_plan_may_hide_prompt_to_preserve_recent_area() {
        let state = sample_dashboard_state();
        let suggestions = vec![dashboard::PathSuggestion {
            path: PathBuf::from("/tmp/alpha.md"),
            display: "alpha.md".to_string(),
            is_dir: false,
        }];

        let plan = dashboard_render_plan(
            &state,
            72,
            3,
            Some("~/Books/"),
            Some(suggestions.as_slice()),
            0,
            Some("路径不存在"),
        );
        let text: Vec<&str> = plan.lines.iter().map(|line| line.text.as_str()).collect();

        assert_eq!(text.len(), 3);
        assert!(text[0].contains("最近阅读"));
        assert!(text[1].contains("EPUB"));
        assert!(text[2].contains("test.epub"));
        assert!(!text.iter().any(|line| line.starts_with("打开:")));
        assert!(!text.iter().any(|line| line.starts_with("错误:")));
        assert_eq!(plan.prompt_cursor, None);
    }

    #[test]
    fn dashboard_visual_lines_use_reader_shelf_layout() {
        let state = dashboard::DashboardState::from_history(vec![dashboard::DashboardEntry {
            id: "epub:book".to_string(),
            kind: dashboard::TargetKind::Epub,
            title: "境界-吴军".to_string(),
            target: "/Users/awu/Data/frontend/github/tread/test.epub".to_string(),
            progress_percent: 1,
            updated_at: "2026-07-09T00:00:00Z".to_string(),
            starred: false,
            hidden: false,
        }]);

        let lines = dashboard_visual_lines(&state, 72);
        let text: Vec<&str> = lines.iter().map(|line| line.text.as_str()).collect();

        assert_eq!(text[0], "tread");
        assert!(text.iter().any(|line| line.contains("最近阅读")));
        assert!(text.iter().any(|line| line.contains("EPUB")));
        assert!(text.iter().any(|line| line.contains("境界-吴军")));
        assert!(text.iter().any(|line| line.contains("Enter open")));
        assert!(
            lines
                .iter()
                .any(|line| line.selected && line.text.contains("EPUB"))
        );
    }

    #[test]
    fn dashboard_visual_lines_truncate_long_targets_by_display_width() {
        let state = dashboard::DashboardState::from_history(vec![dashboard::DashboardEntry {
            id: "md:notes".to_string(),
            kind: dashboard::TargetKind::Markdown,
            title: "很长很长的阅读笔记标题".to_string(),
            target: "/a/very/long/path/that/does/not/fit/in/a/narrow/dashboard/notes.md"
                .to_string(),
            progress_percent: 42,
            updated_at: "2026-07-09T00:00:00Z".to_string(),
            starred: false,
            hidden: false,
        }]);

        let lines = dashboard_visual_lines(&state, 48);

        assert!(lines.iter().any(|line| line.text.contains("…")));
        assert!(lines.iter().all(|line| display_width(&line.text) <= 48));
    }

    #[test]
    fn progress_percent_reports_nonzero_after_first_advance() {
        assert_eq!(progress_percent(0, 1_000), 0);
        assert_eq!(progress_percent(1, 1_000), 1);
        assert_eq!(progress_percent(1, 250), 1);
    }
}
