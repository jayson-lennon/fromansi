use crate::{Color, StyledText};
use codepage_437::CP437_WINGDINGS;
use rexpaint::{XpColor, XpFile};

impl StyledText {
    /// Converts the styled text to a RexPaint XpFile.
    ///
    /// This method creates a RexPaint file with dimensions automatically calculated
    /// from the input text (width = longest line, height = number of lines).
    ///
    /// # Returns
    ///
    /// An `XpFile` containing the styled text with colors and characters encoded
    /// in the RexPaint format.
    ///
    /// # Note
    ///
    /// - Text styles (bold, italic, underline, etc.) are not supported by RexPaint
    ///   and will be ignored
    /// - Characters that cannot be encoded in CP437 will be replaced with '?'
    /// - Default colors are white foreground on black background
    #[must_use]
    pub fn to_rexpaint(&self) -> XpFile {
        let lines = self.split_lines();
        
        // Calculate dimensions
        let height = lines.len().max(1);
        let width = lines
            .iter()
            .map(|line| calculate_line_width(line))
            .max()
            .unwrap_or(80)
            .max(1);

        let mut xp = XpFile::new(width, height);

        // Default colors (white on black)
        let default_fg = XpColor::new(255, 255, 255);
        let default_bg = XpColor::new(0, 0, 0);

        // Fill the file with styled text
        for (y, line) in lines.iter().enumerate() {
            let mut x = 0;
            
            for segment in &line.segments {
                if segment.style.hidden {
                    // Skip hidden segments
                    x += segment.text.chars().count();
                    continue;
                }

                // Determine colors, considering reverse
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

                let fg = fg_color.map_or(default_fg, color_to_xp_color);
                let bg = bg_color.map_or(default_bg, color_to_xp_color);

                // Write each character
                for ch in segment.text.chars() {
                    if x >= width {
                        break; // Don't exceed calculated width
                    }

                    if let Some(cell) = xp.layers[0].get_mut(x, y) {
                        cell.ch = encode_char(ch);
                        cell.fg = fg;
                        cell.bg = bg;
                    }
                    x += 1;
                }
            }
        }

        xp
    }
}

/// Calculates the display width of a line of styled text.
///
/// This counts the number of visible characters in the line.
fn calculate_line_width(line: &StyledText) -> usize {
    line.segments
        .iter()
        .map(|seg| seg.text.chars().count())
        .sum()
}

/// Converts a Color enum to an XpColor.
///
/// For RGB colors, this is a direct mapping.
/// For indexed colors, this converts to RGB using the ANSI color palette.
fn color_to_xp_color(color: &Color) -> XpColor {
    match color {
        Color::Rgb(r, g, b) => XpColor::new(*r, *g, *b),
        Color::Indexed(_idx) => {
            // Convert indexed color to RGB using the same logic as to_hex()
            let hex = color.to_hex();
            // Parse hex color #RRGGBB
            let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(255);
            let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(255);
            let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(255);
            XpColor::new(r, g, b)
        }
    }
}

