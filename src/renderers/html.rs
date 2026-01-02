use crate::{Color, StyledText};

impl StyledText {
    pub fn to_html(&self) -> String {
        self.to_html_with_filter(None)
    }

    pub fn to_html_with_filter(&self, filter_hex: Option<&str>) -> String {
        if filter_hex.is_none() {
            // No filter, use original logic
            format!("<pre>{}</pre>", self.generate_html_spans(filter_hex))
        } else {
            // With filter, process line by line
            use regex::Regex;

            let lines = self.split_lines();
            let mut result = String::new();

            for line in lines {
                let mut line_html = line.generate_html_spans(filter_hex);

                // Trim trailing spans containing only &nbsp;
                let re = Regex::new(r"(<span[^>]*>(&nbsp;)+</span>\s*)+$").unwrap();
                line_html = re.replace_all(&line_html, "").to_string();

                if !line_html.is_empty() {
                    result.push_str(&line_html);
                    result.push('\n');
                }
            }

            if result.ends_with('\n') {
                result.pop();
            }

            format!("<pre>{}</pre>", result)
        }
    }

    fn generate_html_spans(&self, filter_hex: Option<&str>) -> String {
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

            // Check if segment should be filtered
            let fg_hex = fg_color.map(|c| c.to_hex());
            let is_filtered = match (fg_hex, filter_hex) {
                (Some(fh), Some(filt)) if fh == filt && segment.text.chars().all(|c| c == ' ') => true,
                _ => false,
            };

            // For filtered segments, don't apply styling
            let (final_classes, final_styles) = if is_filtered {
                (Vec::new(), Vec::new())
            } else {
                (classes, inline_styles)
            };

            // Build span
            let class_attr = if final_classes.is_empty() {
                String::new()
            } else {
                format!(" class=\"{}\"", final_classes.join(" "))
            };

            let style_attr = if final_styles.is_empty() {
                String::new()
            } else {
                format!(" style=\"{}\"", final_styles.join("; "))
            };

            let text = if is_filtered {
                "&nbsp;".repeat(segment.text.len())
            } else {
                segment.text.clone()
            };

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
        assert_eq!(html, "<pre><span>Hello World</span></pre>");
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
        assert_eq!(html, "<pre><span class=\"fg-1\">Red Text</span></pre>");
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
        assert_eq!(html, "<pre><span class=\"bg-4\">Blue BG</span></pre>");
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
            "<pre><span style=\"color: rgb(255, 0, 128)\">Custom Color</span></pre>"
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
            "<pre><span style=\"background-color: rgb(128, 255, 0)\">Custom BG</span></pre>"
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
        assert_eq!(html, "<pre><span class=\"bold\">Bold Text</span></pre>");
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
            "<pre><span class=\"bold italic underline\">Styled Text</span></pre>"
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
        assert_eq!(html, "<pre><span class=\"fg-7 bg-1\">Reversed</span></pre>");
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
        assert_eq!(
            html,
            "<pre><span>Normal</span><span class=\"bold\">Bold</span></pre>"
        );
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
        assert_eq!(html, "<pre><span>Text</span><span>More</span></pre>");
    }

    #[test]
    fn test_html_multiple_consecutive_spaces() {
        let styled_text = StyledText {
            segments: vec![Segment {
                text: "hello         world".to_string(),
                style: Style::default(),
            }],
        };
        let html = styled_text.to_html();
        assert_eq!(html, "<pre><span>hello         world</span></pre>");
    }

    #[test]
    fn test_html_multiple_lines() {
        let styled_text = StyledText {
            segments: vec![Segment {
                text: "line1\nline2\nline3".to_string(),
                style: Style::default(),
            }],
        };
        let html = styled_text.to_html();
        assert_eq!(html, "<pre><span>line1\nline2\nline3</span></pre>");
    }

    #[test]
    fn test_html_large_input_with_spaces_and_newlines() {
        let styled_text = StyledText {
            segments: vec![Segment {
                text: "This is a test\nwith    multiple   spaces\nand\nnewlines".to_string(),
                style: Style::default(),
            }],
        };
        let html = styled_text.to_html();
        assert_eq!(
            html,
            "<pre><span>This is a test\nwith    multiple   spaces\nand\nnewlines</span></pre>"
        );
    }

    #[test]
    fn test_html_multiple_segments_with_spaces_and_styles() {
        let styled_text = StyledText {
            segments: vec![
                Segment {
                    text: "Normal text   with spaces".to_string(),
                    style: Style::default(),
                },
                Segment {
                    text: "Bold    text".to_string(),
                    style: Style {
                        bold: true,
                        ..Default::default()
                    },
                },
                Segment {
                    text: "\nRed     text".to_string(),
                    style: Style {
                        fg_color: Some(Color::Indexed(1)),
                        ..Default::default()
                    },
                },
                Segment {
                    text: "   Underlined".to_string(),
                    style: Style {
                        underline: true,
                        ..Default::default()
                    },
                },
            ],
        };
        let html = styled_text.to_html();
        assert_eq!(
            html,
            "<pre><span>Normal text   with spaces</span><span class=\"bold\">Bold    text</span><span class=\"fg-1\">\nRed     text</span><span class=\"underline\">   Underlined</span></pre>"
        );
    }

    #[test]
    fn test_html_filter_spaces_with_matching_fg() {
        let styled_text = StyledText {
            segments: vec![
                Segment {
                    text: "Data".to_string(),
                    style: Style::default(),
                },
                Segment {
                    text: "   ".to_string(),
                    style: Style {
                        fg_color: Some(Color::Indexed(0)), // #000000
                        ..Default::default()
                    },
                },
                Segment {
                    text: "More".to_string(),
                    style: Style::default(),
                },
            ],
        };
        let html = styled_text.to_html_with_filter(Some("#000000"));
        assert_eq!(
            html,
            "<pre><span>Data</span><span>&nbsp;&nbsp;&nbsp;</span><span>More</span></pre>"
        );
    }

    #[test]
    fn test_html_filter_trailing_spaces() {
        let styled_text = StyledText {
            segments: vec![
                Segment {
                    text: "Data".to_string(),
                    style: Style::default(),
                },
                Segment {
                    text: "   ".to_string(),
                    style: Style {
                        fg_color: Some(Color::Indexed(0)), // #000000
                        ..Default::default()
                    },
                },
            ],
        };
        let html = styled_text.to_html_with_filter(Some("#000000"));
        assert_eq!(html, "<pre><span>Data</span></pre>");
    }

    #[test]
    fn test_html_no_filter_non_spaces() {
        let styled_text = StyledText {
            segments: vec![
                Segment {
                    text: "Data".to_string(),
                    style: Style::default(),
                },
                Segment {
                    text: "XXX".to_string(),
                    style: Style {
                        fg_color: Some(Color::Indexed(0)), // #000000
                        ..Default::default()
                    },
                },
                Segment {
                    text: "More".to_string(),
                    style: Style::default(),
                },
            ],
        };
        let html = styled_text.to_html_with_filter(Some("#000000"));
        assert_eq!(
            html,
            "<pre><span>Data</span><span class=\"fg-0\">XXX</span><span>More</span></pre>"
        );
    }
}
