use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};

use crate::image::{ImageNode, LineContent, StyledSpan, LinkInfo};

/// Parse Markdown source into content lines (styled text + image nodes).
pub fn parse_markdown(source: &str) -> Vec<LineContent> {
    if source.trim().is_empty() {
        return Vec::new();
    }

    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(source, opts);

    let mut out: Vec<LineContent> = Vec::new();

    // Inline style state
    let mut bold = false;
    let mut italic = false;
    let mut strikethrough = false;
    let mut link = false;

    // Link tracking
    let mut link_url = String::new();

    // Block state
    let mut in_code_block = false;
    #[allow(unused_assignments)]
    let mut code_lang = String::new();
    let mut item_stack: Vec<usize> = Vec::new();
    let mut in_blockquote = false;
    let mut in_heading: Option<HeadingLevel> = None;

    // Image state
    let mut in_image = false;
    let mut image_url = String::new();
    let mut image_alt = String::new();
    let mut image_count: usize = 0;

    let mut cur: Vec<StyledSpan> = Vec::new();

    let push = |cur: &mut Vec<StyledSpan>, text: &str, style: Style, link_info: Option<LinkInfo>| {
        if !text.is_empty() {
            match link_info {
                Some(info) => cur.push(StyledSpan::with_link(text.to_string(), style, info)),
                None => cur.push(StyledSpan::new(text.to_string(), style)),
            }
        }
    };

    let current_style =
        |bold: bool, italic: bool, strikethrough: bool, link: bool, heading: Option<HeadingLevel>| {
            let mut s = Style::default();
            if let Some(level) = heading {
                let color = match level {
                    HeadingLevel::H1 => Color::Yellow,
                    HeadingLevel::H2 => Color::Cyan,
                    HeadingLevel::H3 => Color::Green,
                    HeadingLevel::H4 => Color::Magenta,
                    HeadingLevel::H5 => Color::Blue,
                    HeadingLevel::H6 => Color::White,
                };
                s = s.fg(color).add_modifier(Modifier::BOLD);
                if let HeadingLevel::H1 = level {
                    s = s.add_modifier(Modifier::UNDERLINED);
                }
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
            if strikethrough {
                s = s.add_modifier(Modifier::CROSSED_OUT);
            }
            s
        };

    let terminal_width = crossterm::terminal::size().map(|(w, _)| w as usize).unwrap_or(80);
    let max_width = terminal_width.max(10);

    let flush_line = |out: &mut Vec<LineContent>, cur: &mut Vec<StyledSpan>| {
        if !cur.is_empty() {
            out.push(LineContent::Styled(std::mem::take(cur)));
        }
    };

    for event in parser {
        match event {
            // ── Images ───────────────────────────────────────────────────────
            Event::Start(Tag::Image { dest_url, .. }) => {
                in_image = true;
                image_url = dest_url.to_string();
                image_alt.clear();
            }
            Event::End(TagEnd::Image) => {
                in_image = false;
                // Flush any pending text first
                flush_line(&mut out, &mut cur);
                // Create image node
                let node = ImageNode {
                    alt: image_alt.clone(),
                    url: image_url.clone(),
                    local_path: None, // resolved later by App
                    id: image_count,
                    download_failed: false,
                };
                image_count += 1;
                out.push(LineContent::Image(node));
                out.push(LineContent::Styled(Vec::new())); // blank line after image
            }

            // ── Inline styles ────────────────────────────────────────────────
            Event::Start(Tag::Paragraph) => {
                let indent = item_stack.last().copied().unwrap_or(0);
                if in_blockquote {
                    let bq_style = Style::default().fg(Color::DarkGray);
                    push(&mut cur, "  ▎ ", bq_style, None);
                }
                cur.extend(apply_indent(indent));
            }
            Event::End(TagEnd::Paragraph) => {
                flush_line(&mut out, &mut cur);
                out.push(LineContent::Styled(Vec::new()));
            }

            Event::Start(Tag::Heading { level, .. }) => {
                let prefix = "#".repeat(level as usize) + " ";
                in_heading = Some(level);
                let style = current_style(true, false, false, false, in_heading);
                push(&mut cur, &prefix, style, None);
            }
            Event::End(TagEnd::Heading(_)) => {
                flush_line(&mut out, &mut cur);
                out.push(LineContent::Styled(Vec::new()));
                in_heading = None;
            }

            Event::Start(Tag::Emphasis) => italic = true,
            Event::End(TagEnd::Emphasis) => italic = false,
            Event::Start(Tag::Strong) => bold = true,
            Event::End(TagEnd::Strong) => bold = false,
            Event::Start(Tag::Strikethrough) => strikethrough = true,
            Event::End(TagEnd::Strikethrough) => strikethrough = false,
            Event::Start(Tag::Link { dest_url, .. }) => {
                link = true;
                link_url = dest_url.to_string();
                // Add 🔗 prefix for inline link marker
                let style = current_style(bold, italic, strikethrough, link, in_heading);
                push(&mut cur, "🔗", style, None);
            }
            Event::End(TagEnd::Link) => {
                link = false;
            }

            Event::Code(code_text) => {
                let style = Style::default().fg(Color::Green).bg(Color::Black);
                push(&mut cur, &format!(" {code_text} "), style, None);
            }

            Event::Html(html) | Event::InlineHtml(html) => {
                let style = current_style(bold, italic, strikethrough, link, in_heading);
                let link_info = if link {
                    Some(LinkInfo {
                        url: link_url.clone(),
                        is_external: crate::image::is_remote_url(&link_url),
                    })
                } else {
                    None
                };
                push(&mut cur, html.as_ref(), style, link_info);
            }

            Event::Text(text) => {
                if in_image {
                    image_alt.push_str(text.as_ref());
                } else if link {
                    let style = current_style(bold, italic, strikethrough, link, in_heading);
                    let link_info = Some(LinkInfo {
                        url: link_url.clone(),
                        is_external: crate::image::is_remote_url(&link_url),
                    });
                    push(&mut cur, text.as_ref(), style, link_info);
                } else if in_code_block {
                    let style = Style::default().fg(Color::Green).bg(Color::Black);
                    for line in text.as_ref().split('\n') {
                        if !line.is_empty() || text.ends_with('\n') {
                            push(&mut cur, line, style, None);
                            flush_line(&mut out, &mut cur);
                        }
                    }
                } else {
                    let style = current_style(bold, italic, strikethrough, link, in_heading);
                    push(&mut cur, text.as_ref(), style, None);
                }
            }

            Event::SoftBreak | Event::HardBreak => {
                flush_line(&mut out, &mut cur);
                let indent = item_stack.last().copied().unwrap_or(0);
                if in_blockquote {
                    let bq_style = Style::default().fg(Color::DarkGray);
                    push(&mut cur, "  ▎ ", bq_style, None);
                }
                cur.extend(apply_indent(indent));
            }

            // ── Block structures ─────────────────────────────────────────────
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                code_lang = match kind {
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                if !code_lang.is_empty() {
                    let style = Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC);
                    push(&mut cur, &format!("  ── {code_lang} "), style, None);
                    flush_line(&mut out, &mut cur);
                }
                let style = Style::default().fg(Color::DarkGray);
                push(&mut cur, &"─".repeat(max_width.min(8)), style, None);
                flush_line(&mut out, &mut cur);
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                let style = Style::default().fg(Color::DarkGray);
                push(&mut cur, &"─".repeat(max_width.min(8)), style, None);
                flush_line(&mut out, &mut cur);
                out.push(LineContent::Styled(Vec::new()));
            }

            Event::Start(Tag::List(_)) => {
                flush_line(&mut out, &mut cur);
            }
            Event::End(TagEnd::List(_)) => {
                if item_stack.is_empty() {
                    out.push(LineContent::Styled(Vec::new()));
                }
            }

            Event::Start(Tag::Item) => {
                let indent = item_stack.len() * 2 + 2;
                let style = Style::default().fg(Color::Magenta);
                let indent_str = " ".repeat(indent);
                push(&mut cur, &indent_str, Style::default(), None);
                push(&mut cur, "• ", style, None);
                item_stack.push(indent);
            }
            Event::End(TagEnd::Item) => {
                flush_line(&mut out, &mut cur);
                item_stack.pop();
            }

            Event::Start(Tag::BlockQuote(_)) => {
                in_blockquote = true;
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                flush_line(&mut out, &mut cur);
                in_blockquote = false;
                out.push(LineContent::Styled(Vec::new()));
            }

            Event::Rule => {
                let style = Style::default().fg(Color::DarkGray);
                push(&mut cur, &"─".repeat(max_width.min(8)), style, None);
                flush_line(&mut out, &mut cur);
                out.push(LineContent::Styled(Vec::new()));
            }

            // Tables
            Event::Start(Tag::Table(_)) => {
                let style = Style::default().fg(Color::DarkGray);
                push(&mut cur, &"─".repeat(max_width.min(8)), style, None);
                flush_line(&mut out, &mut cur);
            }
            Event::End(TagEnd::Table) => {
                let style = Style::default().fg(Color::DarkGray);
                push(&mut cur, &"─".repeat(max_width.min(8)), style, None);
                flush_line(&mut out, &mut cur);
                out.push(LineContent::Styled(Vec::new()));
            }
            Event::Start(Tag::TableHead)
            | Event::End(TagEnd::TableHead)
            | Event::Start(Tag::TableRow)
            | Event::End(TagEnd::TableRow) => {}

            Event::Start(Tag::TableCell) => {
                let style = Style::default().fg(Color::White);
                push(&mut cur, " │ ", style, None);
            }
            Event::End(TagEnd::TableCell) => {}

            _ => {}
        }
    }

    if !cur.is_empty() {
        flush_line(&mut out, &mut cur);
    }

    out
}

