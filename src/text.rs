use crate::util::{print_json, read_input};
use anyhow::Result;
use clap::Subcommand;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Subcommand)]
pub enum TextCmd {
    /// Count characters, words, lines and bytes of a text (JSON)
    Stats {
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
    /// Similarity metrics between two strings (Levenshtein, Jaro-Winkler)
    Distance {
        /// First string
        a: String,
        /// Second string
        b: String,
    },
    /// Mask sensitive data (emails, IPs, JWTs, bearer tokens, key-like strings)
    Mask {
        /// Replacement label style, e.g. [MASKED]
        #[arg(short, long, default_value = "[MASKED]")]
        replacement: String,
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
}

pub fn run(cmd: TextCmd) -> Result<()> {
    match cmd {
        TextCmd::Stats { input } => {
            let input = read_input(input)?;
            let words = input.unicode_words().count();
            let graphemes = input.graphemes(true).count();
            let lines = input.lines().count();
            let longest_line = input.lines().map(|l| l.graphemes(true).count()).max();
            print_json(&serde_json::json!({
                "bytes": input.len(),
                "characters": graphemes,
                "characters_without_whitespace": input
                    .graphemes(true)
                    .filter(|g| !g.chars().all(char::is_whitespace))
                    .count(),
                "words": words,
                "lines": lines,
                "longest_line_characters": longest_line.unwrap_or(0),
            }))?;
        }
        TextCmd::Distance { a, b } => {
            let levenshtein = strsim::levenshtein(&a, &b);
            print_json(&serde_json::json!({
                "levenshtein": levenshtein,
                "normalized_levenshtein":
                    (strsim::normalized_levenshtein(&a, &b) * 1000.0).round() / 1000.0,
                "jaro_winkler": (strsim::jaro_winkler(&a, &b) * 1000.0).round() / 1000.0,
                "equal": a == b,
            }))?;
        }
        TextCmd::Mask { replacement, input } => {
            let input = read_input(input)?;
            // Ordered: the most specific patterns first so e.g. a JWT is not
            // half-eaten by the generic hex pattern.
            let patterns: [(&str, &str); 7] = [
                (
                    "jwt",
                    r"\beyJ[A-Za-z0-9_-]{6,}\.[A-Za-z0-9_-]{6,}\.[A-Za-z0-9_-]{6,}\b",
                ),
                ("bearer", r"(?i)\bbearer\s+[A-Za-z0-9._~+/-]{8,}=*"),
                ("aws-key", r"\b(?:AKIA|ASIA)[A-Z0-9]{16}\b"),
                (
                    "email",
                    r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b",
                ),
                ("ipv4", r"\b(?:\d{1,3}\.){3}\d{1,3}\b"),
                ("card", r"\b\d{4}[ -]?\d{4}[ -]?\d{4}[ -]?\d{4}\b"),
                ("hex-secret", r"\b[0-9a-fA-F]{32,}\b"),
            ];
            let mut masked = input;
            let mut counts = serde_json::Map::new();
            for (label, pattern) in patterns {
                let re = regex::Regex::new(pattern).expect("static patterns are valid");
                let mut n = 0usize;
                masked = re
                    .replace_all(&masked, |_: &regex::Captures| {
                        n += 1;
                        replacement.clone()
                    })
                    .into_owned();
                if n > 0 {
                    counts.insert(label.to_string(), serde_json::Value::from(n));
                }
            }
            print_json(&serde_json::json!({
                "masked": masked,
                "found": counts,
            }))?;
        }
    }
    Ok(())
}
