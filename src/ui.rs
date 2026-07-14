use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::App;
use crate::image::LineContent;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    if app.status_bar_visible {
        let content_height = area.height.saturating_sub(1);
        app.set_height(content_height);

        let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(area);

        draw_markdown(frame, app, chunks[0]);
        draw_status_bar(frame, app, chunks[1]);
    } else {
        app.set_height(area.height);
        draw_markdown(frame, app, area);
    }

    if app.help_visible {
        draw_help(frame, area);
    }
    if app.toc_visible && !app.toc.is_empty() {
        draw_toc(frame, app, area);
    }
}

fn draw_markdown(frame: &mut Frame, app: &App, area: Rect) {
    let h = app.height.unwrap_or(0) as usize;

    // Get current focus info
    let focused_item = app.focus_index.and_then(|f| app.focusable_positions.get(f));

    let visible: Vec<Line> = app
        .wrapped_lines
        .iter()
        .enumerate()
        .skip(app.scroll)
        .take(h)
        .map(|(idx, content)| match content {
            LineContent::Styled(spans) => {
                // Check if this line has a focused inline link range.
                let focused_range = focused_item.and_then(|item| item.inline_range_on_line(idx));

                let mut owned: Vec<Span> = Vec::new();
                let mut char_offset = 0;

                for span in spans {
                    let span_width = unicode_width::UnicodeWidthStr::width(span.text.as_str());
                    let span_end = char_offset + span_width;
                    let is_link_focused = span.link.is_some()
                        && focused_range
                            .map(|(start, end)| span_end > start && char_offset < end)
                            .unwrap_or(false);
                    let style = if is_link_focused {
                        // Focused link style
                        span.style
                            .fg(Color::White)
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD)
                            .add_modifier(Modifier::UNDERLINED)
                    } else {
                        span.style
                    };
                    owned.push(Span::styled(span.text.clone(), style));
                    char_offset = span_end;
                }

                Line::from(owned)
            }
            LineContent::Image(node) => {
                let is_focused = focused_item
                    .map(|item| item.is_entire_line_on_line(idx))
                    .unwrap_or(false);

                let alt_display = if node.alt.is_empty() {
                    "image".to_string()
                } else {
                    node.alt.clone()
                };

                let text = if node.local_path.is_none()
                    && crate::image::is_remote_url(&node.url)
                    && node.download_failed
                {
                    format!("[📷 {alt_display} ⚠ 下载失败]")
                } else {
                    format!("[📷 {alt_display}]")
                };

                let style = if is_focused {
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else if node.local_path.is_none()
                    && crate::image::is_remote_url(&node.url)
                    && node.download_failed
                {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::Cyan)
                };

                Line::from(vec![Span::styled(text, style)])
            }
            LineContent::Link(node) => {
                let is_focused = focused_item
                    .map(|item| item.is_entire_line_on_line(idx))
                    .unwrap_or(false);

                let display = if node.text.is_empty() {
                    node.url.clone()
                } else {
                    format!("🔗 {}", node.text)
                };

                let style = if is_focused {
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::UNDERLINED)
                } else {
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::UNDERLINED)
                };

                Line::from(vec![Span::styled(display, style)])
            }
        })
        .collect();

    let paragraph = Paragraph::new(visible);
    frame.render_widget(paragraph, area);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let line = status_bar_line(app, area.width);
    let bar = Paragraph::new(line).style(Style::default().bg(Color::DarkGray));
    frame.render_widget(bar, area);
}

fn status_bar_line(app: &App, area_width: u16) -> Line<'static> {
    let total = app.total_lines();
    let current = app.scroll + 1;
    let h = app.height.unwrap_or(0) as usize;
    let percent = if total > 0 {
        (app.scroll + h) * 100 / total
    } else {
        100
    };
    let percent = percent.min(100);

    let left = Span::styled(
        format!(" {} ", &app.filename),
        Style::default()
            .fg(Color::White)
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    let middle = Span::styled(
        format!(" {}/{} ", current, total),
        Style::default().fg(Color::White).bg(Color::DarkGray),
    );

    // Show status message if present, otherwise show help/quit hints
    let right_text = if let Some(msg) = &app.status_message {
        format!(" ⚠ {msg} ")
    } else if app.help_visible {
        " 按任意键关闭帮助 ".to_string()
    } else {
        format!(" {percent}% [?]帮助 [D]书架 [q]退出 ")
    };
    let right_style = if app.status_message.is_some() {
        Style::default().fg(Color::Yellow).bg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    };
    let right = Span::styled(right_text, right_style);

    let used = left.width() + middle.width() + right.width();
    let filler_len = (area_width as usize).saturating_sub(used);
    let filler = Span::styled(" ".repeat(filler_len), Style::default().bg(Color::DarkGray));

    Line::from(vec![left, middle, right, filler])
}

