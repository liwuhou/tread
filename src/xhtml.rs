use quick_xml::events::Event;
use quick_xml::reader::Reader;
use ratatui::style::{Color, Modifier, Style};
use unicode_width::UnicodeWidthStr;

use crate::image::{ImageNode, LineContent, StyledSpan, LinkInfo};

/// Represents a parsed HTML table.
#[derive(Debug, Clone)]
struct Table {
    rows: Vec<Vec<String>>,
    has_header: bool,
}

impl Table {
    fn new() -> Self {
        Self {
            rows: Vec::new(),
            has_header: false,
        }
    }

    fn add_row(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }

    fn current_row_mut(&mut self) -> Option<&mut Vec<String>> {
        self.rows.last_mut()
    }

    /// Calculate the maximum width for each column.
    fn column_widths(&self) -> Vec<usize> {
        if self.rows.is_empty() {
            return Vec::new();
        }

        let max_cols = self.rows.iter().map(|r| r.len()).max().unwrap_or(0);
        let mut widths = vec![0; max_cols];

        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                let cell_width = UnicodeWidthStr::width(cell.as_str());
                if cell_width > widths[i] {
                    widths[i] = cell_width;
                }
            }
        }

        widths
    }

    /// Render the table to styled lines with Unicode borders.
    fn render(&self, terminal_width: usize) -> Vec<LineContent> {
        if self.rows.is_empty() {
            return Vec::new();
        }

        let mut lines = Vec::new();
        let col_widths = self.column_widths();

        // Calculate total width and compress if needed
        let total_width: usize = col_widths.iter().sum::<usize>() + (col_widths.len() + 1); // borders
        let compressed_widths = if total_width > terminal_width {
            self.compress_column_widths(&col_widths, terminal_width)
        } else {
            col_widths
        };

        // Top border
        lines.push(self.render_border("┌", "┬", "┐", &compressed_widths));

        // Header row (if exists)
        let start_row = if self.has_header && !self.rows.is_empty() {
            lines.push(self.render_row(&self.rows[0], &compressed_widths, true));
            lines.push(self.render_border("├", "┼", "┤", &compressed_widths));
            1
        } else {
            0
        };

        // Data rows
        for (i, row) in self.rows.iter().enumerate().skip(start_row) {
            lines.push(self.render_row(row, &compressed_widths, false));
        }

        // Bottom border
        lines.push(self.render_border("└", "┴", "┘", &compressed_widths));

        lines
    }

    fn compress_column_widths(&self, widths: &[usize], max_width: usize) -> Vec<usize> {
        let current_total: usize = widths.iter().sum::<usize>() + widths.len() + 1;
        if current_total <= max_width {
            return widths.to_vec();
        }

        // Proportionally compress
        let available = max_width.saturating_sub(widths.len() + 1);
        let total_content: usize = widths.iter().sum();

        if total_content == 0 {
            return widths.to_vec();
        }

        widths
            .iter()
            .map(|&w| {
                let proportion = w as f64 / total_content as f64;
                (proportion * available as f64).floor() as usize
            })
            .collect()
    }

    fn render_border(&self, left: &str, middle: &str, right: &str, widths: &[usize]) -> LineContent {
        let mut segments: Vec<StyledSpan> = Vec::new();
        segments.push(StyledSpan::new(left.to_string(), Style::default().fg(Color::DarkGray)));

        for (i, &width) in widths.iter().enumerate() {
            segments.push(StyledSpan::new("─".repeat(width), Style::default().fg(Color::DarkGray)));
            if i < widths.len() - 1 {
                segments.push(StyledSpan::new(middle.to_string(), Style::default().fg(Color::DarkGray)));
            }
        }

        segments.push(StyledSpan::new(right.to_string(), Style::default().fg(Color::DarkGray)));
        LineContent::Styled(segments)
    }

    fn render_row(&self, row: &[String], widths: &[usize], is_header: bool) -> LineContent {
        let mut segments: Vec<StyledSpan> = Vec::new();
        let style = if is_header {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        segments.push(StyledSpan::new("│".to_string(), Style::default().fg(Color::DarkGray)));

        for (i, &width) in widths.iter().enumerate() {
            let cell = row.get(i).map(|s| s.as_str()).unwrap_or("");
            let cell_width = UnicodeWidthStr::width(cell);

            if cell_width > width {
                // Truncate
                let truncated = self.truncate_to_width(cell, width.saturating_sub(3));
                segments.push(StyledSpan::new(format!(" {} ", truncated), style));
            } else {
                let padding = width.saturating_sub(cell_width);
                segments.push(StyledSpan::new(format!(" {}{} ", cell, " ".repeat(padding)), style));
            }

            if i < widths.len() - 1 {
                segments.push(StyledSpan::new("│".to_string(), Style::default().fg(Color::DarkGray)));
            }
        }

        segments.push(StyledSpan::new("│".to_string(), Style::default().fg(Color::DarkGray)));
        LineContent::Styled(segments)
    }

    fn truncate_to_width(&self, text: &str, max_width: usize) -> String {
        if max_width < 3 {
            return ".".repeat(max_width);
        }

        let mut width = 0;
        let mut result = String::new();

        for ch in text.chars() {
            let char_width = UnicodeWidthStr::width(ch.to_string().as_str());
            if width + char_width > max_width - 3 {
                break;
            }
            result.push(ch);
            width += char_width;
        }

        result.push_str("...");
        result
    }
}

