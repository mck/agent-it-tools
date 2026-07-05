use crate::util::{print_json, read_input};
use anyhow::{Context, Result};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum RegexCmd {
    /// Test a regular expression against text; reports all matches with positions and named groups
    Test {
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
}

pub fn run(cmd: RegexCmd) -> Result<()> {
    match cmd {
        RegexCmd::Test {
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
    }
    Ok(())
}
