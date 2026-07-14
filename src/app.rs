use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Style;

use crate::image::{ImageNode, LineContent, LinkInfo, StyledSpan};

/// Wrap content lines to fit within `max_width` columns.
/// Image lines pass through unchanged (they occupy exactly one visual line).
pub fn wrap_lines(lines: &[LineContent], max_width: usize) -> Vec<LineContent> {
    let max_width = max_width.max(4);
    let mut result: Vec<LineContent> = Vec::new();

    for line in lines {
        match line {
            LineContent::Image(node) => {
                result.push(LineContent::Image(node.clone()));
            }
            LineContent::Link(node) => {
                result.push(LineContent::Link(node.clone()));
            }
            LineContent::Styled(spans) => {
                let wrapped = wrap_styled_line(spans, max_width);
                for w in wrapped {
                    result.push(LineContent::Styled(w));
                }
            }
        }
    }

    result
}

/// Wrap a single styled line into multiple styled lines.
fn wrap_styled_line(spans: &[StyledSpan], max_width: usize) -> Vec<Vec<StyledSpan>> {
    // Tokenize: each token is a (word, style, link) tuple
    let mut tokens: Vec<(String, Style, Option<crate::image::LinkInfo>)> = Vec::new();
    for span in spans {
        for word in split_words_preserve(&span.text) {
            tokens.push((word.to_string(), span.style, span.link.clone()));
        }
    }

    if tokens.is_empty() {
        return vec![Vec::new()];
    }

    let mut result: Vec<Vec<StyledSpan>> = Vec::new();
    let mut current: Vec<StyledSpan> = Vec::new();
    let mut current_width: usize = 0;

    for (word, style, link) in tokens {
        let word_width = unicode_width::UnicodeWidthStr::width(word.as_str());

        if word.trim().is_empty() {
            if current_width + word_width <= max_width {
                let span = match link {
                    Some(ref l) => StyledSpan::with_link(word.clone(), style, l.clone()),
                    None => StyledSpan::new(word.clone(), style),
                };
                current.push(span);
                current_width += word_width;
            }
            continue;
        }

        if current_width + word_width <= max_width {
            let span = match link {
                Some(ref l) => StyledSpan::with_link(word.clone(), style, l.clone()),
                None => StyledSpan::new(word.clone(), style),
            };
            current.push(span);
            current_width += word_width;
            continue;
        }

        if !current.is_empty() {
            result.push(std::mem::take(&mut current));
            current_width = 0;
        }

        if word_width <= max_width {
            let span = match link {
                Some(ref l) => StyledSpan::with_link(word.clone(), style, l.clone()),
                None => StyledSpan::new(word.clone(), style),
            };
            current.push(span);
            current_width = word_width;
        } else {
            let mut buf = String::new();
            let mut buf_width: usize = 0;
            for ch in word.chars() {
                let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
                if buf_width + ch_width > max_width && !buf.is_empty() {
                    let span = match link {
                        Some(ref l) => StyledSpan::with_link(buf.clone(), style, l.clone()),
                        None => StyledSpan::new(buf.clone(), style),
                    };
                    result.push(vec![span]);
                    buf.clear();
                    buf_width = 0;
                }
                buf.push(ch);
                buf_width += ch_width;
            }
            if !buf.is_empty() {
                let span = match link {
                    Some(ref l) => StyledSpan::with_link(buf.clone(), style, l.clone()),
                    None => StyledSpan::new(buf.clone(), style),
                };
                current.push(span);
                current_width = buf_width;
            }
        }
    }

    result.push(current);
    result
}

/// Split a string into word tokens preserving whitespace.
fn split_words_preserve(s: &str) -> Vec<&str> {
    let mut tokens: Vec<&str> = Vec::new();
    let mut chars = s.char_indices().peekable();

    while let Some(&(i, ch)) = chars.peek() {
        if ch == ' ' || ch == '\t' {
            let start = i;
            let mut end = i;
            while let Some(&(idx, c)) = chars.peek() {
                if c == ' ' || c == '\t' {
                    end = idx + c.len_utf8();
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(&s[start..end]);
        } else if unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0) >= 2 {
            let start = i;
            let end = i + ch.len_utf8();
            tokens.push(&s[start..end]);
            chars.next();
        } else {
            let start = i;
            let mut end = i;
            while let Some(&(idx, c)) = chars.peek() {
                if c == ' ' || c == '\t' {
                    break;
                }
                if unicode_width::UnicodeWidthChar::width(c).unwrap_or(0) >= 2 {
                    break;
                }
                end = idx + c.len_utf8();
                chars.next();
            }
            if end > start {
                tokens.push(&s[start..end]);
            }
        }
    }

    tokens
}

fn same_link(a: &LinkInfo, b: &LinkInfo) -> bool {
    a.url == b.url && a.is_external == b.is_external
}

// ─────────────────────────────────────────────────────────────────────────────
// App state
// ─────────────────────────────────────────────────────────────────────────────

/// A focusable item in the rendered document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusableItem {
    /// A block image placeholder; the entire visual line is focusable.
    Image { line_idx: usize },
    /// A legacy block link; the entire visual line is focusable.
    BlockLink {
        line_idx: usize,
        url: String,
        is_external: bool,
    },
    /// An inline link range within a styled visual line.
    InlineLink {
        line_idx: usize,
        start_offset: usize,
        end_offset: usize,
        url: String,
        is_external: bool,
    },
}

