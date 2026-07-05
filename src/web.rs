use crate::util::{print_json, read_input};
use anyhow::{bail, Context, Result};
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;
use chrono::Utc;
use clap::Subcommand;
use url::Url;
use woothee::parser::Parser as UaParser;

#[derive(Subcommand)]
pub enum WebCmd {
    /// Percent-encode text for safe use in a URL component
    UrlEncode {
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
    /// Decode a percent-encoded URL component
    UrlDecode {
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
    /// Parse a URL into its components (JSON)
    UrlParse {
        /// URL (reads stdin if omitted)
        input: Option<String>,
    },
    /// Decode a JWT's header and payload without verifying the signature (JSON)
    Jwt {
        /// JWT token (reads stdin if omitted)
        input: Option<String>,
    },
    /// Parse a user-agent string into browser/OS details (JSON)
    UserAgent {
        /// User-agent string (reads stdin if omitted)
        input: Option<String>,
    },
    /// Escape text for safe embedding in HTML
    HtmlEscape {
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
    /// Decode HTML entities back to text
    HtmlUnescape {
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
    /// Generate a Basic Auth Authorization header value
    BasicAuth {
        /// Username
        username: String,
        /// Password
        password: String,
    },
    /// Turn a string into a URL-safe slug
    Slugify {
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
    /// Render Markdown (GFM) to HTML
    MarkdownToHtml {
        /// Markdown input (reads stdin if omitted)
        input: Option<String>,
    },
    /// Convert HTML to Markdown
    HtmlToMarkdown {
        /// HTML input (reads stdin if omitted)
        input: Option<String>,
    },
    /// Look up MIME type for a file extension, or extensions for a MIME type
    Mime {
        /// A file name/extension (e.g. photo.avif) or a MIME type (e.g. image/png)
        input: Option<String>,
    },
    /// Build a URL from a base, path, query parameters and fragment
    UrlBuild {
        /// Base URL, e.g. https://api.example.com
        #[arg(short, long)]
        base: String,
        /// Path to set, e.g. /v2/tokens
        #[arg(long)]
        path: Option<String>,
        /// Query parameter key=value (repeatable, values get percent-encoded)
        #[arg(short, long)]
        param: Vec<String>,
        /// Fragment (without '#')
        #[arg(long)]
        fragment: Option<String>,
    },
    /// Convert an internationalized domain name to ASCII punycode
    PunycodeEncode {
        /// Domain (reads stdin if omitted)
        input: Option<String>,
    },
    /// Convert an ASCII punycode domain back to unicode
    PunycodeDecode {
        /// Domain (reads stdin if omitted)
        input: Option<String>,
    },
    /// Parse a CSS color and show it in hex, rgb and hsl (JSON)
    Color {
        /// Any CSS color: hex, rgb(), hsl(), named... (reads stdin if omitted)
        input: Option<String>,
    },
    /// WCAG contrast ratio between two CSS colors (JSON)
    Contrast {
        /// Foreground color
        foreground: String,
        /// Background color
        background: String,
    },
}

fn decode_jwt_part(part: &str, what: &str) -> Result<serde_json::Value> {
    let bytes = URL_SAFE_NO_PAD
        .decode(part)
        .with_context(|| format!("JWT {what} is not valid base64url"))?;
    serde_json::from_slice(&bytes).with_context(|| format!("JWT {what} is not valid JSON"))
}

pub fn run(cmd: WebCmd) -> Result<()> {
    match cmd {
        WebCmd::UrlEncode { input } => {
            let input = read_input(input)?;
            println!("{}", urlencoding::encode(&input));
        }
        WebCmd::UrlDecode { input } => {
            let input = read_input(input)?;
            println!(
                "{}",
                urlencoding::decode(&input).context("invalid percent-encoding")?
            );
        }
        WebCmd::UrlParse { input } => {
            let input = read_input(input)?;
            let url = Url::parse(input.trim()).context("invalid URL")?;
            let params: serde_json::Map<String, serde_json::Value> = url
                .query_pairs()
                .map(|(k, v)| (k.into_owned(), serde_json::Value::String(v.into_owned())))
                .collect();
            print_json(&serde_json::json!({
                "scheme": url.scheme(),
                "username": url.username(),
                "password": url.password(),
                "host": url.host_str(),
                "port": url.port_or_known_default(),
                "path": url.path(),
                "query": url.query(),
                "params": params,
                "fragment": url.fragment(),
            }))?;
        }
        WebCmd::Jwt { input } => {
            let input = read_input(input)?;
            let token = input.trim().trim_start_matches("Bearer ");
            let parts: Vec<&str> = token.split('.').collect();
            if parts.len() != 3 {
                bail!(
                    "invalid JWT: expected 3 dot-separated segments, found {}",
                    parts.len()
                );
            }
            let header = decode_jwt_part(parts[0], "header")?;
            let payload = decode_jwt_part(parts[1], "payload")?;
            let expired = payload
                .get("exp")
                .and_then(|v| v.as_i64())
                .map(|exp| exp < Utc::now().timestamp());
            print_json(&serde_json::json!({
                "header": header,
                "payload": payload,
                "signature": parts[2],
                "expired": expired,
            }))?;
        }
        WebCmd::UserAgent { input } => {
            let input = read_input(input)?;
            let ua = input.trim();
            match UaParser::new().parse(ua) {
                Some(r) => print_json(&serde_json::json!({
                    "name": r.name,
                    "category": r.category,
                    "browser_type": r.browser_type,
                    "version": r.version,
                    "os": r.os,
                    "os_version": r.os_version,
                    "vendor": r.vendor,
                }))?,
                None => bail!("could not parse user-agent string"),
            }
        }
        WebCmd::HtmlEscape { input } => {
            let input = read_input(input)?;
            println!("{}", html_escape::encode_quoted_attribute(&input));
        }
        WebCmd::HtmlUnescape { input } => {
            let input = read_input(input)?;
            println!("{}", html_escape::decode_html_entities(&input));
        }
        WebCmd::BasicAuth { username, password } => {
            let encoded = STANDARD.encode(format!("{username}:{password}"));
            println!("Basic {encoded}");
        }
        WebCmd::Slugify { input } => {
            let input = read_input(input)?;
            println!("{}", slug::slugify(input));
        }
        WebCmd::MarkdownToHtml { input } => {
            let input = read_input(input)?;
            let mut options = comrak::Options::default();
            options.extension.table = true;
            options.extension.strikethrough = true;
            options.extension.autolink = true;
            options.extension.tasklist = true;
            print!("{}", comrak::markdown_to_html(&input, &options));
        }
        WebCmd::HtmlToMarkdown { input } => {
            let input = read_input(input)?;
            let md =
                htmd::convert(&input).map_err(|e| anyhow::anyhow!("cannot convert HTML: {e}"))?;
            println!("{}", md.trim_end());
        }
        WebCmd::Mime { input } => {
            let input = read_input(input)?;
            let query = input.trim();
            if query.contains('/') {
                let extensions = mime_guess::get_mime_extensions_str(query)
                    .with_context(|| format!("unknown MIME type '{query}'"))?;
                print_json(&serde_json::json!({
                    "mime": query,
                    "extensions": extensions,
                }))?;
            } else {
                let guess = mime_guess::from_path(query)
                    .first()
                    .with_context(|| format!("no MIME type known for '{query}'"))?;
                print_json(&serde_json::json!({
                    "input": query,
                    "mime": guess.essence_str(),
                }))?;
            }
        }
        WebCmd::UrlBuild {
            base,
            path,
            param,
            fragment,
        } => {
            let mut url = Url::parse(base.trim()).context("invalid base URL")?;
            if let Some(p) = path {
                url.set_path(&p);
            }
            for pair in &param {
                let (k, v) = pair
                    .split_once('=')
                    .with_context(|| format!("parameter '{pair}' is not key=value"))?;
                url.query_pairs_mut().append_pair(k, v);
            }
            if let Some(f) = fragment {
                url.set_fragment(Some(&f));
            }
            println!("{url}");
        }
        WebCmd::PunycodeEncode { input } => {
            let input = read_input(input)?;
            let ascii = idna::domain_to_ascii(input.trim())
                .map_err(|e| anyhow::anyhow!("invalid domain: {e}"))?;
            println!("{ascii}");
        }
        WebCmd::PunycodeDecode { input } => {
            let input = read_input(input)?;
            let (unicode, result) = idna::domain_to_unicode(input.trim());
            result.map_err(|e| anyhow::anyhow!("invalid punycode domain: {e}"))?;
            println!("{unicode}");
        }
        WebCmd::Color { input } => {
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
        WebCmd::Contrast {
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
