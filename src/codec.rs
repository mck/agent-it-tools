//! The encode/decode codec pairs: base64, hex, gzip, punycode, html entities.

use crate::util::read_input;
use anyhow::{Context, Result};
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum EncodeCmd {
    /// Encode text to Base64
    Base64 {
        /// Use URL-safe alphabet without padding
        #[arg(long)]
        url_safe: bool,
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
    /// Encode text to a lowercase hex string
    Hex {
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
    /// Gzip-compress text and emit it as Base64
    Gzip {
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
    /// Convert an internationalized domain name to ASCII punycode (IDNA)
    Punycode {
        /// Domain (reads stdin if omitted)
        input: Option<String>,
    },
    /// Escape text for safe embedding in HTML (quotes included)
    Html {
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum DecodeCmd {
    /// Decode Base64 to text (standard and URL-safe alphabets both accepted)
    Base64 {
        /// Input Base64 (reads stdin if omitted)
        input: Option<String>,
    },
    /// Decode a hex string to text (0x prefix and whitespace tolerated)
    Hex {
        /// Input hex (reads stdin if omitted)
        input: Option<String>,
    },
    /// Decompress Base64-encoded gzip data back to text
    Gzip {
        /// Input Base64 (reads stdin if omitted)
        input: Option<String>,
    },
    /// Convert an ASCII punycode domain back to unicode
    Punycode {
        /// Domain (reads stdin if omitted)
        input: Option<String>,
    },
    /// Decode HTML entities back to plain text
    Html {
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
}

pub fn run_encode(cmd: EncodeCmd) -> Result<()> {
    match cmd {
        EncodeCmd::Base64 { url_safe, input } => {
            let input = read_input(input)?;
            let encoded = if url_safe {
                URL_SAFE_NO_PAD.encode(input.as_bytes())
            } else {
                STANDARD.encode(input.as_bytes())
            };
            println!("{encoded}");
        }
        EncodeCmd::Hex { input } => {
            let input = read_input(input)?;
            println!("{}", hex::encode(input.as_bytes()));
        }
        EncodeCmd::Gzip { input } => {
            use std::io::Write;
            let input = read_input(input)?;
            let mut encoder =
                flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(input.as_bytes())?;
            println!("{}", STANDARD.encode(encoder.finish()?));
        }
        EncodeCmd::Punycode { input } => {
            let input = read_input(input)?;
            let ascii = idna::domain_to_ascii(input.trim())
                .map_err(|e| anyhow::anyhow!("invalid domain: {e}"))?;
            println!("{ascii}");
        }
        EncodeCmd::Html { input } => {
            let input = read_input(input)?;
            println!("{}", html_escape::encode_quoted_attribute(&input));
        }
    }
    Ok(())
}

pub fn run_decode(cmd: DecodeCmd) -> Result<()> {
    match cmd {
        DecodeCmd::Base64 { input } => {
            let input = read_input(input)?;
            let trimmed: String = input.split_whitespace().collect();
            let bytes = STANDARD
                .decode(&trimmed)
                .or_else(|_| URL_SAFE_NO_PAD.decode(&trimmed))
                .context("invalid Base64 input")?;
            let text =
                String::from_utf8(bytes).context("decoded Base64 payload is not valid UTF-8")?;
            println!("{text}");
        }
        DecodeCmd::Hex { input } => {
            let input = read_input(input)?;
            let trimmed: String = input.split_whitespace().collect();
            let bytes =
                hex::decode(trimmed.trim_start_matches("0x")).context("invalid hex input")?;
            let text =
                String::from_utf8(bytes).context("decoded hex payload is not valid UTF-8 text")?;
            println!("{text}");
        }
        DecodeCmd::Gzip { input } => {
            use std::io::Read;
            let input = read_input(input)?;
            let trimmed: String = input.split_whitespace().collect();
            let bytes = STANDARD
                .decode(&trimmed)
                .or_else(|_| URL_SAFE_NO_PAD.decode(&trimmed))
                .context("invalid Base64 input")?;
            let mut decoder = flate2::read::GzDecoder::new(bytes.as_slice());
            let mut out = String::new();
            decoder
                .read_to_string(&mut out)
                .context("input is not valid gzip data or not UTF-8 text")?;
            println!("{out}");
        }
        DecodeCmd::Punycode { input } => {
            let input = read_input(input)?;
            let (unicode, result) = idna::domain_to_unicode(input.trim());
            result.map_err(|e| anyhow::anyhow!("invalid punycode domain: {e}"))?;
            println!("{unicode}");
        }
        DecodeCmd::Html { input } => {
            let input = read_input(input)?;
            println!("{}", html_escape::decode_html_entities(&input));
        }
    }
    Ok(())
}
