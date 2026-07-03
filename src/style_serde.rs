use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

/// Serializable representation of a Style.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StyleData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fg: Option<ColorData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg: Option<ColorData>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub bold: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub italic: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub underline: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub strikethrough: bool,
}

fn is_false(v: &bool) -> bool {
    !*v
}

/// Serializable color representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColorData {
    Named(String),
    Rgb { r: u8, g: u8, b: u8 },
}

impl From<Color> for ColorData {
    fn from(c: Color) -> Self {
        match c {
            Color::Black => ColorData::Named("black".to_string()),
            Color::Red => ColorData::Named("red".to_string()),
            Color::Green => ColorData::Named("green".to_string()),
            Color::Yellow => ColorData::Named("yellow".to_string()),
            Color::Blue => ColorData::Named("blue".to_string()),
            Color::Magenta => ColorData::Named("magenta".to_string()),
            Color::Cyan => ColorData::Named("cyan".to_string()),
            Color::White => ColorData::Named("white".to_string()),
            Color::DarkGray => ColorData::Named("dark_gray".to_string()),
            Color::Gray => ColorData::Named("gray".to_string()),
            Color::LightRed => ColorData::Named("light_red".to_string()),
            Color::LightGreen => ColorData::Named("light_green".to_string()),
            Color::LightYellow => ColorData::Named("light_yellow".to_string()),
            Color::LightBlue => ColorData::Named("light_blue".to_string()),
            Color::LightMagenta => ColorData::Named("light_magenta".to_string()),
            Color::LightCyan => ColorData::Named("light_cyan".to_string()),
            Color::Rgb(r, g, b) => ColorData::Rgb { r, g, b },
            Color::Indexed(i) => ColorData::Named(format!("indexed({i})")),
            Color::Reset => ColorData::Named("reset".to_string()),
        }
    }
}

impl From<&ColorData> for Color {
    fn from(c: &ColorData) -> Self {
        match c {
            ColorData::Named(name) => match name.as_str() {
                "black" => Color::Black,
                "red" => Color::Red,
                "green" => Color::Green,
                "yellow" => Color::Yellow,
                "blue" => Color::Blue,
                "magenta" => Color::Magenta,
                "cyan" => Color::Cyan,
                "white" => Color::White,
                "dark_gray" => Color::DarkGray,
                "gray" => Color::Gray,
                "light_red" => Color::LightRed,
                "light_green" => Color::LightGreen,
                "light_yellow" => Color::LightYellow,
                "light_blue" => Color::LightBlue,
                "light_magenta" => Color::LightMagenta,
                "light_cyan" => Color::LightCyan,
                "reset" => Color::Reset,
                s if s.starts_with("indexed(") => {
                    let idx: u8 = s
                        .trim_start_matches("indexed(")
                        .trim_end_matches(')')
                        .parse()
                        .unwrap_or(0);
                    Color::Indexed(idx)
                }
                _ => Color::Reset,
            },
            ColorData::Rgb { r, g, b } => Color::Rgb(*r, *g, *b),
        }
    }
}

impl From<Style> for StyleData {
    fn from(s: Style) -> Self {
        StyleData {
            fg: s.fg.map(ColorData::from),
            bg: s.bg.map(ColorData::from),
            bold: s.add_modifier.contains(Modifier::BOLD),
            italic: s.add_modifier.contains(Modifier::ITALIC),
            underline: s.add_modifier.contains(Modifier::UNDERLINED),
            strikethrough: s.add_modifier.contains(Modifier::CROSSED_OUT),
        }
    }
}

impl From<&StyleData> for Style {
    fn from(s: &StyleData) -> Self {
        let mut style = Style::default();
        if let Some(ref fg) = s.fg {
            style = style.fg(Color::from(fg));
        }
        if let Some(ref bg) = s.bg {
            style = style.bg(Color::from(bg));
        }
        if s.bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        if s.italic {
            style = style.add_modifier(Modifier::ITALIC);
        }
        if s.underline {
            style = style.add_modifier(Modifier::UNDERLINED);
        }
        if s.strikethrough {
            style = style.add_modifier(Modifier::CROSSED_OUT);
        }
        style
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_style_serializes_to_empty() {
        let style = Style::default();
        let data = StyleData::from(style);
        let json = serde_json::to_string(&data).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn style_with_fg_serializes() {
        let style = Style::default().fg(Color::Red);
        let data = StyleData::from(style);
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("\"fg\""));
        assert!(json.contains("\"red\""));
    }

    #[test]
    fn style_with_bold_serializes() {
        let style = Style::default().add_modifier(Modifier::BOLD);
        let data = StyleData::from(style);
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("\"bold\":true"));
    }

    #[test]
    fn roundtrip_named_color() {
        let style = Style::default().fg(Color::Cyan).bg(Color::DarkGray);
        let data = StyleData::from(style);
        let json = serde_json::to_string(&data).unwrap();
        let loaded: StyleData = serde_json::from_str(&json).unwrap();
        let restored = Style::from(&loaded);
        assert_eq!(restored.fg, Some(Color::Cyan));
        assert_eq!(restored.bg, Some(Color::DarkGray));
    }

    #[test]
    fn roundtrip_rgb_color() {
        let style = Style::default().fg(Color::Rgb(255, 128, 0));
        let data = StyleData::from(style);
        let json = serde_json::to_string(&data).unwrap();
        let loaded: StyleData = serde_json::from_str(&json).unwrap();
        let restored = Style::from(&loaded);
        assert_eq!(restored.fg, Some(Color::Rgb(255, 128, 0)));
    }

    #[test]
    fn roundtrip_modifiers() {
        let style = Style::default()
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::ITALIC)
            .add_modifier(Modifier::UNDERLINED)
            .add_modifier(Modifier::CROSSED_OUT);
        let data = StyleData::from(style);
        let json = serde_json::to_string(&data).unwrap();
        let loaded: StyleData = serde_json::from_str(&json).unwrap();
        let restored = Style::from(&loaded);
        assert!(restored.add_modifier.contains(Modifier::BOLD));
        assert!(restored.add_modifier.contains(Modifier::ITALIC));
        assert!(restored.add_modifier.contains(Modifier::UNDERLINED));
        assert!(restored.add_modifier.contains(Modifier::CROSSED_OUT));
    }

    #[test]
    fn roundtrip_complex_style() {
        let style = Style::default()
            .fg(Color::Yellow)
            .bg(Color::Black)
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::UNDERLINED);
        let data = StyleData::from(style);
        let json = serde_json::to_string(&data).unwrap();
        let loaded: StyleData = serde_json::from_str(&json).unwrap();
        let restored = Style::from(&loaded);
        assert_eq!(restored.fg, Some(Color::Yellow));
        assert_eq!(restored.bg, Some(Color::Black));
        assert!(restored.add_modifier.contains(Modifier::BOLD));
        assert!(restored.add_modifier.contains(Modifier::UNDERLINED));
    }
}
