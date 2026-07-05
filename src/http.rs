use crate::util::{print_json, read_input};
use anyhow::{bail, Context, Result};
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;
use chrono::Utc;
use clap::Subcommand;
use woothee::parser::Parser as UaParser;

#[derive(Subcommand)]
pub enum HttpCmd {
    /// Build an HTTP Basic Auth Authorization header value from username and password
    BasicAuth {
        /// Username
        username: String,
        /// Password
        password: String,
    },
    /// Parse a user-agent string into browser, version, OS and device category
    UserAgent {
        /// User-agent string (reads stdin if omitted)
        input: Option<String>,
    },
    /// Look up MIME type for a file extension, or extensions for a MIME type
    Mime {
        /// A file name/extension (e.g. photo.avif) or a MIME type (e.g. image/png)
        input: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum JwtCmd {
    /// Decode a JWT's header and payload WITHOUT verifying the signature
    Decode {
        /// JWT token (reads stdin if omitted)
        input: Option<String>,
    },
}

fn decode_jwt_part(part: &str, what: &str) -> Result<serde_json::Value> {
    let bytes = URL_SAFE_NO_PAD
        .decode(part)
        .with_context(|| format!("JWT {what} is not valid base64url"))?;
    serde_json::from_slice(&bytes).with_context(|| format!("JWT {what} is not valid JSON"))
}

pub fn run_http(cmd: HttpCmd) -> Result<()> {
    match cmd {
        HttpCmd::BasicAuth { username, password } => {
            let encoded = STANDARD.encode(format!("{username}:{password}"));
            println!("Basic {encoded}");
        }
        HttpCmd::UserAgent { input } => {
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
        HttpCmd::Mime { input } => {
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
    }
    Ok(())
}

pub fn run_jwt(cmd: JwtCmd) -> Result<()> {
    match cmd {
        JwtCmd::Decode { input } => {
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
    }
    Ok(())
}
