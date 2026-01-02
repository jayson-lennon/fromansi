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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyledText {
    pub segments: Vec<Segment>,
}

impl StyledText {
    pub fn segments(&self) -> &[Segment] {
        &self.segments
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
}
