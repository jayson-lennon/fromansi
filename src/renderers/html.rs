use crate::{Color, StyledText};

impl StyledText {
    pub fn to_html(&self) -> String {
        let mut html = String::new();
        for segment in &self.segments {
            if segment.text.is_empty() {
                continue;
            }
            let mut classes = Vec::new();
            let mut inline_styles = Vec::new();

            // Handle colors, considering reverse
            let (fg_color, bg_color) = if segment.style.reverse {
                (
                    segment.style.bg_color.as_ref(),
                    segment.style.fg_color.as_ref(),
                )
            } else {
                (
                    segment.style.fg_color.as_ref(),
                    segment.style.bg_color.as_ref(),
                )
            };

            if let Some(color) = fg_color {
                match color {
                    Color::Indexed(idx) => classes.push(format!("fg-{}", idx)),
                    Color::Rgb(r, g, b) => {
                        inline_styles.push(format!("color: rgb({}, {}, {})", r, g, b))
                    }
                }
            }

            if let Some(color) = bg_color {
                match color {
                    Color::Indexed(idx) => classes.push(format!("bg-{}", idx)),
                    Color::Rgb(r, g, b) => {
                        inline_styles.push(format!("background-color: rgb({}, {}, {})", r, g, b))
                    }
                }
            }

            // Add style classes
            if segment.style.bold {
                classes.push("bold".to_string());
            }
            if segment.style.dim {
                classes.push("dim".to_string());
            }
            if segment.style.italic {
                classes.push("italic".to_string());
            }
            if segment.style.underline {
                classes.push("underline".to_string());
            }
            if segment.style.blink {
                classes.push("blink".to_string());
            }
            if segment.style.strikethrough {
                classes.push("strikethrough".to_string());
            }
            if segment.style.hidden {
                classes.push("hidden".to_string());
            }

            // Build span
            let class_attr = if classes.is_empty() {
                String::new()
            } else {
                format!(" class=\"{}\"", classes.join(" "))
            };

            let style_attr = if inline_styles.is_empty() {
                String::new()
            } else {
                format!(" style=\"{}\"", inline_styles.join("; "))
            };

            let text = &segment.text;
            html.push_str(&format!(
                "<span{}{}>{}</span>",
                class_attr, style_attr, text
            ));
        }
        html
    }
}

#[cfg(test)]
mod tests {
    use crate::{Segment, Style};

    use super::*;

    #[test]
    fn test_html_plain_text() {
        let styled_text = StyledText {
            segments: vec![Segment {
                text: "Hello World".to_string(),
                style: Style::default(),
            }],
        };
        let html = styled_text.to_html();
        assert_eq!(html, "<span>Hello World</span>");
    }

    #[test]
    fn test_html_indexed_fg_color() {
        let styled_text = StyledText {
            segments: vec![Segment {
                text: "Red Text".to_string(),
                style: Style {
                    fg_color: Some(Color::Indexed(1)),
                    ..Default::default()
                },
            }],
        };
        let html = styled_text.to_html();
        assert_eq!(html, "<span class=\"fg-1\">Red Text</span>");
    }

    #[test]
    fn test_html_indexed_bg_color() {
        let styled_text = StyledText {
            segments: vec![Segment {
                text: "Blue BG".to_string(),
                style: Style {
                    bg_color: Some(Color::Indexed(4)),
                    ..Default::default()
                },
            }],
        };
        let html = styled_text.to_html();
        assert_eq!(html, "<span class=\"bg-4\">Blue BG</span>");
    }

    #[test]
    fn test_html_rgb_fg_color() {
        let styled_text = StyledText {
            segments: vec![Segment {
                text: "Custom Color".to_string(),
                style: Style {
                    fg_color: Some(Color::Rgb(255, 0, 128)),
                    ..Default::default()
                },
            }],
        };
        let html = styled_text.to_html();
        assert_eq!(
            html,
            "<span style=\"color: rgb(255, 0, 128)\">Custom Color</span>"
        );
    }

    #[test]
    fn test_html_rgb_bg_color() {
        let styled_text = StyledText {
            segments: vec![Segment {
                text: "Custom BG".to_string(),
                style: Style {
                    bg_color: Some(Color::Rgb(128, 255, 0)),
                    ..Default::default()
                },
            }],
        };
        let html = styled_text.to_html();
        assert_eq!(
            html,
            "<span style=\"background-color: rgb(128, 255, 0)\">Custom BG</span>"
        );
    }

    #[test]
    fn test_html_bold_style() {
        let styled_text = StyledText {
            segments: vec![Segment {
                text: "Bold Text".to_string(),
                style: Style {
                    bold: true,
                    ..Default::default()
                },
            }],
        };
        let html = styled_text.to_html();
        assert_eq!(html, "<span class=\"bold\">Bold Text</span>");
    }

    #[test]
    fn test_html_multiple_styles() {
        let styled_text = StyledText {
            segments: vec![Segment {
                text: "Styled Text".to_string(),
                style: Style {
                    bold: true,
                    italic: true,
                    underline: true,
                    ..Default::default()
                },
            }],
        };
        let html = styled_text.to_html();
        assert_eq!(
            html,
            "<span class=\"bold italic underline\">Styled Text</span>"
        );
    }

    #[test]
    fn test_html_reverse_colors() {
        let styled_text = StyledText {
            segments: vec![Segment {
                text: "Reversed".to_string(),
                style: Style {
                    fg_color: Some(Color::Indexed(1)),
                    bg_color: Some(Color::Indexed(7)),
                    reverse: true,
                    ..Default::default()
                },
            }],
        };
        let html = styled_text.to_html();
        assert_eq!(html, "<span class=\"fg-7 bg-1\">Reversed</span>");
    }

    #[test]
    fn test_html_multiple_segments() {
        let styled_text = StyledText {
            segments: vec![
                Segment {
                    text: "Normal".to_string(),
                    style: Style::default(),
                },
                Segment {
                    text: "Bold".to_string(),
                    style: Style {
                        bold: true,
                        ..Default::default()
                    },
                },
            ],
        };
        let html = styled_text.to_html();
        assert_eq!(html, "<span>Normal</span><span class=\"bold\">Bold</span>");
    }

    #[test]
    fn test_html_empty_segments_skipped() {
        let styled_text = StyledText {
            segments: vec![
                Segment {
                    text: "Text".to_string(),
                    style: Style::default(),
                },
                Segment {
                    text: "".to_string(),
                    style: Style {
                        bold: true,
                        ..Default::default()
                    },
                },
                Segment {
                    text: "More".to_string(),
                    style: Style::default(),
                },
            ],
        };
        let html = styled_text.to_html();
        assert_eq!(html, "<span>Text</span><span>More</span>");
    }
}
