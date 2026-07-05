use crate::util::{print_json, read_input};
use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum ColorCmd {
    /// Parse any CSS color (hex, rgb(), hsl(), named) and show hex, rgb, hsl and WCAG luminance
    Parse {
        /// Any CSS color: hex, rgb(), hsl(), named... (reads stdin if omitted)
        input: Option<String>,
    },
    /// WCAG 2.x contrast ratio between two CSS colors, with AA/AAA verdicts
    Contrast {
        /// Foreground color
        foreground: String,
        /// Background color
        background: String,
    },
}

/// WCAG 2.x relative luminance of an sRGB color.
fn relative_luminance(color: &csscolorparser::Color) -> f64 {
    let channel = |c: f32| -> f64 {
        let c = c as f64;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color.r) + 0.7152 * channel(color.g) + 0.0722 * channel(color.b)
}

pub fn run(cmd: ColorCmd) -> Result<()> {
    match cmd {
        ColorCmd::Parse { input } => {
            let input = read_input(input)?;
            let color = csscolorparser::parse(input.trim())
                .map_err(|e| anyhow::anyhow!("invalid CSS color: {e}"))?;
            let [r, g, b, a] = color.to_rgba8();
            let [h, s, l, _] = color.to_hsla();
            print_json(&serde_json::json!({
                "hex": color.to_css_hex(),
                "rgb": format!("rgb({r} {g} {b}{})", if a < 255 { format!(" / {:.2}", a as f32 / 255.0) } else { String::new() }),
                "hsl": format!("hsl({:.0} {:.0}% {:.0}%)", h, s * 100.0, l * 100.0),
                "luminance": (relative_luminance(&color) * 10000.0).round() / 10000.0,
            }))?;
        }
        ColorCmd::Contrast {
            foreground,
            background,
        } => {
            let fg = csscolorparser::parse(foreground.trim())
                .map_err(|e| anyhow::anyhow!("invalid foreground color: {e}"))?;
            let bg = csscolorparser::parse(background.trim())
                .map_err(|e| anyhow::anyhow!("invalid background color: {e}"))?;
            let (l1, l2) = (relative_luminance(&fg), relative_luminance(&bg));
            let (hi, lo) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
            let ratio = (hi + 0.05) / (lo + 0.05);
            let ratio = (ratio * 100.0).round() / 100.0;
            print_json(&serde_json::json!({
                "ratio": ratio,
                "aa_normal_text": ratio >= 4.5,
                "aa_large_text": ratio >= 3.0,
                "aaa_normal_text": ratio >= 7.0,
                "aaa_large_text": ratio >= 4.5,
            }))?;
        }
    }
    Ok(())
}