/// Convert XHTML content to styled lines for terminal display.
pub fn xhtml_to_lines(
    html: &str,
    image_extractor: Option<&dyn Fn(&str) -> Option<std::path::PathBuf>>,
) -> Vec<LineContent> {
    let mut reader = Reader::from_str(html);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();

    let mut out: Vec<LineContent> = Vec::new();
    let mut cur: Vec<StyledSpan> = Vec::new();

    // Style state
    let mut bold = false;
    let mut italic = false;
    let mut in_code = false;
    let mut in_pre = false;
    let mut link = false;

    // Link tracking
    let mut link_url = String::new();
    let mut heading: Option<u8> = None;
    let mut list_depth: usize = 0;
    let mut in_blockquote = false;
    let mut image_count: usize = 0;

    // Track which tags we should ignore content of
    let mut skip_depth: usize = 0;

    // Table parsing state
    let mut in_table = false;
    let mut in_thead = false;
    let mut in_row = false;
    let mut in_cell = false;
    let mut current_table = Table::new();
    let mut current_cell_content = String::new();
    let mut cell_is_header = false;

    let push = |cur: &mut Vec<StyledSpan>, text: &str, style: Style, link_info: Option<LinkInfo>| {
        if !text.is_empty() {
            match link_info {
                Some(info) => cur.push(StyledSpan::with_link(text.to_string(), style, info)),
                None => cur.push(StyledSpan::new(text.to_string(), style)),
            }
        }
    };

    let current_style = |bold: bool, italic: bool, in_code: bool, link: bool, heading: Option<u8>| {
        let mut s = Style::default();
        if let Some(level) = heading {
            let color = match level {
                1 => Color::Yellow,
                2 => Color::Cyan,
                3 => Color::Green,
                4 => Color::Magenta,
                5 => Color::Blue,
                _ => Color::White,
            };
            s = s.fg(color).add_modifier(Modifier::BOLD);
            if level == 1 {
                s = s.add_modifier(Modifier::UNDERLINED);
            }
        }
        if in_code {
            s = s.fg(Color::Green).bg(Color::Black);
        }
        if link {
            s = s.fg(Color::Blue).add_modifier(Modifier::UNDERLINED);
        }
        if bold {
            s = s.add_modifier(Modifier::BOLD);
        }
        if italic {
            s = s.add_modifier(Modifier::ITALIC);
        }
        s
    };

    let flush = |out: &mut Vec<LineContent>, cur: &mut Vec<StyledSpan>| {
        if !cur.is_empty() {
            out.push(LineContent::Styled(std::mem::take(cur)));
        }
    };

    let list_indent = |depth: usize| " ".repeat(depth * 2 + 2);

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = e.local_name();
                let tag = std::str::from_utf8(local.as_ref()).unwrap_or("");

                if skip_depth > 0 && !matches!(tag, "head" | "style" | "script") {
                    buf.clear();
                    continue;
                }

                match tag {
                    "head" | "style" | "script" => {
                        skip_depth += 1;
                    }
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                        let level = tag[1..].parse::<u8>().unwrap_or(1);
                        let prefix = "#".repeat(level as usize) + " ";
                        heading = Some(level);
                        push(&mut cur, &prefix, current_style(bold, italic, in_code, link, heading), None);
                    }
                    "p" => {
                        if list_depth > 0 {
                            push(&mut cur, &list_indent(list_depth), Style::default(), None);
                        }
                        if in_blockquote {
                            push(&mut cur, "  ▎ ", Style::default().fg(Color::DarkGray), None);
                        }
                    }
                    "strong" | "b" => bold = true,
                    "em" | "i" => italic = true,
                    "a" => {
                        link = true;
                        // Capture href
                        for attr in e.attributes().flatten() {
                            if attr.key.local_name().as_ref() == b"href" {
                                link_url = String::from_utf8_lossy(&attr.value).into_owned();
                            }
                        }
                        // Add 🔗 prefix for inline link marker
                        let style = current_style(bold, italic, in_code, link, heading);
                        push(&mut cur, "🔗", style, None);
                    }
                    "code" if !in_pre => {
                        in_code = true;
                    }
                    "pre" => {
                        in_pre = true;
                        // Border top
                        push(&mut cur, &"─".repeat(8), Style::default().fg(Color::DarkGray), None);
                        flush(&mut out, &mut cur);
                    }
                    "ul" | "ol" => {
                        list_depth += 1;
                    }
                    "li" => {
                        let indent_str = list_indent(list_depth.saturating_sub(1));
                        push(&mut cur, &indent_str, Style::default(), None);
                        push(&mut cur, "• ", Style::default().fg(Color::Magenta), None);
                    }
                    "blockquote" => {
                        in_blockquote = true;
                        push(&mut cur, "  ▎ ", Style::default().fg(Color::DarkGray), None);
                    }
                    "img" => {
                        // img is usually self-closing (Empty), but handle Start too
                        let mut src = String::new();
                        let mut alt = String::new();
                        for attr in e.attributes().flatten() {
                            match attr.key.local_name().as_ref() {
                                b"src" => src = String::from_utf8_lossy(&attr.value).into_owned(),
                                b"alt" => alt = String::from_utf8_lossy(&attr.value).into_owned(),
                                _ => {}
                            }
                        }
                        if !src.is_empty() {
                            flush(&mut out, &mut cur);
                            let local_path = image_extractor.and_then(|f| f(&src));
                            let failed = local_path.is_none();
                            out.push(LineContent::Image(ImageNode {
                                alt,
                                url: src,
                                local_path,
                                id: image_count,
                                download_failed: failed,
                            }));
                            image_count += 1;
                        }
                    }
                    "table" => {
                        flush(&mut out, &mut cur);
                        in_table = true;
                        current_table = Table::new();
                    }
                    "thead" => {
                        in_thead = true;
                        current_table.has_header = true;
                    }
                    "tbody" => {}
                    "tr" => {
                        in_row = true;
                        current_table.add_row(Vec::new());
                    }
                    "th" | "td" => {
                        in_cell = true;
                        cell_is_header = (tag == "th");
                        current_cell_content.clear();
                    }
                    _ => {} // Unknown tags: ignore but keep processing content
                }
            }

            Ok(Event::Empty(ref e)) => {
                let local = e.local_name();
                let tag = std::str::from_utf8(local.as_ref()).unwrap_or("");
                if skip_depth > 0 {
                    buf.clear();
                    continue;
                }

                match tag {
                    "img" => {
                        let mut src = String::new();
                        let mut alt = String::new();
                        for attr in e.attributes().flatten() {
                            match attr.key.local_name().as_ref() {
                                b"src" => src = String::from_utf8_lossy(&attr.value).into_owned(),
                                b"alt" => alt = String::from_utf8_lossy(&attr.value).into_owned(),
                                _ => {}
                            }
                        }
                        if !src.is_empty() {
                            flush(&mut out, &mut cur);
                            let local_path = image_extractor.and_then(|f| f(&src));
                            let failed = local_path.is_none();
                            out.push(LineContent::Image(ImageNode {
                                alt,
                                url: src,
                                local_path,
                                id: image_count,
                                download_failed: failed,
                            }));
                            image_count += 1;
                        }
                    }
                    "br" => {
                        flush(&mut out, &mut cur);
                    }
                    "hr" => {
                        push(&mut cur, &"─".repeat(8), Style::default().fg(Color::DarkGray), None);
                        flush(&mut out, &mut cur);
                        out.push(LineContent::Styled(Vec::new()));
                    }
                    "item" if tag == "item" => {} // handled elsewhere
                    _ => {}
                }
            }

            Ok(Event::End(ref e)) => {
                let local = e.local_name();
                let tag = std::str::from_utf8(local.as_ref()).unwrap_or("");

                if skip_depth > 0 && matches!(tag, "head" | "style" | "script") {
                    skip_depth = skip_depth.saturating_sub(1);
                    buf.clear();
                    continue;
                }

                match tag {
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                        flush(&mut out, &mut cur);
                        out.push(LineContent::Styled(Vec::new()));
                        heading = None;
                    }
                    "p" => {
                        flush(&mut out, &mut cur);
                        out.push(LineContent::Styled(Vec::new()));
                    }
                    "strong" | "b" => bold = false,
                    "em" | "i" => italic = false,
                    "a" => {
                        link = false;
                    }
                    "code" if !in_pre => in_code = false,
                    "pre" => {
                        in_pre = false;
                        push(&mut cur, &"─".repeat(8), Style::default().fg(Color::DarkGray), None);
                        flush(&mut out, &mut cur);
                        out.push(LineContent::Styled(Vec::new()));
                    }
                    "ul" | "ol" => {
                        list_depth = list_depth.saturating_sub(1);
                        if list_depth == 0 {
                            out.push(LineContent::Styled(Vec::new()));
                        }
                    }
                    "li" => {
                        flush(&mut out, &mut cur);
                    }
                    "blockquote" => {
                        flush(&mut out, &mut cur);
                        in_blockquote = false;
                        out.push(LineContent::Styled(Vec::new()));
                    }
                    "table" => {
                        in_table = false;
                        // Render the table with a default terminal width of 80
                        let terminal_width = crossterm::terminal::size()
                            .map(|(w, _)| w as usize)
                            .unwrap_or(80);
                        let table_lines = current_table.render(terminal_width);
                        for line in table_lines {
                            out.push(line);
                        }
                        out.push(LineContent::Styled(Vec::new())); // Empty line after table
                    }
                    "thead" => {
                        in_thead = false;
                    }
                    "tbody" => {}
                    "tr" => {
                        in_row = false;
                    }
                    "th" | "td" => {
                        in_cell = false;
                        // Add the collected cell content to the current row
                        if let Some(row) = current_table.current_row_mut() {
                            row.push(current_cell_content.trim().to_string());
                        }
                    }
                    _ => {}
                }
            }

            Ok(Event::Text(ref e)) => {
                if skip_depth > 0 {
                    buf.clear();
                    continue;
                }
                let text = e.unescape().unwrap_or_default().to_string();
                // Skip pure whitespace between block elements
                let trimmed = text.trim();
                if trimmed.is_empty() && !in_pre && !in_code && !in_cell {
                    buf.clear();
                    continue;
                }

                // If inside a table cell, collect text for the cell
                if in_cell {
                    if in_cell {
                        current_cell_content.push_str(&text);
                    }
                    buf.clear();
                    continue;
                }

                // In pre/code block, handle newlines specially
                if in_pre || in_code {
                    // Split text by newlines and flush each line separately
                    let lines: Vec<&str> = text.split('\n').collect();
                    for (i, line) in lines.iter().enumerate() {
                        if !line.is_empty() {
                            let style = current_style(bold, italic, in_code, link, heading);
                            push(&mut cur, line, style, None);
                        }
                        // Flush after each line except the last (to avoid extra empty line)
                        if i < lines.len() - 1 {
                            flush(&mut out, &mut cur);
                        }
                    }
                } else {
                    let style = current_style(bold, italic, in_code, link, heading);
                    let link_info = if link {
                        Some(LinkInfo {
                            url: link_url.clone(),
                            is_external: crate::image::is_remote_url(&link_url),
                        })
                    } else {
                        None
                    };
                    push(&mut cur, &text, style, link_info);
                }
            }

            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }

    // Flush remaining
    if !cur.is_empty() {
        flush(&mut out, &mut cur);
    }

    out
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn styled_text(lines: &[LineContent]) -> Vec<String> {
        lines
            .iter()
            .filter_map(|lc| match lc {
                LineContent::Styled(spans) => {
                    Some(spans.iter().map(|s| s.text.as_str()).collect::<String>())
                }
                _ => None,
            })
            .collect()
    }

    fn images(lines: &[LineContent]) -> Vec<&ImageNode> {
        lines
            .iter()
            .filter_map(|lc| match lc {
                LineContent::Image(node) => Some(node),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn paragraph_basic() {
        let html = r#"<html><body><p>Hello world</p></body></html>"#;
        let lines = xhtml_to_lines(html, None);
        let text = styled_text(&lines);
        assert!(text.iter().any(|l| l.contains("Hello world")));
    }

    #[test]
    fn headings() {
        let html = r#"<html><body><h1>Title</h1><h2>Sub</h2></body></html>"#;
        let lines = xhtml_to_lines(html, None);
        let text = styled_text(&lines);
        assert!(text.iter().any(|l| l.contains("# Title")));
        assert!(text.iter().any(|l| l.contains("## Sub")));
    }

    #[test]
    fn bold_and_italic() {
        let html = r#"<html><body><p><strong>bold</strong> and <em>italic</em></p></body></html>"#;
        let lines = xhtml_to_lines(html, None);
        let text = styled_text(&lines);
        assert!(text.iter().any(|l| l.contains("bold")));
        assert!(text.iter().any(|l| l.contains("italic")));
    }

    #[test]
    fn lists() {
        let html = r#"<html><body><ul><li>Item 1</li><li>Item 2</li></ul></body></html>"#;
        let lines = xhtml_to_lines(html, None);
        let text = styled_text(&lines);
        assert!(text.iter().any(|l| l.contains("•")));
        assert!(text.iter().any(|l| l.contains("Item 1")));
    }

    #[test]
    fn code_block() {
        let html = r#"<html><body><pre><code>fn main() {}</code></pre></body></html>"#;
        let lines = xhtml_to_lines(html, None);
        let text = styled_text(&lines);
        assert!(text.iter().any(|l| l.contains("fn main()")));
        assert!(text.iter().any(|l| l.contains('─')));
    }

    #[test]
    fn links() {
        let html = r#"<html><body><p><a href="https://example.com">click</a></p></body></html>"#;
        let lines = xhtml_to_lines(html, None);
        let text = styled_text(&lines);
        assert!(text.iter().any(|l| l.contains("click")));
    }

    #[test]
    fn images_extracted() {
        let html = r#"<html><body><img src="photo.jpg" alt="A photo"/></body></html>"#;
        let lines = xhtml_to_lines(html, None);
        let imgs = images(&lines);
        assert_eq!(imgs.len(), 1);
        assert_eq!(imgs[0].alt, "A photo");
        assert_eq!(imgs[0].url, "photo.jpg");
    }

    #[test]
    fn image_with_extractor() {
        let html = r#"<html><body><img src="img.png" alt="test"/></body></html>"#;
        let extractor = |href: &str| -> Option<std::path::PathBuf> {
            if href == "img.png" {
                Some(std::path::PathBuf::from("/cached/img.png"))
            } else {
                None
            }
        };
        let lines = xhtml_to_lines(html, Some(&extractor));
        let imgs = images(&lines);
        assert_eq!(imgs.len(), 1);
        assert!(imgs[0].local_path.is_some());
        assert!(!imgs[0].download_failed);
    }

    #[test]
    fn image_extractor_returns_none_marks_failed() {
        let html = r#"<html><body><img src="missing.png" alt="gone"/></body></html>"#;
        let extractor = |_href: &str| -> Option<std::path::PathBuf> { None };
        let lines = xhtml_to_lines(html, Some(&extractor));
        let imgs = images(&lines);
        assert!(imgs[0].download_failed);
    }

    #[test]
    fn unknown_tags_preserve_text() {
        let html = r#"<html><body><ruby>漢</ruby><p>Normal</p></body></html>"#;
        let lines = xhtml_to_lines(html, None);
        let text = styled_text(&lines);
        assert!(text.iter().any(|l| l.contains("漢")));
        assert!(text.iter().any(|l| l.contains("Normal")));
    }

    #[test]
    fn head_and_style_ignored() {
        let html = r#"<html><head><title>Title</title><style>body{color:red}</style></head><body><p>Content</p></body></html>"#;
        let lines = xhtml_to_lines(html, None);
        let text = styled_text(&lines);
        assert!(!text.iter().any(|l| l.contains("color:red")));
        assert!(text.iter().any(|l| l.contains("Content")));
    }

    #[test]
    fn blockquote() {
        let html = r#"<html><body><blockquote>A quote</blockquote></body></html>"#;
        let lines = xhtml_to_lines(html, None);
        let text = styled_text(&lines);
        assert!(text.iter().any(|l| l.contains('▎')));
        assert!(text.iter().any(|l| l.contains("A quote")));
    }

    #[test]
    fn empty_body() {
        let html = r#"<html><body></body></html>"#;
        let lines = xhtml_to_lines(html, None);
        // Should not panic, may have empty lines
        assert!(lines.is_empty() || lines.iter().all(|l| match l {
            LineContent::Styled(s) => s.is_empty(),
            _ => true,
        }));
    }

    /// Extract inline link infos from styled spans.
    fn inline_link_infos(lines: &[LineContent]) -> Vec<&LinkInfo> {
        lines
            .iter()
            .filter_map(|lc| match lc {
                LineContent::Styled(spans) => Some(spans),
                _ => None,
            })
            .flat_map(|spans| spans.iter())
            .filter_map(|span| span.link.as_ref())
            .collect()
    }

    #[test]
    fn link_external() {
        let html = r#"<html><body><p><a href="https://example.com">visit</a></p></body></html>"#;
        let lines = xhtml_to_lines(html, None);
        let links = inline_link_infos(&lines);
        assert!(!links.is_empty());
        assert_eq!(links[0].url, "https://example.com");
        assert!(links[0].is_external);
    }

    #[test]
    fn link_internal() {
        let html = r#"<html><body><p><a href="chapter2.xhtml">next</a></p></body></html>"#;
        let lines = xhtml_to_lines(html, None);
        let links = inline_link_infos(&lines);
        assert!(!links.is_empty());
        assert!(!links[0].is_external);
    }

    #[test]
    fn link_has_link_icon_prefix() {
        let html = r#"<html><body><p><a href="https://example.com">visit</a></p></body></html>"#;
        let lines = xhtml_to_lines(html, None);
        let text = styled_text(&lines);
        assert!(text.iter().any(|l| l.contains("🔗")));
    }
}
