use codepage_437::CP437_WINGDINGS;
use regex::Regex;
use rexpaint::XpFile;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

mod renderers;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Color {
    Indexed(u8),
    Rgb(u8, u8, u8),
}

impl Color {
    pub fn to_hex(&self) -> String {
        match self {
            Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
            Color::Indexed(idx) => {
                if *idx < 16 {
                    let standard_colors = [
                        "#000000", "#800000", "#008000", "#808000", "#000080", "#800080",
                        "#008080", "#c0c0c0", "#808080", "#ff0000", "#00ff00", "#ffff00",
                        "#0000ff", "#ff00ff", "#00ffff", "#ffffff",
                    ];
                    standard_colors[*idx as usize].to_string()
                } else if *idx < 232 {
                    let i = *idx as usize - 16;
                    let r = (i / 36) * 51;
                    let g = ((i % 36) / 6) * 51;
                    let b = (i % 6) * 51;
                    format!("#{:02x}{:02x}{:02x}", r, g, b)
                } else {
                    let gray = 8 + (*idx as usize - 232) * 10;
                    format!("#{:02x}{:02x}{:02x}", gray, gray, gray)
                }
            }
        }
    }

    pub fn to_indexed_if_possible(&self) -> Option<u8> {
        let hex = self.to_hex();
        (0..=255).find(|&i| Color::Indexed(i).to_hex() == hex)
    }
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyledText {
    pub segments: Vec<Segment>,
}

impl StyledText {
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    pub fn split_lines(&self) -> Vec<StyledText> {
        let mut lines = Vec::new();
        let mut current_line = Vec::new();

        for segment in &self.segments {
            let parts: Vec<&str> = segment.text.split('\n').collect();
            for (i, part) in parts.iter().enumerate() {
                if !part.is_empty() {
                    current_line.push(Segment {
                        text: part.to_string(),
                        style: segment.style.clone(),
                    });
                }
                if i < parts.len() - 1 {
                    // end of line
                    lines.push(StyledText {
                        segments: current_line,
                    });
                    current_line = Vec::new();
                }
            }
        }
        if !current_line.is_empty() {
            lines.push(StyledText {
                segments: current_line,
            });
        }
        lines
    }
}

pub type ParsedData = StyledText;

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
                    if i >= params.len() {
                        break;
                    }
                    let sub = params[i];
                    if sub == 5 {
                        // 256 color
                        i += 1;
                        if i >= params.len() {
                            break;
                        }
                        current_style.fg_color = Some(Color::Indexed(params[i] as u8));
                    } else if sub == 2 {
                        // Truecolor
                        i += 1;
                        if i + 2 >= params.len() {
                            break;
                        }
                        current_style.fg_color = Some(Color::Rgb(
                            params[i] as u8,
                            params[i + 1] as u8,
                            params[i + 2] as u8,
                        ));
                        i += 2;
                    }
                }
                48 => {
                    // Extended background color
                    i += 1;
                    if i >= params.len() {
                        break;
                    }
                    let sub = params[i];
                    if sub == 5 {
                        // 256 color
                        i += 1;
                        if i >= params.len() {
                            break;
                        }
                        current_style.bg_color = Some(Color::Indexed(params[i] as u8));
                    } else if sub == 2 {
                        // Truecolor
                        i += 1;
                        if i + 2 >= params.len() {
                            break;
                        }
                        current_style.bg_color = Some(Color::Rgb(
                            params[i] as u8,
                            params[i + 1] as u8,
                            params[i + 2] as u8,
                        ));
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

    StyledText { segments }
}

pub fn rexpaint_to_ansi(data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    use std::io::Cursor;
    let mut f = Cursor::new(data);
    let xp = XpFile::read(&mut f)?;
    let mut output = String::new();

    for layer in &xp.layers {
        for y in 0..layer.height {
            for x in 0..layer.width {
                let cell = layer.get(x, y).unwrap();
                let ch = if cell.ch != 0 {
                    CP437_WINGDINGS.decode(cell.ch as u8)
                } else {
                    ' '
                };
                if cell.bg.is_transparent() {
                    output.push_str(&format!(
                        "\x1b[38;2;{};{};{}m{}\x1b[0m",
                        cell.fg.r, cell.fg.g, cell.fg.b, ch
                    ));
                } else {
                    output.push_str(&format!(
                        "\x1b[38;2;{};{};{};48;2;{};{};{}m{}\x1b[0m",
                        cell.fg.r, cell.fg.g, cell.fg.b, cell.bg.r, cell.bg.g, cell.bg.b, ch
                    ));
                }
            }
            output.push('\n');
        }
    }
    Ok(output)
}

pub fn generate_css() -> String {
    let mut css = String::new();

    // Header comment
    css.push_str("/* ANSI Color Styles for fromansi HTML output */\n\n");

    // Text styles
    css.push_str(".bold { font-weight: bold; }\n");
    css.push_str(".italic { font-style: italic; }\n");
    css.push_str(".underline { text-decoration: underline; }\n");
    css.push_str(".strikethrough { text-decoration: line-through; }\n");
    css.push_str(".dim { opacity: 0.5; }\n");
    css.push_str(".blink { animation: blink 1s infinite; }\n");
    css.push_str("@keyframes blink { 0%, 50% { opacity: 1; } 51%, 100% { opacity: 0; } }\n");
    css.push_str(
        ".reverse { /* Note: reverse is handled by swapping fg/bg in HTML generation */ }\n",
    );
    css.push_str(".hidden { visibility: hidden; }\n\n");

    // Standard 16 colors
    let standard_colors = [
        "#000000", "#800000", "#008000", "#808000", "#000080", "#800080", "#008080", "#c0c0c0",
        "#808080", "#ff0000", "#00ff00", "#ffff00", "#0000ff", "#ff00ff", "#00ffff", "#ffffff",
    ];

    (0..16).for_each(|i| {
        css.push_str(&format!(".fg{} {{ color: {}; }}\n", i, standard_colors[i]));
        css.push_str(&format!(
            ".bg{} {{ background-color: {}; }}\n",
            i, standard_colors[i]
        ));
    });
    css.push('\n');

    // Color cube 16-231
    for i in 16..232 {
        let r = ((i - 16) / 36) * 51;
        let g = (((i - 16) % 36) / 6) * 51;
        let b = ((i - 16) % 6) * 51;
        let hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
        css.push_str(&format!(".fg{} {{ color: {}; }}\n", i, hex));
        css.push_str(&format!(".bg{} {{ background-color: {}; }}\n", i, hex));
    }
    css.push('\n');

    // Grayscale 232-255
    for i in 232..256 {
        let gray = 8 + (i - 232) * 10;
        let hex = format!("#{:02x}{:02x}{:02x}", gray, gray, gray);
        css.push_str(&format!(".fg{} {{ color: {}; }}\n", i, hex));
        css.push_str(&format!(".bg{} {{ background-color: {}; }}\n", i, hex));
    }

    css
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_fg_color() {
        let input = "\x1b[31mRed\x1b[0m";
        let result = parse_ansi(input);
        let expected = StyledText {
            segments: vec![Segment {
                text: "Red".to_string(),
                style: Style {
                    fg_color: Some(Color::Indexed(1)),
                    ..Default::default()
                },
            }],
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_basic_bg_color() {
        let input = "\x1b[41mRed BG\x1b[0m";
        let result = parse_ansi(input);
        let expected = StyledText {
            segments: vec![Segment {
                text: "Red BG".to_string(),
                style: Style {
                    bg_color: Some(Color::Indexed(1)),
                    ..Default::default()
                },
            }],
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_basic_fg_bg_color() {
        let input = "\x1b[32;44mGreen on Blue\x1b[0m";
        let result = parse_ansi(input);
        let expected = StyledText {
            segments: vec![Segment {
                text: "Green on Blue".to_string(),
                style: Style {
                    fg_color: Some(Color::Indexed(2)),
                    bg_color: Some(Color::Indexed(4)),
                    ..Default::default()
                },
            }],
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_terminal_styles() {
        let input = "\x1b[1;3;4mBold Italic Underline\x1b[0m";
        let result = parse_ansi(input);
        let expected = StyledText {
            segments: vec![Segment {
                text: "Bold Italic Underline".to_string(),
                style: Style {
                    bold: true,
                    italic: true,
                    underline: true,
                    ..Default::default()
                },
            }],
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_indexed_fg_color() {
        let input = "\x1b[38;5;196mBright Red\x1b[0m";
        let result = parse_ansi(input);
        let expected = StyledText {
            segments: vec![Segment {
                text: "Bright Red".to_string(),
                style: Style {
                    fg_color: Some(Color::Indexed(196)),
                    ..Default::default()
                },
            }],
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_indexed_bg_color() {
        let input = "\x1b[48;5;200mMagenta BG\x1b[0m";
        let result = parse_ansi(input);
        let expected = StyledText {
            segments: vec![Segment {
                text: "Magenta BG".to_string(),
                style: Style {
                    bg_color: Some(Color::Indexed(200)),
                    ..Default::default()
                },
            }],
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_true_color_fg() {
        let input = "\x1b[38;2;255;0;0mTrue Red\x1b[0m";
        let result = parse_ansi(input);
        let expected = StyledText {
            segments: vec![Segment {
                text: "True Red".to_string(),
                style: Style {
                    fg_color: Some(Color::Rgb(255, 0, 0)),
                    ..Default::default()
                },
            }],
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_true_color_bg() {
        let input = "\x1b[48;2;0;255;128mCyan BG\x1b[0m";
        let result = parse_ansi(input);
        let expected = StyledText {
            segments: vec![Segment {
                text: "Cyan BG".to_string(),
                style: Style {
                    bg_color: Some(Color::Rgb(0, 255, 128)),
                    ..Default::default()
                },
            }],
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_mixed_styles_and_colors() {
        let input = "\x1b[1;38;2;255;165;0;48;5;0mOrange on Black\x1b[0m";
        let result = parse_ansi(input);
        let expected = StyledText {
            segments: vec![Segment {
                text: "Orange on Black".to_string(),
                style: Style {
                    bold: true,
                    fg_color: Some(Color::Rgb(255, 165, 0)),
                    bg_color: Some(Color::Indexed(0)),
                    ..Default::default()
                },
            }],
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_rexpaint_to_ansi_conversion() {
        let xp_data = include_bytes!("test-dedup.xp");
        let ansi = rexpaint_to_ansi(xp_data).unwrap();
        let actual_bytes = ansi.as_bytes();

        let hex = "5b1b3833323b303b303b303b206d5b1b6d305b1b3833323b303b383b3b3937313b383834323b303b383b3b3937316d381b20305b1b6d335b3b383b3235323b353b303b303834323b323b3535303b303b206d5b1b6d305b1b3833323b303b303b303b206d5b1b6d305b1b3833323b303b303b303b206d5b1b6d301b0a335b3b383b323b303b306d301b20305b1b6d335b3b383b323b303b306d301b20305b1b6d335b3b383b323b303b306d301b20305b1b6d335b3b383b323b303b306d301b20305b1b6d335b3b383b3230313b323b3030313b323834323b313b3230303b313b3230206d5b1b6d30000a";
        let mut expected_bytes = Vec::new();
        for i in (0..hex.len()).step_by(4) {
            let word_hex = &hex[i..i + 4];
            let word = u16::from_str_radix(word_hex, 16).unwrap();
            expected_bytes.push((word & 0xff) as u8);
            expected_bytes.push((word >> 8) as u8);
        }
        // Remove trailing null if present
        if expected_bytes.last() == Some(&0) {
            expected_bytes.pop();
        }

        assert_eq!(actual_bytes, expected_bytes.as_slice());
    }
}