/// Encodes a character to CP437 format for RexPaint.
///
/// Characters that cannot be encoded in CP437 are replaced with '?' (character code 63).
fn encode_char(ch: char) -> u32 {
    if ch == '\n' || ch == '\r' {
        // Newlines should not appear in individual cells
        return u32::from(CP437_WINGDINGS.encode(' ').unwrap_or(32));
    }
    
    u32::from(CP437_WINGDINGS.encode(ch).unwrap_or(63)) // 63 is '?'
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Segment, Style};

    #[test]
    fn test_rexpaint_plain_text() {
        let styled_text = StyledText {
            segments: vec![Segment {
                text: "Hello".to_string(),
                style: Style::default(),
            }],
        };
        let xp = styled_text.to_rexpaint();
        
        assert_eq!(xp.layers[0].width, 5);
        assert_eq!(xp.layers[0].height, 1);
        
        // Check first character
        let cell = xp.layers[0].get(0, 0).unwrap();
        assert_eq!(cell.ch, u32::from(CP437_WINGDINGS.encode('H').unwrap()));
        assert_eq!(cell.fg, XpColor::new(255, 255, 255)); // default white
        assert_eq!(cell.bg, XpColor::new(0, 0, 0)); // default black
    }

    #[test]
    fn test_rexpaint_with_colors() {
        let styled_text = StyledText {
            segments: vec![Segment {
                text: "Red".to_string(),
                style: Style {
                    fg_color: Some(Color::Rgb(255, 0, 0)),
                    bg_color: Some(Color::Rgb(0, 0, 255)),
                    ..Default::default()
                },
            }],
        };
        let xp = styled_text.to_rexpaint();
        
        let cell = xp.layers[0].get(0, 0).unwrap();
        assert_eq!(cell.fg, XpColor::new(255, 0, 0)); // red
        assert_eq!(cell.bg, XpColor::new(0, 0, 255)); // blue
    }

    #[test]
    fn test_rexpaint_indexed_colors() {
        let styled_text = StyledText {
            segments: vec![Segment {
                text: "X".to_string(),
                style: Style {
                    fg_color: Some(Color::Indexed(1)), // red
                    bg_color: Some(Color::Indexed(4)), // blue
                    ..Default::default()
                },
            }],
        };
        let xp = styled_text.to_rexpaint();
        
        let cell = xp.layers[0].get(0, 0).unwrap();
        // Indexed 1 is #800000 (dark red)
        assert_eq!(cell.fg, XpColor::new(128, 0, 0));
        // Indexed 4 is #000080 (dark blue)
        assert_eq!(cell.bg, XpColor::new(0, 0, 128));
    }

    #[test]
    fn test_rexpaint_multiline() {
        let styled_text = StyledText {
            segments: vec![Segment {
                text: "Line1\nLine2\nLine3".to_string(),
                style: Style::default(),
            }],
        };
        let xp = styled_text.to_rexpaint();
        
        assert_eq!(xp.layers[0].height, 3);
        assert_eq!(xp.layers[0].width, 5); // "Line1" is longest
    }

    #[test]
    fn test_rexpaint_reverse_colors() {
        let styled_text = StyledText {
            segments: vec![Segment {
                text: "R".to_string(),
                style: Style {
                    fg_color: Some(Color::Rgb(255, 0, 0)),
                    bg_color: Some(Color::Rgb(0, 255, 0)),
                    reverse: true,
                    ..Default::default()
                },
            }],
        };
        let xp = styled_text.to_rexpaint();
        
        let cell = xp.layers[0].get(0, 0).unwrap();
        // Colors should be swapped
        assert_eq!(cell.fg, XpColor::new(0, 255, 0)); // green (was bg)
        assert_eq!(cell.bg, XpColor::new(255, 0, 0)); // red (was fg)
    }

    #[test]
    fn test_rexpaint_hidden_text() {
        let styled_text = StyledText {
            segments: vec![
                Segment {
                    text: "Visible".to_string(),
                    style: Style::default(),
                },
                Segment {
                    text: "Hidden".to_string(),
                    style: Style {
                        hidden: true,
                        ..Default::default()
                    },
                },
            ],
        };
        let xp = styled_text.to_rexpaint();
        
        // Width should only count visible text
        // But hidden text still takes up space in the calculation
        assert_eq!(xp.layers[0].width, 13); // "Visible" + "Hidden"
    }

    #[test]
    fn test_rexpaint_varying_line_widths() {
        let styled_text = StyledText {
            segments: vec![
                Segment {
                    text: "Short".to_string(),
                    style: Style::default(),
                },
                Segment {
                    text: "\n".to_string(),
                    style: Style::default(),
                },
                Segment {
                    text: "Much longer line".to_string(),
                    style: Style::default(),
                },
            ],
        };
        let xp = styled_text.to_rexpaint();
        
        assert_eq!(xp.layers[0].width, 16); // "Much longer line"
        assert_eq!(xp.layers[0].height, 2);
    }

    #[test]
    fn test_encode_char() {
        assert_eq!(encode_char('A'), u32::from(CP437_WINGDINGS.encode('A').unwrap()));
        assert_eq!(encode_char(' '), u32::from(CP437_WINGDINGS.encode(' ').unwrap()));
        // Newlines should be converted to spaces
        assert_eq!(encode_char('\n'), u32::from(CP437_WINGDINGS.encode(' ').unwrap()));
    }

    #[test]
    fn test_color_to_xp_color_rgb() {
        let color = Color::Rgb(128, 64, 32);
        let xp_color = color_to_xp_color(&color);
        assert_eq!(xp_color, XpColor::new(128, 64, 32));
    }

    #[test]
    fn test_color_to_xp_color_indexed() {
        let color = Color::Indexed(0); // black
        let xp_color = color_to_xp_color(&color);
        assert_eq!(xp_color, XpColor::new(0, 0, 0));
        
        let color = Color::Indexed(15); // white
        let xp_color = color_to_xp_color(&color);
        assert_eq!(xp_color, XpColor::new(255, 255, 255));
    }
}