fn draw_toc(frame: &mut Frame, app: &App, area: Rect) {
    let height = (app.toc.len() as u16 + 4).min(area.height.saturating_sub(4));
    let popup = centered_rect(60, height, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" 目录 ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let items: Vec<Line> = app
        .toc
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let indent = "  ".repeat(entry.level.saturating_sub(1));
            let marker = if i == app.toc_selection { "▶ " } else { "  " };
            let style = if i == app.toc_selection {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            Line::from(vec![
                Span::styled(marker, style),
                Span::styled(format!("{indent}{}", entry.title), style),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(items).block(block);
    frame.render_widget(paragraph, popup);
}

fn draw_help(frame: &mut Frame, area: Rect) {
    let popup = centered_rect(50, 25, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" 快捷键 ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let help_lines = vec![
        Line::from(""),
        help_row("j / ↓", "下一行（底部自动下一章）"),
        help_row("k / ↑", "上一行（顶部自动上一章）"),
        help_row("Ctrl+d", "向下翻半页"),
        help_row("Ctrl+u", "向上翻半页"),
        help_row("Ctrl+f", "向下翻一页"),
        help_row("Ctrl+b", "向上翻一页"),
        help_row("PgDn/PgUp", "翻半页"),
        help_row("g / Home", "回到顶部"),
        help_row("G / End", "跳转底部"),
        help_row("Tab", "下一个可交互元素"),
        help_row("Shift+Tab", "上一个可交互元素"),
        help_row("Enter", "打开图片/链接"),
        help_row("Ctrl+n/p", "下/上一章 (EPUB)"),
        help_row("t", "目录 (EPUB)"),
        help_row("f", "切换状态栏"),
        help_row("?", "显示/关闭帮助"),
        help_row("D", "返回 dashboard"),
        help_row("q / Esc", "退出"),
    ];

    let paragraph = Paragraph::new(help_lines).block(block);
    frame.render_widget(paragraph, popup);
}

fn help_row<'a>(key: &'a str, desc: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(
            format!("{key:<12}"),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(desc, Style::default().fg(Color::White)),
    ])
}

fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let vert = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(height),
        Constraint::Fill(1),
    ])
    .split(area);

    let horiz = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Percentage(percent_x),
        Constraint::Fill(1),
    ])
    .split(vert[1]);

    horiz[1]
}

#[cfg(test)]
mod tests {
    use super::status_bar_line;
    use crate::{app::App, image::LineContent};
    use unicode_width::UnicodeWidthStr;

    fn render_text_width(line: &ratatui::text::Line<'_>) -> usize {
        let text: String = line.spans.iter().map(|span| span.content.as_ref()).collect();
        UnicodeWidthStr::width(text.as_str())
    }

    #[test]
    fn status_bar_fills_full_width_with_wide_characters() {
        let mut app = App::new(
            vec![LineContent::Styled(Vec::new()); 269],
            "LLM 编译知识库：如何让知识库持续积累和互联？".to_string(),
        );
        app.set_height(1);

        let line = status_bar_line(&app, 120);

        assert_eq!(render_text_width(&line), 120);
    }

    #[test]
    fn status_bar_fills_full_width_with_status_message() {
        let mut app = App::new(vec![LineContent::Styled(Vec::new()); 42], "test.md".to_string());
        app.set_height(10);
        app.status_message = Some("下载失败，请重试".to_string());

        let line = status_bar_line(&app, 80);

        assert_eq!(render_text_width(&line), 80);
    }
}