impl FocusableItem {
    pub fn line_idx(&self) -> usize {
        match self {
            Self::Image { line_idx }
            | Self::BlockLink { line_idx, .. }
            | Self::InlineLink { line_idx, .. } => *line_idx,
        }
    }

    pub fn inline_range_on_line(&self, line_idx: usize) -> Option<(usize, usize)> {
        match self {
            Self::InlineLink {
                line_idx: item_line,
                start_offset,
                end_offset,
                ..
            } if *item_line == line_idx => Some((*start_offset, *end_offset)),
            _ => None,
        }
    }

    pub fn is_entire_line_on_line(&self, line_idx: usize) -> bool {
        match self {
            Self::Image {
                line_idx: item_line,
            }
            | Self::BlockLink {
                line_idx: item_line,
                ..
            } => *item_line == line_idx,
            Self::InlineLink { .. } => false,
        }
    }

    pub fn link(&self) -> Option<(String, bool)> {
        match self {
            Self::BlockLink {
                url, is_external, ..
            }
            | Self::InlineLink {
                url, is_external, ..
            } => Some((url.clone(), *is_external)),
            Self::Image { .. } => None,
        }
    }
}

/// Terminal reader application state.
pub struct App {
    pub lines: Vec<LineContent>,
    pub wrapped_lines: Vec<LineContent>,
    pub scroll: usize,
    pub height: Option<u16>,
    pub help_visible: bool,
    pub filename: String,
    /// Focusable items in document order.
    pub focusable_positions: Vec<FocusableItem>,
    /// Currently focused item index in `focusable_positions` (None = no focus).
    pub focus_index: Option<usize>,
    /// Temporary status message.
    pub status_message: Option<String>,
    // ── EPUB fields ─────────────────────────────────────────────────────────
    /// EPUB book hash (None for Markdown files).
    pub epub_hash: Option<String>,
    /// Total chapters in EPUB.
    pub chapter_count: usize,
    /// Current chapter index (0-based).
    pub current_chapter: usize,
    /// Table of contents entries.
    pub toc: Vec<crate::epub::TocEntry>,
    /// Spine hrefs (one per chapter) — for mapping TOC/link hrefs to chapter indices.
    pub spine_hrefs: Vec<String>,
    /// Whether TOC overlay is visible.
    pub toc_visible: bool,
    /// Selected index in TOC overlay.
    pub toc_selection: usize,
    /// Whether the status bar is visible.
    pub status_bar_visible: bool,
}

pub enum Action {
    Continue,
    Quit,
    Dashboard,
    /// Request to load a different chapter (EPUB mode).
    /// The bool indicates whether to scroll to bottom (true = prev chapter, false = next chapter).
    LoadChapter(usize, bool),
}

impl App {
    pub fn new(lines: Vec<LineContent>, filename: String) -> Self {
        Self {
            lines,
            wrapped_lines: Vec::new(),
            scroll: 0,
            height: None,
            help_visible: false,
            filename,
            focusable_positions: Vec::new(),
            focus_index: None,
            status_message: None,
            epub_hash: None,
            chapter_count: 0,
            current_chapter: 0,
            toc: Vec::new(),
            spine_hrefs: Vec::new(),
            toc_visible: false,
            toc_selection: 0,
            status_bar_visible: true,
        }
    }

    /// Enable EPUB mode with chapter navigation.
    pub fn set_epub_mode(
        &mut self,
        book_hash: Option<String>,
        chapter_count: usize,
        initial_chapter: usize,
        toc: Vec<crate::epub::TocEntry>,
        spine_hrefs: Vec<String>,
    ) {
        self.epub_hash = book_hash;
        self.chapter_count = chapter_count;
        self.current_chapter = initial_chapter;
        self.toc = toc;
        self.spine_hrefs = spine_hrefs;
    }

    /// Map an href to a chapter index. Tries exact match, then prefix match.
    pub fn chapter_for_href(&self, href: &str) -> Option<usize> {
        // Strip fragment identifier
        let base_href = href.split('#').next().unwrap_or(href);
        // Exact match
        self.spine_hrefs
            .iter()
            .position(|h| h == base_href)
            .or_else(|| {
                // Prefix/contains match
                self.spine_hrefs.iter().position(|h| {
                    h == base_href || h.ends_with(base_href) || base_href.ends_with(h)
                })
            })
    }

    pub fn set_height(&mut self, height: u16) {
        let old_height = self.height;
        self.height = Some(height);
        if old_height != Some(height) {
            let term_width = crossterm::terminal::size()
                .map(|(w, _)| w as usize)
                .unwrap_or(80);
            self.wrapped_lines = wrap_lines(&self.lines, term_width);
            self.rebuild_focusable_positions();
            self.clamp_scroll();
        }
    }

