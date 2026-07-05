use crate::util::{print_json, read_input};
use anyhow::{Context, Result};
use clap::Subcommand;
use url::Url;

#[derive(Subcommand)]
pub enum UrlCmd {
    /// Percent-encode text for safe use in a URL component
    Encode {
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
    /// Decode a percent-encoded URL component
    Decode {
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
    /// Parse a URL into scheme, host, port, path, query params, fragment and credentials
    Parse {
        /// URL (reads stdin if omitted)
        input: Option<String>,
    },
    /// Build a URL from a base, path, query parameters and fragment
    Build {
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
}

pub fn run(cmd: UrlCmd) -> Result<()> {
    match cmd {
        UrlCmd::Encode { input } => {
            let input = read_input(input)?;
            println!("{}", urlencoding::encode(&input));
        }
        UrlCmd::Decode { input } => {
            let input = read_input(input)?;
            println!(
                "{}",
                urlencoding::decode(&input).context("invalid percent-encoding")?
            );
        }
        UrlCmd::Parse { input } => {
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
        UrlCmd::Build {
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
    }
    Ok(())
}
