use crate::util::{print_json, read_input};
use anyhow::{bail, Context, Result};
use clap::Subcommand;
use heck::{
    ToKebabCase, ToLowerCamelCase, ToShoutySnakeCase, ToSnakeCase, ToTitleCase, ToTrainCase,
    ToUpperCamelCase,
};
use similar::TextDiff;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Subcommand)]
pub enum TextCmd {
    /// Convert a string between naming cases
    Case {
        /// Target case: camel | pascal | snake | constant | kebab | train | title | dot | path | lower | upper
        #[arg(short, long)]
        to: String,
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
    /// Turn any string into a URL-safe slug (ASCII, lowercase, hyphen-separated)
    #[command(visible_alias = "slug")]
    Slugify {
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
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
    /// Compute a unified diff between two texts or files
    Diff {
        /// Old text, or a file path with --files
        old: String,
        /// New text, or a file path with --files
        new: String,
        /// Treat the two arguments as file paths
        #[arg(long)]
        files: bool,
        /// Lines of context around changes
        #[arg(short, long, default_value_t = 3)]
        context: usize,
    },
}

pub fn run(cmd: TextCmd) -> Result<()> {
    match cmd {
        TextCmd::Case { to, input } => {
            let input = read_input(input)?;
            let out = match to.to_lowercase().as_str() {
                "camel" => input.to_lower_camel_case(),
                "pascal" => input.to_upper_camel_case(),
                "snake" => input.to_snake_case(),
                "constant" | "shouty" => input.to_shouty_snake_case(),
                "kebab" => input.to_kebab_case(),
                "train" => input.to_train_case(),
                "title" => input.to_title_case(),
                "dot" => input.to_snake_case().replace('_', "."),
                "path" => input.to_snake_case().replace('_', "/"),
                "lower" => input.to_lowercase(),
                "upper" => input.to_uppercase(),
                other => bail!(
                    "unsupported case: {other} (expected camel, pascal, snake, constant, kebab, train, title, dot, path, lower or upper)"
                ),
            };
            println!("{out}");
        }
        TextCmd::Slugify { input } => {
            let input = read_input(input)?;
            println!("{}", slug::slugify(input));
        }
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
                let re = ::regex::Regex::new(pattern).expect("static patterns are valid");
                let mut n = 0usize;
                masked = re
                    .replace_all(&masked, |_: &::regex::Captures| {
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
        TextCmd::Diff {
            old,
            new,
            files,
            context,
        } => {
            let (old_text, new_text, old_label, new_label) = if files {
                (
                    std::fs::read_to_string(&old)
                        .with_context(|| format!("cannot read file '{old}'"))?,
                    std::fs::read_to_string(&new)
                        .with_context(|| format!("cannot read file '{new}'"))?,
                    old.clone(),
                    new.clone(),
                )
            } else {
                (old, new, "old".to_string(), "new".to_string())
            };
            let diff = TextDiff::from_lines(&old_text, &new_text);
            print!(
                "{}",
                diff.unified_diff()
                    .context_radius(context)
                    .header(&old_label, &new_label)
            );
        }
    }
    Ok(())
}
