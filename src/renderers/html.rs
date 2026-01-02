use crate::{StyledText, Color};

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
                (segment.style.bg_color.as_ref(), segment.style.fg_color.as_ref())
            } else {
                (segment.style.fg_color.as_ref(), segment.style.bg_color.as_ref())
            };

            if let Some(color) = fg_color {
                match color {
                    Color::Indexed(idx) => classes.push(format!("fg-{}", idx)),
                    Color::Rgb(r, g, b) => inline_styles.push(format!("color: rgb({}, {}, {})", r, g, b)),
                }
            }

            if let Some(color) = bg_color {
                match color {
                    Color::Indexed(idx) => classes.push(format!("bg-{}", idx)),
                    Color::Rgb(r, g, b) => inline_styles.push(format!("background-color: rgb({}, {}, {})", r, g, b)),
                }
            }

            // Add style classes
            if segment.style.bold { classes.push("bold".to_string()); }
            if segment.style.dim { classes.push("dim".to_string()); }
            if segment.style.italic { classes.push("italic".to_string()); }
            if segment.style.underline { classes.push("underline".to_string()); }
            if segment.style.blink { classes.push("blink".to_string()); }
            if segment.style.strikethrough { classes.push("strikethrough".to_string()); }
            if segment.style.hidden { classes.push("hidden".to_string()); }

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
            html.push_str(&format!("<span{}{}>{}</span>", class_attr, style_attr, text));
        }
        html
    }
}