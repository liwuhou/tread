use quick_xml::events::Event;
use quick_xml::reader::Reader;
use ratatui::style::{Color, Modifier, Style};

use crate::image::{ImageNode, LineContent};

/// Convert XHTML content to styled lines for terminal display.
pub fn xhtml_to_lines(
    html: &str,
    image_extractor: Option<&dyn Fn(&str) -> Option<std::path::PathBuf>>,
) -> Vec<LineContent> {
    let mut reader = Reader::from_str(html);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();

    let mut out: Vec<LineContent> = Vec::new();
    let mut cur: Vec<(String, Style)> = Vec::new();

    // Style state
    let mut bold = false;
    let mut italic = false;
    let mut in_code = false;
    let mut in_pre = false;
    let mut link = false;

    // Link tracking
    let mut link_url = String::new();
    let mut link_text = String::new();
    let mut pending_links: Vec<crate::image::LinkNode> = Vec::new();
    let mut heading: Option<u8> = None;
    let mut list_depth: usize = 0;
    let mut in_blockquote = false;
    let mut image_count: usize = 0;

    // Track which tags we should ignore content of
    let mut skip_depth: usize = 0;

    let push = |cur: &mut Vec<(String, Style)>, text: &str, style: Style| {
        if !text.is_empty() {
            cur.push((text.to_string(), style));
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

    let flush = |out: &mut Vec<LineContent>, cur: &mut Vec<(String, Style)>| {
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
                        push(&mut cur, &prefix, current_style(bold, italic, in_code, link, heading));
                    }
                    "p" => {
                        if list_depth > 0 {
                            push(&mut cur, &list_indent(list_depth), Style::default());
                        }
                        if in_blockquote {
                            push(&mut cur, "  ▎ ", Style::default().fg(Color::DarkGray));
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
                        link_text.clear();
                    }
                    "code" if !in_pre => {
                        in_code = true;
                    }
                    "pre" => {
                        in_pre = true;
                        // Border top
                        push(&mut cur, &"─".repeat(8), Style::default().fg(Color::DarkGray));
                        flush(&mut out, &mut cur);
                    }
                    "ul" | "ol" => {
                        list_depth += 1;
                    }
                    "li" => {
                        let indent_str = list_indent(list_depth.saturating_sub(1));
                        push(&mut cur, &indent_str, Style::default());
                        push(&mut cur, "• ", Style::default().fg(Color::Magenta));
                    }
                    "blockquote" => {
                        in_blockquote = true;
                        push(&mut cur, "  ▎ ", Style::default().fg(Color::DarkGray));
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
                        push(&mut cur, &"─".repeat(8), Style::default().fg(Color::DarkGray));
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
                        // Emit any collected links as focusable items
                        for link_node in pending_links.drain(..) {
                            out.push(LineContent::Link(link_node));
                        }
                        out.push(LineContent::Styled(Vec::new()));
                    }
                    "strong" | "b" => bold = false,
                    "em" | "i" => italic = false,
                    "a" => {
                        link = false;
                        if !link_url.is_empty() {
                            let is_external = crate::image::is_remote_url(&link_url);
                            pending_links.push(crate::image::LinkNode {
                                text: link_text.clone(),
                                url: link_url.clone(),
                                is_external,
                            });
                        }
                    }
                    "code" if !in_pre => in_code = false,
                    "pre" => {
                        in_pre = false;
                        push(&mut cur, &"─".repeat(8), Style::default().fg(Color::DarkGray));
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
                if trimmed.is_empty() && !in_pre && !in_code {
                    buf.clear();
                    continue;
                }
                let style = current_style(bold, italic, in_code, link, heading);
                push(&mut cur, &text, style);
                if link {
                    link_text.push_str(&text);
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
                    Some(spans.iter().map(|(t, _)| t.as_str()).collect::<String>())
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

    fn link_nodes(lines: &[LineContent]) -> Vec<&crate::image::LinkNode> {
        lines
            .iter()
            .filter_map(|lc| match lc {
                LineContent::Link(node) => Some(node),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn link_external() {
        let html = r#"<html><body><p><a href="https://example.com">visit</a></p></body></html>"#;
        let lines = xhtml_to_lines(html, None);
        let links = link_nodes(&lines);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].text, "visit");
        assert_eq!(links[0].url, "https://example.com");
        assert!(links[0].is_external);
    }

    #[test]
    fn link_internal() {
        let html = r#"<html><body><p><a href="chapter2.xhtml">next</a></p></body></html>"#;
        let lines = xhtml_to_lines(html, None);
        let links = link_nodes(&lines);
        assert_eq!(links.len(), 1);
        assert!(!links[0].is_external);
    }
}