fn apply_indent(indent: usize) -> Vec<StyledSpan> {
    if indent == 0 {
        Vec::new()
    } else {
        vec![StyledSpan::new(" ".repeat(indent), Style::default())]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::{Color, Modifier};

    /// Extract plain text from styled lines only (skip images).
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

    /// Get styled lines only.
    fn styled_lines(lines: &[LineContent]) -> Vec<&Vec<StyledSpan>> {
        lines
            .iter()
            .filter_map(|lc| match lc {
                LineContent::Styled(spans) => Some(spans),
                _ => None,
            })
            .collect()
    }

    /// Get image nodes only.
    fn image_nodes(lines: &[LineContent]) -> Vec<&ImageNode> {
        lines
            .iter()
            .filter_map(|lc| match lc {
                LineContent::Image(node) => Some(node),
                _ => None,
            })
            .collect()
    }

    fn line_has_modifier(line: &[StyledSpan], modifier: Modifier) -> bool {
        line.iter().any(|s| s.style.add_modifier.contains(modifier))
    }

    fn line_has_fg(line: &[StyledSpan], color: Color) -> bool {
        line.iter().any(|s| s.style.fg == Some(color))
    }

    // ── Existing tests (updated for LineContent) ────────────────────────────

    #[test]
    fn h1_has_yellow_bold_underline() {
        let lines = parse_markdown("# Hello");
        let sl = styled_lines(&lines);
        assert!(!sl.is_empty());
        let first = sl[0];
        assert!(plain(&lines)[0].contains("Hello"));
        assert!(line_has_fg(first, Color::Yellow));
        assert!(line_has_modifier(first, Modifier::BOLD));
        assert!(line_has_modifier(first, Modifier::UNDERLINED));
    }

    #[test]
    fn h2_has_cyan_bold() {
        let lines = parse_markdown("## Subtitle");
        let first = styled_lines(&lines)[0];
        assert!(plain(&lines)[0].contains("Subtitle"));
        assert!(line_has_fg(first, Color::Cyan));
        assert!(line_has_modifier(first, Modifier::BOLD));
    }

    #[test]
    fn h3_has_green_bold() {
        let lines = parse_markdown("### Section");
        let first = styled_lines(&lines)[0];
        assert!(line_has_fg(first, Color::Green));
        assert!(line_has_modifier(first, Modifier::BOLD));
    }

    #[test]
    fn h4_has_magenta_bold() {
        let lines = parse_markdown("#### Sub");
        let first = styled_lines(&lines)[0];
        assert!(line_has_fg(first, Color::Magenta));
        assert!(line_has_modifier(first, Modifier::BOLD));
    }

    #[test]
    fn h5_has_blue_bold() {
        let lines = parse_markdown("##### Deep");
        let first = styled_lines(&lines)[0];
        assert!(line_has_fg(first, Color::Blue));
        assert!(line_has_modifier(first, Modifier::BOLD));
    }

    #[test]
    fn bold_text_has_bold_modifier() {
        let lines = parse_markdown("some **bold** text");
        let all_bold: Vec<_> = styled_lines(&lines)
            .into_iter()
            .flat_map(|l| l.iter())
            .filter(|span| span.text.contains("bold"))
            .collect();
        assert!(!all_bold.is_empty());
        assert!(all_bold.iter().any(|span| span.style.add_modifier.contains(Modifier::BOLD)));
    }

    #[test]
    fn italic_text_has_italic_modifier() {
        let lines = parse_markdown("some *italic* text");
        let all_it: Vec<_> = styled_lines(&lines)
            .into_iter()
            .flat_map(|l| l.iter())
            .filter(|span| span.text.contains("italic"))
            .collect();
        assert!(!all_it.is_empty());
        assert!(all_it.iter().any(|span| span.style.add_modifier.contains(Modifier::ITALIC)));
    }

    #[test]
    fn strikethrough_text_has_crossed_out_modifier() {
        let lines = parse_markdown("some ~~struck~~ text");
        let all_st: Vec<_> = styled_lines(&lines)
            .into_iter()
            .flat_map(|l| l.iter())
            .filter(|span| span.text.contains("struck"))
            .collect();
        assert!(!all_st.is_empty());
        assert!(all_st.iter().any(|span| span.style.add_modifier.contains(Modifier::CROSSED_OUT)));
    }

    #[test]
    fn inline_code_has_green_fg_black_bg() {
        let lines = parse_markdown("some `code` here");
        let code_spans: Vec<_> = styled_lines(&lines)
            .into_iter()
            .flat_map(|l| l.iter())
            .filter(|span| span.text.contains("code"))
            .collect();
        assert!(!code_spans.is_empty());
        assert!(code_spans.iter().any(|span| span.style.fg == Some(Color::Green)));
    }

    #[test]
    fn link_has_blue_underlined() {
        let lines = parse_markdown("[click](https://example.com)");
        let link_spans: Vec<_> = styled_lines(&lines)
            .into_iter()
            .flat_map(|l| l.iter())
            .filter(|span| span.text.contains("click"))
            .collect();
        assert!(!link_spans.is_empty());
        assert!(link_spans.iter().any(|span| span.style.fg == Some(Color::Blue)));
        assert!(link_spans.iter().any(|span| span.style.add_modifier.contains(Modifier::UNDERLINED)));
    }

    #[test]
    fn fenced_code_block_with_language() {
        let lines = parse_markdown("```rust\nlet x = 1;\n```");
        let text = plain(&lines);
        assert!(text.iter().any(|l| l.contains("rust")));
        assert!(text.iter().any(|l| l.contains("let x = 1;")));
        assert!(text.iter().any(|l| l.contains('─')));
    }

    #[test]
    fn fenced_code_block_without_language() {
        let lines = parse_markdown("```\nhello\n```");
        let text = plain(&lines);
        assert!(text.iter().any(|l| l.contains("hello")));
        assert!(text.iter().any(|l| l.contains('─')));
    }

    #[test]
    fn unordered_list_has_bullet_marker() {
        let lines = parse_markdown("- item one\n- item two");
        let text = plain(&lines);
        assert!(text.iter().any(|l| l.contains('•')));
        assert!(text.iter().any(|l| l.contains("item one")));
        assert!(text.iter().any(|l| l.contains("item two")));
    }

    #[test]
    fn nested_list_has_extra_indent() {
        let lines = parse_markdown("- outer\n  - inner");
        let text = plain(&lines);
        let inner_line = text.iter().find(|l| l.contains("inner")).unwrap();
        let outer_line = text.iter().find(|l| l.contains("outer")).unwrap();
        let outer_indent = outer_line.len() - outer_line.trim_start().len();
        let inner_indent = inner_line.len() - inner_line.trim_start().len();
        assert!(inner_indent > outer_indent,
            "outer={outer_line:?} (indent={outer_indent}), inner={inner_line:?} (indent={inner_indent})");
    }

    #[test]
    fn blockquote_has_prefix() {
        let lines = parse_markdown("> quoted text");
        let text = plain(&lines);
        assert!(text.iter().any(|l| l.contains('▎')));
        assert!(text.iter().any(|l| l.contains("quoted text")));
    }

    #[test]
    fn table_cells_separated_by_pipe() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |";
        let lines = parse_markdown(md);
        let text = plain(&lines);
        assert!(text.iter().any(|l| l.contains('│')));
        assert!(text.iter().any(|l| l.contains('A')));
        assert!(text.iter().any(|l| l.contains('1')));
    }

    #[test]
    fn horizontal_rule_renders_dashes() {
        let lines = parse_markdown("---");
        let text = plain(&lines);
        assert!(text.iter().any(|l| l.contains('─')));
    }

    #[test]
    fn empty_input_returns_empty_vec() {
        let lines = parse_markdown("");
        assert!(lines.is_empty());
    }

    // ── 3.x: Image tests ────────────────────────────────────────────────────

    #[test]
    fn image_with_alt_text() {
        let lines = parse_markdown("![公司 Logo](logo.png)");
        let imgs = image_nodes(&lines);
        assert_eq!(imgs.len(), 1);
        assert_eq!(imgs[0].alt, "公司 Logo");
        assert_eq!(imgs[0].url, "logo.png");
        assert_eq!(imgs[0].id, 0);
    }

    #[test]
    fn image_with_empty_alt() {
        let lines = parse_markdown("![](photo.jpg)");
        let imgs = image_nodes(&lines);
        assert_eq!(imgs.len(), 1);
        assert_eq!(imgs[0].alt, "");
        assert_eq!(imgs[0].url, "photo.jpg");
    }

    #[test]
    fn image_with_remote_url() {
        let lines = parse_markdown("![封面](https://example.com/cover.jpg)");
        let imgs = image_nodes(&lines);
        assert_eq!(imgs.len(), 1);
        assert_eq!(imgs[0].url, "https://example.com/cover.jpg");
    }

    #[test]
    fn multiple_images_get_unique_ids() {
        let lines = parse_markdown("![a](1.png)\n\n![b](2.png)\n\n![c](3.png)");
        let imgs = image_nodes(&lines);
        assert_eq!(imgs.len(), 3);
        assert_eq!(imgs[0].id, 0);
        assert_eq!(imgs[1].id, 1);
        assert_eq!(imgs[2].id, 2);
    }

    #[test]
    fn image_preserves_surrounding_text() {
        let lines = parse_markdown("前文\n\n![图](img.png)\n\n后文");
        let text = plain(&lines);
        assert!(text.iter().any(|l| l.contains("前文")));
        assert!(text.iter().any(|l| l.contains("后文")));
        assert_eq!(image_nodes(&lines).len(), 1);
    }

    fn link_nodes(lines: &[LineContent]) -> Vec<&crate::image::LinkNode> {
        lines
            .iter()
            .filter_map(|lc| match lc {
                LineContent::Link(node) => Some(node),
                _ => None,
            })
            .collect()
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
        let lines = parse_markdown("[click](https://example.com)");
        let links = inline_link_infos(&lines);
        assert!(!links.is_empty());
        assert_eq!(links[0].url, "https://example.com");
        assert!(links[0].is_external);
    }

    #[test]
    fn link_internal() {
        let lines = parse_markdown("[next](chapter2.xhtml#section)");
        let links = inline_link_infos(&lines);
        assert!(!links.is_empty());
        assert!(!links[0].is_external);
    }

    #[test]
    fn multiple_links_in_paragraph() {
        let lines = parse_markdown("See [a](url1) and [b](url2)");
        let links = inline_link_infos(&lines);
        assert!(links.len() >= 2);
    }

    #[test]
    fn link_has_link_icon_prefix() {
        let lines = parse_markdown("[click](https://example.com)");
        let text = plain(&lines);
        assert!(text.iter().any(|l| l.contains("🔗")));
    }
}
