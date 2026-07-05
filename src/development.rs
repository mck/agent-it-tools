use crate::util::{print_json, read_input};
use anyhow::{bail, Context, Result};
use chrono::Utc;
use clap::Subcommand;
use cron::Schedule;
use similar::TextDiff;
use std::str::FromStr;

#[derive(Subcommand)]
pub enum DevCmd {
    /// Validate a crontab expression and list its next occurrences (JSON)
    Cron {
        /// Cron expression, 5-field classic or 6/7-field with seconds/years
        expression: String,
        /// Number of upcoming occurrences to list
        #[arg(short, long, default_value_t = 5)]
        count: usize,
    },
    /// Generate UUIDs
    #[command(disable_version_flag = true)]
    Uuid {
        /// UUID version: v4 | v7 | nil
        #[arg(short, long, default_value = "v4")]
        version: String,
        /// Number of UUIDs to generate (one per line)
        #[arg(short, long, default_value_t = 1)]
        count: usize,
    },
    /// Test a regular expression against text and report matches (JSON)
    Regex {
        /// Regular expression pattern
        #[arg(short, long)]
        pattern: String,
        /// Case-insensitive matching
        #[arg(short, long)]
        ignore_case: bool,
        /// ^ and $ match line boundaries
        #[arg(short, long)]
        multiline: bool,
        /// . matches newlines
        #[arg(short = 's', long)]
        dot_all: bool,
        /// Text to test (reads stdin if omitted)
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

pub fn run(cmd: DevCmd) -> Result<()> {
    match cmd {
        DevCmd::Cron { expression, count } => {
            let raw = expression.trim();
            // The cron crate wants a seconds field; promote classic 5-field
            // crontab expressions by pinning seconds to 0.
            let normalized = if raw.split_whitespace().count() == 5 {
                format!("0 {raw}")
            } else {
                raw.to_string()
            };
            let schedule = Schedule::from_str(&normalized)
                .with_context(|| format!("invalid cron expression: '{raw}'"))?;
            let next: Vec<String> = schedule
                .upcoming(Utc)
                .take(count)
                .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
                .collect();
            print_json(&serde_json::json!({
                "expression": raw,
                "normalized": normalized,
                "valid": true,
                "next": next,
            }))?;
        }
        DevCmd::Uuid { version, count } => {
            for _ in 0..count {
                let id = match version.to_lowercase().as_str() {
                    "v4" | "4" => uuid::Uuid::new_v4(),
                    "v7" | "7" => uuid::Uuid::now_v7(),
                    "nil" => uuid::Uuid::nil(),
                    other => bail!("unsupported UUID version: {other} (expected v4, v7 or nil)"),
                };
                println!("{id}");
            }
        }
        DevCmd::Regex {
            pattern,
            ignore_case,
            multiline,
            dot_all,
            input,
        } => {
            let text = read_input(input)?;
            let re = regex::RegexBuilder::new(&pattern)
                .case_insensitive(ignore_case)
                .multi_line(multiline)
                .dot_matches_new_line(dot_all)
                .build()
                .context("invalid regular expression")?;
            let group_names: Vec<Option<&str>> = re.capture_names().collect();
            let matches: Vec<serde_json::Value> = re
                .captures_iter(&text)
                .map(|caps| {
                    let whole = caps.get(0).expect("group 0 always participates");
                    let groups: serde_json::Map<String, serde_json::Value> = group_names
                        .iter()
                        .enumerate()
                        .skip(1)
                        .map(|(i, name)| {
                            let key = name.map(str::to_string).unwrap_or_else(|| i.to_string());
                            let value = caps
                                .get(i)
                                .map(|m| serde_json::Value::String(m.as_str().to_string()))
                                .unwrap_or(serde_json::Value::Null);
                            (key, value)
                        })
                        .collect();
                    serde_json::json!({
                        "match": whole.as_str(),
                        "start": whole.start(),
                        "end": whole.end(),
                        "groups": groups,
                    })
                })
                .collect();
            print_json(&serde_json::json!({
                "pattern": pattern,
                "is_match": !matches.is_empty(),
                "match_count": matches.len(),
                "matches": matches,
            }))?;
        }
        DevCmd::Diff {
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
