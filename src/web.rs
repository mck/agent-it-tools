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
    }
    Ok(())
}