    fn rebuild_focusable_positions(&mut self) {
        let mut positions: Vec<FocusableItem> = Vec::new();

        for (line_idx, lc) in self.wrapped_lines.iter().enumerate() {
            match lc {
                LineContent::Image(_) => {
                    // Entire line is focusable
                    positions.push(FocusableItem::Image { line_idx });
                }
                LineContent::Link(node) => {
                    // Legacy block link support
                    positions.push(FocusableItem::BlockLink {
                        line_idx,
                        url: node.url.clone(),
                        is_external: node.is_external,
                    });
                }
                LineContent::Styled(spans) => {
                    // Scan for inline links and merge adjacent spans that belong to the same link.
                    let mut char_offset = 0;
                    let mut pending: Option<(usize, usize, LinkInfo)> = None;

                    for span in spans {
                        let span_width = unicode_width::UnicodeWidthStr::width(span.text.as_str());
                        match (&mut pending, &span.link) {
                            (Some((_, end_offset, current_link)), Some(link))
                                if same_link(current_link, link) =>
                            {
                                *end_offset = char_offset + span_width;
                            }
                            (Some((start_offset, end_offset, current_link)), Some(link)) => {
                                positions.push(FocusableItem::InlineLink {
                                    line_idx,
                                    start_offset: *start_offset,
                                    end_offset: *end_offset,
                                    url: current_link.url.clone(),
                                    is_external: current_link.is_external,
                                });
                                pending =
                                    Some((char_offset, char_offset + span_width, link.clone()));
                            }
                            (None, Some(link)) => {
                                pending =
                                    Some((char_offset, char_offset + span_width, link.clone()));
                            }
                            (Some((start_offset, end_offset, current_link)), None) => {
                                positions.push(FocusableItem::InlineLink {
                                    line_idx,
                                    start_offset: *start_offset,
                                    end_offset: *end_offset,
                                    url: current_link.url.clone(),
                                    is_external: current_link.is_external,
                                });
                                pending = None;
                            }
                            (None, None) => {}
                        }
                        char_offset += span_width;
                    }

                    if let Some((start_offset, end_offset, link)) = pending {
                        positions.push(FocusableItem::InlineLink {
                            line_idx,
                            start_offset,
                            end_offset,
                            url: link.url,
                            is_external: link.is_external,
                        });
                    }
                }
            }
        }

        self.focusable_positions = positions;

        // Clamp focus
        if let Some(focus) = self.focus_index {
            if focus >= self.focusable_positions.len() {
                self.focus_index = if self.focusable_positions.is_empty() {
                    None
                } else {
                    Some(self.focusable_positions.len() - 1)
                };
            }
        }
    }

    pub fn scroll_to_line(&mut self, line: usize) {
        self.scroll = line;
        self.clamp_scroll();
    }

    /// Replace the content lines (used when switching EPUB chapters).
    pub fn replace_content(&mut self, new_lines: Vec<LineContent>) {
        self.lines = new_lines;
        self.scroll = 0;
        self.focus_index = None;
        self.status_message = None;
        // Force re-wrap on next draw
        let h = self.height;
        self.height = None;
        if let Some(h) = h {
            self.set_height(h);
        }
    }

    fn clamp_scroll(&mut self) {
        let max = self.max_scroll();
        if self.scroll > max {
            self.scroll = max;
        }
    }

    fn max_scroll(&self) -> usize {
        let h = self.height.unwrap_or(0) as usize;
        self.wrapped_lines.len().saturating_sub(h)
    }

    pub fn total_lines(&self) -> usize {
        self.wrapped_lines.len()
    }

    fn find_first_focusable_from_viewport(&self) -> usize {
        self.focusable_positions
            .iter()
            .position(|item| item.line_idx() >= self.scroll)
            .unwrap_or_else(|| self.focusable_positions.len().saturating_sub(1))
    }

    fn find_last_focusable_before_viewport(&self, height: usize) -> usize {
        let viewport_end = self.scroll.saturating_add(height);
        self.focusable_positions
            .iter()
            .rposition(|item| item.line_idx() < viewport_end)
            .unwrap_or(0)
    }

    /// Get the focused link info (if any).
    /// Returns (url, is_external) for the focused link.
    pub fn focused_link(&self) -> Option<(String, bool)> {
        let focus_idx = self.focus_index?;
        self.focusable_positions.get(focus_idx)?.link()
    }

