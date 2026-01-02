use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Color {
    Indexed(u8),
    Rgb(u8, u8, u8),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Style {
    pub fg_color: Option<Color>,
    pub bg_color: Option<Color>,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub blink: bool,
    pub reverse: bool,
    pub hidden: bool,
    pub strikethrough: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    pub text: String,
    pub style: Style,
}

pub type ParsedData = Vec<Segment>;

static ANSI_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\x1b\[([0-9;]*)m").unwrap());

pub fn parse_ansi(input: &str) -> ParsedData {
    let mut segments = Vec::new();
    let mut current_style = Style::default();
    let mut last_end = 0;

    for cap in ANSI_REGEX.captures_iter(input) {
        let full_match = cap.get(0).unwrap();
        let params_str = cap.get(1).unwrap().as_str();

        // Add text before this escape
        let text_before = &input[last_end..full_match.start()];
        if !text_before.is_empty() {
            segments.push(Segment {
                text: text_before.to_string(),
                style: current_style.clone(),
            });
        }

        // Parse the parameters
        let params: Vec<u32> = if params_str.is_empty() {
            vec![0]
        } else {
            params_str
                .split(';')
                .filter_map(|s| s.parse().ok())
                .collect()
        };

        let mut i = 0;
        while i < params.len() {
            let param = params[i];
            match param {
                0 => current_style = Style::default(), // reset
                1 => current_style.bold = true,
                2 => current_style.dim = true,
                3 => current_style.italic = true,
                4 => current_style.underline = true,
                5 => current_style.blink = true,
                7 => current_style.reverse = true,
                8 => current_style.hidden = true,
                9 => current_style.strikethrough = true,
                22 => current_style.bold = false,
                23 => current_style.italic = false,
                24 => current_style.underline = false,
                25 => current_style.blink = false,
                27 => current_style.reverse = false,
                28 => current_style.hidden = false,
                29 => current_style.strikethrough = false,
                30..=37 => current_style.fg_color = Some(Color::Indexed((param - 30) as u8)),
                40..=47 => current_style.bg_color = Some(Color::Indexed((param - 40) as u8)),
                90..=97 => current_style.fg_color = Some(Color::Indexed((param - 82) as u8)), // bright
                100..=107 => current_style.bg_color = Some(Color::Indexed((param - 92) as u8)), // bright
                38 => {
                    // Extended foreground color
                    i += 1;
                    if i >= params.len() { break; }
                    let sub = params[i];
                    if sub == 5 {
                        // 256 color
                        i += 1;
                        if i >= params.len() { break; }
                        current_style.fg_color = Some(Color::Indexed(params[i] as u8));
                    } else if sub == 2 {
                        // Truecolor
                        i += 1;
                        if i + 2 >= params.len() { break; }
                        current_style.fg_color = Some(Color::Rgb(params[i] as u8, params[i + 1] as u8, params[i + 2] as u8));
                        i += 2;
                    }
                }
                48 => {
                    // Extended background color
                    i += 1;
                    if i >= params.len() { break; }
                    let sub = params[i];
                    if sub == 5 {
                        // 256 color
                        i += 1;
                        if i >= params.len() { break; }
                        current_style.bg_color = Some(Color::Indexed(params[i] as u8));
                    } else if sub == 2 {
                        // Truecolor
                        i += 1;
                        if i + 2 >= params.len() { break; }
                        current_style.bg_color = Some(Color::Rgb(params[i] as u8, params[i + 1] as u8, params[i + 2] as u8));
                        i += 2;
                    }
                }
                _ => {} // ignore unknown
            }
            i += 1;
        }

        last_end = full_match.end();
    }

    // Add remaining text
    let remaining = &input[last_end..];
    if !remaining.is_empty() {
        segments.push(Segment {
            text: remaining.to_string(),
            style: current_style,
        });
    }

    segments
}

pub fn to_html(segments: &ParsedData) -> String {
    let mut html = String::from(
        "<html><head><meta charset=\"utf-8\"><style>pre { white-space: pre-wrap; } .bold { font-weight: bold; } .underline { text-decoration: underline; }</style></head><body><pre>",
    );
    for segment in segments {
        let style = get_inline_style(&segment.style);
        let classes = get_classes(&segment.style);
        let class_attr = if classes.is_empty() {
            String::new()
        } else {
            format!(" class=\"{}\"", classes.join(" "))
        };
        let style_attr = if style.is_empty() {
            String::new()
        } else {
            format!(" style=\"{}\"", style)
        };
        html.push_str(&format!(
            "<span{}{}>{}</span>",
            class_attr,
            style_attr,
            escape_html(&segment.text)
        ));
    }
    html.push_str("</pre></body></html>");
    html
}

fn get_classes(style: &Style) -> Vec<&'static str> {
    let mut classes = Vec::new();
    if style.bold {
        classes.push("bold");
    }
    if style.underline {
        classes.push("underline");
    }
    // add more if needed
    classes
}

fn get_inline_style(style: &Style) -> String {
    let mut styles = Vec::new();
    if let Some(color) = &style.fg_color {
        styles.push(format!("color: {}", color_to_css(color)));
    }
    if let Some(color) = &style.bg_color {
        styles.push(format!("background-color: {}", color_to_css(color)));
    }
    styles.join("; ")
}

fn color_to_css(color: &Color) -> String {
    match color {
        Color::Rgb(r, g, b) => format!("rgb({},{},{})", r, g, b),
        Color::Indexed(i) => {
            // Basic 16 colors, for simplicity
            match i {
                0 => "rgb(0,0,0)".to_string(),
                1 => "rgb(128,0,0)".to_string(),
                2 => "rgb(0,128,0)".to_string(),
                3 => "rgb(128,128,0)".to_string(),
                4 => "rgb(0,0,128)".to_string(),
                5 => "rgb(128,0,128)".to_string(),
                6 => "rgb(0,128,128)".to_string(),
                7 => "rgb(192,192,192)".to_string(),
                8 => "rgb(128,128,128)".to_string(),
                9 => "rgb(255,0,0)".to_string(),
                10 => "rgb(0,255,0)".to_string(),
                11 => "rgb(255,255,0)".to_string(),
                12 => "rgb(0,0,255)".to_string(),
                13 => "rgb(255,0,255)".to_string(),
                14 => "rgb(0,255,255)".to_string(),
                15 => "rgb(255,255,255)".to_string(),
                _ => "rgb(0,0,0)".to_string(), // default
            }
        }
    }
}

fn escape_html(s: &str) -> String {
    s.replace("&", "&").replace("<", "<").replace(">", ">")
}