    /// Get the focused content item (if any).
    pub fn focused_item(&self) -> Option<&LineContent> {
        let focus_idx = self.focus_index?;
        let line_idx = self.focusable_positions.get(focus_idx)?.line_idx();
        Some(&self.wrapped_lines[line_idx])
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Action {
        // TOC overlay mode
        if self.toc_visible {
            match key.code {
                KeyCode::Esc | KeyCode::Char('t') => {
                    self.toc_visible = false;
                    return Action::Continue;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.toc_selection = self.toc_selection.saturating_sub(1);
                    return Action::Continue;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.toc_selection + 1 < self.toc.len() {
                        self.toc_selection += 1;
                    }
                    return Action::Continue;
                }
                KeyCode::Enter => {
                    // Jump to selected TOC entry
                    if let Some(entry) = self.toc.get(self.toc_selection) {
                        if let Some(chapter) = self.chapter_for_href(&entry.href) {
                            self.toc_visible = false;
                            return Action::LoadChapter(chapter, false);
                        } else {
                            self.status_message = Some(format!("未找到: {}", &entry.href));
                        }
                    }
                    self.toc_visible = false;
                    return Action::Continue;
                }
                _ => return Action::Continue,
            }
        }

        if self.help_visible {
            self.help_visible = false;
            return Action::Continue;
        }

        // Clear status message on any key except Enter
        if key.code != KeyCode::Enter {
            self.status_message = None;
        }

        let h = self.height.unwrap_or(0) as usize;
        let half_page = (h / 2).max(1);

        match key.code {
            KeyCode::Char('D') => return Action::Dashboard,
            KeyCode::Char('q') | KeyCode::Char('Q') => return Action::Quit,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Action::Quit;
            }
            KeyCode::Esc => {
                if self.toc_visible {
                    self.toc_visible = false;
                    return Action::Continue;
                }
                return Action::Quit;
            }

            // EPUB: chapter navigation
            KeyCode::Char('n')
                if key.modifiers.contains(KeyModifiers::CONTROL) && self.chapter_count > 0 =>
            {
                if self.current_chapter + 1 < self.chapter_count {
                    return Action::LoadChapter(self.current_chapter + 1, false);
                } else {
                    self.status_message = Some("已是最后一章".to_string());
                }
            }
            KeyCode::Char('p')
                if key.modifiers.contains(KeyModifiers::CONTROL) && self.chapter_count > 0 =>
            {
                if self.current_chapter > 0 {
                    return Action::LoadChapter(self.current_chapter - 1, true);
                } else {
                    self.status_message = Some("已是第一章".to_string());
                }
            }

            // EPUB: table of contents
            KeyCode::Char('t') if self.chapter_count > 0 => {
                self.toc_visible = !self.toc_visible;
                self.toc_selection = 0;
            }

            // Image focus navigation
            KeyCode::Tab => {
                if !self.focusable_positions.is_empty() {
                    self.focus_index = Some(match self.focus_index {
                        None => self.find_first_focusable_from_viewport(),
                        Some(f) => (f + 1) % self.focusable_positions.len(),
                    });
                    // Scroll to show the focused item
                    if let Some(focus) = self.focus_index {
                        let line_idx = self.focusable_positions[focus].line_idx();
                        if line_idx < self.scroll || line_idx >= self.scroll + h {
                            self.scroll = line_idx.saturating_sub(h / 2);
                        }
                    }
                }
            }
            KeyCode::BackTab => {
                if !self.focusable_positions.is_empty() {
                    self.focus_index = Some(match self.focus_index {
                        None => self.find_last_focusable_before_viewport(h),
                        Some(0) => self.focusable_positions.len() - 1,
                        Some(f) => f - 1,
                    });
                    if let Some(focus) = self.focus_index {
                        let line_idx = self.focusable_positions[focus].line_idx();
                        if line_idx < self.scroll || line_idx >= self.scroll + h {
                            self.scroll = line_idx.saturating_sub(h / 2);
                        }
                    }
                }
            }
            KeyCode::Enter => {
                // Open focused link if any
                if let Some((url, is_external)) = self.focused_link() {
                    if is_external {
                        match crate::image::open_url(&url) {
                            Ok(()) => self.status_message = Some(format!("已打开: {}", &url)),
                            Err(e) => self.status_message = Some(format!("打开失败: {e}")),
                        }
                    } else if self.chapter_count > 0 {
                        // Internal link in EPUB — try to find target chapter
                        if let Some(chapter) = self.chapter_for_href(&url) {
                            return Action::LoadChapter(chapter, false);
                        } else {
                            self.status_message = Some(format!("链接目标未找到: {}", &url));
                        }
                    } else {
                        self.status_message = Some(format!("链接: {} (非 EPUB 模式)", &url));
                    }
                } else {
                    // Check if focused item is an image
                    match self.focused_item() {
                        Some(LineContent::Image(img)) => {
                            if let Some(ref path) = img.local_path {
                                match crate::image::open_with_viewer(path) {
                                    Ok(()) => self.status_message = None,
                                    Err(e) => self.status_message = Some(format!("打开失败: {e}")),
                                }
                            } else {
                                self.status_message =
                                    Some("图片未缓存或下载失败，无法打开".to_string());
                            }
                        }
                        _ => {
                            self.status_message = None;
                        }
                    }
                }
            }

            KeyCode::Char('j') | KeyCode::Down => {
                if self.scroll >= self.max_scroll() {
                    // At bottom — auto-advance to next chapter (EPUB only)
                    if self.chapter_count > 0 && self.current_chapter + 1 < self.chapter_count {
                        return Action::LoadChapter(self.current_chapter + 1, false);
                    }
                } else {
                    self.scroll = self.scroll.saturating_add(1);
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.scroll == 0 {
                    // At top — auto-retreat to previous chapter (EPUB only)
                    if self.chapter_count > 0 && self.current_chapter > 0 {
                        return Action::LoadChapter(self.current_chapter - 1, true);
                    }
                } else {
                    self.scroll = self.scroll.saturating_sub(1);
                }
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll = self.scroll.saturating_add(half_page);
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll = self.scroll.saturating_sub(half_page);
            }
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll = self.scroll.saturating_add(h);
            }
            KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll = self.scroll.saturating_sub(h);
            }
            KeyCode::PageDown => {
                self.scroll = self.scroll.saturating_add(half_page);
            }
            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_sub(half_page);
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.scroll = 0;
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.scroll = self.max_scroll();
            }
            KeyCode::Char('?') => {
                self.help_visible = true;
            }
            KeyCode::Char('f') => {
                self.status_bar_visible = !self.status_bar_visible;
                // Force re-wrap since content height changed
                self.height = None;
            }
            _ => {}
        }

        self.clamp_scroll();
        Action::Continue
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::LineContent;
    use ratatui::style::{Color, Modifier, Style};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn key_ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    fn key_shift(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::SHIFT)
    }

    /// Build N styled lines, each containing "line_i".
    fn make_lines(n: usize) -> Vec<LineContent> {
        (0..n)
            .map(|i| {
                LineContent::Styled(vec![StyledSpan::new(format!("line_{i}"), Style::default())])
            })
            .collect()
    }

    fn make_lines_with_images(text_count: usize, image_count: usize) -> Vec<LineContent> {
        let mut lines = Vec::new();
        for i in 0..text_count {
            lines.push(LineContent::Styled(vec![StyledSpan::new(
                format!("line_{i}"),
                Style::default(),
            )]));
            if i < image_count {
                lines.push(LineContent::Image(ImageNode {
                    alt: format!("img_{i}"),
                    url: format!("img_{i}.png"),
                    local_path: None,
                    id: i,
                    download_failed: false,
                }));
            }
        }
        lines
    }

    fn plain(lines: &[LineContent]) -> Vec<String> {
        lines
            .iter()
            .filter_map(|lc| match lc {
                LineContent::Styled(spans) => {
                    Some(spans.iter().map(|s| s.text.as_str()).collect::<String>())
                }
                LineContent::Image(_) | LineContent::Link(_) => None,
            })
            .collect()
    }

    // ── 3.x: wrap_lines ──────────────────────────────────────────────────────

    #[test]
    fn wrap_empty_input_returns_empty() {
        let result = wrap_lines(&[], 10);
        assert!(result.is_empty());
    }

    #[test]
    fn wrap_empty_line_stays_empty() {
        let input = vec![LineContent::Styled(Vec::new())];
        let result = wrap_lines(&input, 10);
        assert_eq!(result.len(), 1);
        match &result[0] {
            LineContent::Styled(spans) => assert!(spans.is_empty()),
            _ => panic!("expected styled line"),
        }
    }

    #[test]
    fn wrap_short_line_not_broken() {
        let input = vec![LineContent::Styled(vec![StyledSpan::new(
            "hello".to_string(),
            Style::default(),
        )])];
        let result = wrap_lines(&input, 80);
        assert_eq!(result.len(), 1);
        assert_eq!(plain(&result)[0], "hello");
    }

    #[test]
    fn wrap_english_breaks_at_word_boundary() {
        let input = vec![LineContent::Styled(vec![StyledSpan::new(
            "hello world foo".to_string(),
            Style::default(),
        )])];
        let result = wrap_lines(&input, 12);
        assert!(
            result.len() >= 2,
            "expected at least 2 lines, got {}",
            result.len()
        );
        let text = plain(&result);
        assert_eq!(text[0].trim(), "hello world");
        assert_eq!(text[1].trim(), "foo");
    }

    #[test]
    fn wrap_cjk_breaks_at_character_boundary() {
        let input = vec![LineContent::Styled(vec![StyledSpan::new(
            "你好世界".to_string(),
            Style::default(),
        )])];
        let result = wrap_lines(&input, 4);
        assert!(result.len() >= 2, "expected at least 2 lines for CJK wrap");
    }

    #[test]
    fn wrap_mixed_cjk_english() {
        let input = vec![LineContent::Styled(vec![StyledSpan::new(
            "hi 你好".to_string(),
            Style::default(),
        )])];
        let result = wrap_lines(&input, 6);
        assert!(result.len() >= 2, "expected mixed content to wrap");
    }

    #[test]
    fn wrap_long_word_force_breaks() {
        let input = vec![LineContent::Styled(vec![StyledSpan::new(
            "abcdefghijklmnop".to_string(),
            Style::default(),
        )])];
        let result = wrap_lines(&input, 8);
        assert!(result.len() >= 2, "expected force-break for long word");
    }

    #[test]
    fn wrap_preserves_style() {
        let bold = Style::default().add_modifier(Modifier::BOLD);
        let input = vec![LineContent::Styled(vec![StyledSpan::new(
            "hello world".to_string(),
            bold,
        )])];
        let result = wrap_lines(&input, 80);
        assert!(!result.is_empty());
        match &result[0] {
            LineContent::Styled(spans) => {
                assert!(
                    spans
                        .iter()
                        .any(|span| span.style.add_modifier.contains(Modifier::BOLD))
                );
            }
            _ => panic!("expected styled line"),
        }
    }

    #[test]
    fn wrap_image_passes_through() {
        let input = vec![
            LineContent::Styled(vec![StyledSpan::new("text".to_string(), Style::default())]),
            LineContent::Image(ImageNode {
                alt: "test".to_string(),
                url: "test.png".to_string(),
                local_path: None,
                id: 0,
                download_failed: false,
            }),
        ];
        let result = wrap_lines(&input, 80);
        assert_eq!(result.len(), 2);
        match &result[1] {
            LineContent::Image(node) => assert_eq!(node.alt, "test"),
            _ => panic!("expected image line"),
        }
    }

    // ── 4.x: App state ───────────────────────────────────────────────────────

    fn make_app(n_lines: usize, height: u16) -> App {
        let lines = make_lines(n_lines);
        let mut app = App::new(lines, "test.md".to_string());
        app.set_height(height);
        app
    }

    #[test]
    fn j_scrolls_down_one_line() {
        let mut app = make_app(100, 10);
        assert_eq!(app.scroll, 0);
        app.handle_key(key(KeyCode::Char('j')));
        assert_eq!(app.scroll, 1);
    }

    #[test]
    fn k_scrolls_up_one_line() {
        let mut app = make_app(100, 10);
        app.scroll_to_line(5);
        app.handle_key(key(KeyCode::Char('k')));
        assert_eq!(app.scroll, 4);
    }

    #[test]
    fn ctrl_d_half_page_down() {
        let mut app = make_app(100, 10);
        app.handle_key(key_ctrl(KeyCode::Char('d')));
        assert_eq!(app.scroll, 5);
    }

    #[test]
    fn ctrl_u_half_page_up() {
        let mut app = make_app(100, 10);
        app.scroll_to_line(10);
        app.handle_key(key_ctrl(KeyCode::Char('u')));
        assert_eq!(app.scroll, 5);
    }

    #[test]
    fn ctrl_f_full_page_down() {
        let mut app = make_app(100, 10);
        app.handle_key(key_ctrl(KeyCode::Char('f')));
        assert_eq!(app.scroll, 10);
    }

    #[test]
    fn ctrl_b_full_page_up() {
        let mut app = make_app(100, 10);
        app.scroll_to_line(20);
        app.handle_key(key_ctrl(KeyCode::Char('b')));
        assert_eq!(app.scroll, 10);
    }

    #[test]
    fn g_jumps_to_top() {
        let mut app = make_app(100, 10);
        app.scroll_to_line(50);
        app.handle_key(key(KeyCode::Char('g')));
        assert_eq!(app.scroll, 0);
    }

    #[test]
    fn shift_g_jumps_to_bottom() {
        let mut app = make_app(100, 10);
        app.handle_key(key(KeyCode::Char('G')));
        assert_eq!(app.scroll, 90);
    }

    #[test]
    fn home_jumps_to_top() {
        let mut app = make_app(100, 10);
        app.scroll_to_line(50);
        app.handle_key(key(KeyCode::Home));
        assert_eq!(app.scroll, 0);
    }

    #[test]
    fn end_jumps_to_bottom() {
        let mut app = make_app(100, 10);
        app.handle_key(key(KeyCode::End));
        assert_eq!(app.scroll, 90);
    }

    #[test]
    fn page_down_half_page() {
        let mut app = make_app(100, 10);
        app.handle_key(key(KeyCode::PageDown));
        assert_eq!(app.scroll, 5);
    }

    #[test]
    fn page_up_half_page() {
        let mut app = make_app(100, 10);
        app.scroll_to_line(10);
        app.handle_key(key(KeyCode::PageUp));
        assert_eq!(app.scroll, 5);
    }

    #[test]
    fn scroll_cannot_go_below_zero() {
        let mut app = make_app(100, 10);
        assert_eq!(app.scroll, 0);
        app.handle_key(key(KeyCode::Char('k')));
        assert_eq!(app.scroll, 0);
    }

    #[test]
    fn scroll_cannot_exceed_max() {
        let mut app = make_app(100, 10);
        app.handle_key(key(KeyCode::Char('G')));
        let max = app.scroll;
        app.handle_key(key(KeyCode::Char('j')));
        assert_eq!(app.scroll, max);
    }

    #[test]
    fn short_content_cannot_scroll() {
        let mut app = make_app(5, 10);
        assert_eq!(app.scroll, 0);
        app.handle_key(key(KeyCode::Char('j')));
        assert_eq!(app.scroll, 0);
    }

    #[test]
    fn question_mark_toggles_help() {
        let mut app = make_app(100, 10);
        assert!(!app.help_visible);
        app.handle_key(key(KeyCode::Char('?')));
        assert!(app.help_visible);
        app.handle_key(key(KeyCode::Char('x')));
        assert!(!app.help_visible);
    }

    #[test]
    fn q_returns_quit() {
        let mut app = make_app(100, 10);
        let action = app.handle_key(key(KeyCode::Char('q')));
        assert!(matches!(action, Action::Quit));
    }

    #[test]
    fn ctrl_c_returns_quit() {
        let mut app = make_app(100, 10);
        let action = app.handle_key(key_ctrl(KeyCode::Char('c')));
        assert!(matches!(action, Action::Quit));
    }

    #[test]
    fn esc_returns_quit() {
        let mut app = make_app(100, 10);
        let action = app.handle_key(key(KeyCode::Esc));
        assert!(matches!(action, Action::Quit));
    }

    #[test]
    fn capital_d_returns_dashboard() {
        let mut app = make_app(100, 10);
        let action = app.handle_key(key(KeyCode::Char('D')));
        assert!(matches!(action, Action::Dashboard));
    }

    #[test]
    fn set_height_rewraps_and_clamps() {
        let mut app = make_app(100, 50);
        app.scroll_to_line(40);
        app.set_height(10);
        assert_eq!(app.scroll, 40);
        app.set_height(5);
        assert!(app.scroll <= app.scroll.max(0));
    }

    #[test]
    fn total_lines_returns_wrapped_count() {
        let mut app = make_app(100, 10);
        assert_eq!(app.total_lines(), 100);
    }

    // ── 4.x: Image focus navigation ─────────────────────────────────────────

    #[test]
    fn tab_focuses_first_image() {
        let lines = make_lines_with_images(5, 3);
        let mut app = App::new(lines, "test.md".to_string());
        app.set_height(50);
        assert!(app.focus_index.is_none());
        app.handle_key(key(KeyCode::Tab));
        assert_eq!(app.focus_index, Some(0));
    }

    #[test]
    fn tab_initial_focus_starts_from_viewport() {
        let lines = make_lines_with_images(10, 5);
        let mut app = App::new(lines, "test.md".to_string());
        app.set_height(3);
        app.scroll_to_line(4);

        assert!(app.focus_index.is_none());
        app.handle_key(key(KeyCode::Tab));

        // Images are at wrapped line indices 1, 3, 5, 7, 9.
        // From scroll=4, Tab should choose the first focusable item in/after viewport: line 5.
        assert_eq!(app.focus_index, Some(2));
        assert_eq!(app.focusable_positions[2].line_idx(), 5);
    }

    #[test]
    fn shift_tab_initial_focus_starts_from_viewport() {
        let lines = make_lines_with_images(10, 5);
        let mut app = App::new(lines, "test.md".to_string());
        app.set_height(3);
        app.scroll_to_line(4);

        assert!(app.focus_index.is_none());
        app.handle_key(key_shift(KeyCode::BackTab));

        // Visible range is 4..7; Shift+Tab should choose the last focusable item
        // in/before viewport: line 5.
        assert_eq!(app.focus_index, Some(2));
        assert_eq!(app.focusable_positions[2].line_idx(), 5);
    }

    #[test]
    fn tab_cycles_through_images() {
        let lines = make_lines_with_images(5, 3);
        let mut app = App::new(lines, "test.md".to_string());
        app.set_height(50);
        app.handle_key(key(KeyCode::Tab));
        assert_eq!(app.focus_index, Some(0));
        app.handle_key(key(KeyCode::Tab));
        assert_eq!(app.focus_index, Some(1));
        app.handle_key(key(KeyCode::Tab));
        assert_eq!(app.focus_index, Some(2));
        // Wrap around
        app.handle_key(key(KeyCode::Tab));
        assert_eq!(app.focus_index, Some(0));
    }

    #[test]
    fn shift_tab_goes_to_previous_image() {
        let lines = make_lines_with_images(5, 3);
        let mut app = App::new(lines, "test.md".to_string());
        app.set_height(50);
        app.handle_key(key(KeyCode::Tab));
        app.handle_key(key(KeyCode::Tab));
        assert_eq!(app.focus_index, Some(1));
        app.handle_key(key_shift(KeyCode::BackTab));
        assert_eq!(app.focus_index, Some(0));
    }

    #[test]
    fn shift_tab_wraps_to_last() {
        let lines = make_lines_with_images(5, 3);
        let mut app = App::new(lines, "test.md".to_string());
        app.set_height(50);
        // Shift+Tab from no focus → last image
        app.handle_key(key_shift(KeyCode::BackTab));
        assert_eq!(app.focus_index, Some(2));
    }

    #[test]
    fn tab_with_no_images_does_nothing() {
        let lines = make_lines(10);
        let mut app = App::new(lines, "test.md".to_string());
        app.set_height(50);
        assert!(app.focusable_positions.is_empty());
        app.handle_key(key(KeyCode::Tab));
        assert!(app.focus_index.is_none());
    }

    #[test]
    fn focused_item_returns_correct_node() {
        let lines = make_lines_with_images(5, 3);
        let mut app = App::new(lines, "test.md".to_string());
        app.set_height(50);
        app.handle_key(key(KeyCode::Tab));
        let item = app.focused_item();
        assert!(item.is_some());
        match item.unwrap() {
            LineContent::Image(node) => assert_eq!(node.alt, "img_0"),
            _ => panic!("expected Image"),
        }
    }

    #[test]
    fn multi_word_inline_link_is_one_focus_target_after_wrap() {
        let link = crate::image::LinkInfo {
            url: "https://example.com".to_string(),
            is_external: true,
        };
        let lines = vec![LineContent::Styled(vec![StyledSpan::with_link(
            "Learn more".to_string(),
            Style::default(),
            link,
        )])];
        let mut app = App::new(lines, "test.md".to_string());
        app.set_height(50);

        assert_eq!(plain(&app.wrapped_lines), vec!["Learn more".to_string()]);
        assert_eq!(app.focusable_positions.len(), 1);

        app.handle_key(key(KeyCode::Tab));
        assert_eq!(app.focus_index, Some(0));
        assert_eq!(
            app.focused_link(),
            Some(("https://example.com".to_string(), true))
        );
    }

    #[test]
    fn tab_navigates_links_and_images_together() {
        // Mix of images and links
        let lines = vec![
            LineContent::Styled(vec![StyledSpan::new("text".to_string(), Style::default())]),
            LineContent::Image(crate::image::ImageNode {
                alt: "img".to_string(),
                url: "img.png".to_string(),
                local_path: None,
                id: 0,
                download_failed: false,
            }),
            LineContent::Styled(vec![StyledSpan::new(
                "more text".to_string(),
                Style::default(),
            )]),
            LineContent::Link(crate::image::LinkNode {
                text: "link".to_string(),
                url: "https://example.com".to_string(),
                is_external: true,
            }),
        ];
        let mut app = App::new(lines, "test.md".to_string());
        app.set_height(50);
        assert_eq!(app.focusable_positions.len(), 2);
        // Tab to image
        app.handle_key(key(KeyCode::Tab));
        assert_eq!(app.focus_index, Some(0));
        // Tab to link
        app.handle_key(key(KeyCode::Tab));
        assert_eq!(app.focus_index, Some(1));
        // Tab wraps to image
        app.handle_key(key(KeyCode::Tab));
        assert_eq!(app.focus_index, Some(0));
    }

    #[test]
    fn enter_on_external_link_opens_url() {
        let lines = vec![LineContent::Link(crate::image::LinkNode {
            text: "visit".to_string(),
            url: "https://example.com".to_string(),
            is_external: true,
        })];
        let mut app = App::new(lines, "test.md".to_string());
        app.set_height(50);
        app.handle_key(key(KeyCode::Tab)); // focus the link
        // Enter should set status message (we can't actually open a URL in tests)
        app.handle_key(key(KeyCode::Enter));
        assert!(app.status_message.is_some());
    }

    #[test]
    fn chapter_for_href_finds_chapter() {
        let mut app = App::new(vec![], "test.epub".to_string());
        app.set_epub_mode(
            None,
            3,
            0,
            vec![
                crate::epub::TocEntry {
                    title: "第一章".to_string(),
                    href: "ch1.xhtml".to_string(),
                    level: 1,
                },
                crate::epub::TocEntry {
                    title: "第二章".to_string(),
                    href: "ch2.xhtml".to_string(),
                    level: 1,
                },
                crate::epub::TocEntry {
                    title: "第三章".to_string(),
                    href: "ch3.xhtml".to_string(),
                    level: 1,
                },
            ],
            vec![
                "ch1.xhtml".to_string(),
                "ch2.xhtml".to_string(),
                "ch3.xhtml".to_string(),
            ],
        );
        assert_eq!(app.chapter_for_href("ch1.xhtml"), Some(0));
        assert_eq!(app.chapter_for_href("ch2.xhtml"), Some(1));
        assert_eq!(app.chapter_for_href("ch2.xhtml#section"), Some(1));
        assert_eq!(app.chapter_for_href("nonexistent"), None);
    }

    #[test]
    fn internal_link_jumps_to_chapter() {
        let lines = vec![LineContent::Link(crate::image::LinkNode {
            text: "跳转".to_string(),
            url: "ch2.xhtml".to_string(),
            is_external: false,
        })];
        let mut app = App::new(lines, "test.epub".to_string());
        app.set_epub_mode(
            None,
            3,
            0,
            vec![],
            vec![
                "ch1.xhtml".to_string(),
                "ch2.xhtml".to_string(),
                "ch3.xhtml".to_string(),
            ],
        );
        app.set_height(50);
        app.handle_key(key(KeyCode::Tab)); // focus link
        let action = app.handle_key(key(KeyCode::Enter));
        assert!(matches!(action, Action::LoadChapter(1, _)));
    }

    #[test]
    fn toc_navigation_and_jump() {
        let mut app = App::new(vec![], "test.epub".to_string());
        app.set_epub_mode(
            None,
            3,
            0,
            vec![
                crate::epub::TocEntry {
                    title: "第一章".to_string(),
                    href: "ch1.xhtml".to_string(),
                    level: 1,
                },
                crate::epub::TocEntry {
                    title: "第二章".to_string(),
                    href: "ch2.xhtml".to_string(),
                    level: 1,
                },
            ],
            vec!["ch1.xhtml".to_string(), "ch2.xhtml".to_string()],
        );
        // Open TOC
        app.handle_key(key(KeyCode::Char('t')));
        assert!(app.toc_visible);
        assert_eq!(app.toc_selection, 0);
        // Navigate down
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.toc_selection, 1);
        // Enter → jump to chapter 1
        let action = app.handle_key(key(KeyCode::Enter));
        assert!(matches!(action, Action::LoadChapter(1, _)));
        assert!(!app.toc_visible);
    }

    #[test]
    fn f_toggles_status_bar() {
        let mut app = make_app(10, 10);
        assert!(app.status_bar_visible);
        app.handle_key(key(KeyCode::Char('f')));
        assert!(!app.status_bar_visible);
        app.handle_key(key(KeyCode::Char('f')));
        assert!(app.status_bar_visible);
    }

    #[test]
    fn j_at_bottom_advances_chapter() {
        let mut app = make_app(100, 10);
        app.set_epub_mode(None, 3, 0, vec![], vec![]);
        // Scroll to bottom
        app.handle_key(key(KeyCode::Char('G')));
        assert!(app.scroll >= app.max_scroll());
        // j at bottom → next chapter
        let action = app.handle_key(key(KeyCode::Char('j')));
        assert!(matches!(action, Action::LoadChapter(1, _)));
    }

    #[test]
    fn k_at_top_retreats_chapter() {
        let mut app = make_app(100, 10);
        app.set_epub_mode(None, 3, 1, vec![], vec![]);
        // At top (scroll = 0)
        assert_eq!(app.scroll, 0);
        // k at top → previous chapter
        let action = app.handle_key(key(KeyCode::Char('k')));
        assert!(matches!(action, Action::LoadChapter(0, _)));
    }

    #[test]
    fn j_at_bottom_last_chapter_stays() {
        let mut app = make_app(100, 10);
        app.set_epub_mode(None, 3, 2, vec![], vec![]); // Last chapter (index 2)
        app.handle_key(key(KeyCode::Char('G'))); // Go to bottom
        // j at bottom of last chapter → stay
        let action = app.handle_key(key(KeyCode::Char('j')));
        assert!(matches!(action, Action::Continue));
    }

    #[test]
    fn k_at_top_first_chapter_stays() {
        let mut app = make_app(100, 10);
        app.set_epub_mode(None, 3, 0, vec![], vec![]); // First chapter
        // k at top of first chapter → stay
        let action = app.handle_key(key(KeyCode::Char('k')));
        assert!(matches!(action, Action::Continue));
    }
}
