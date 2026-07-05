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
    /// Evaluate a mathematical expression exactly
    Calc {
        /// Expression, e.g. "sin(0.5)^2 + 17 * (3 - 1.5)" (reads stdin if omitted)
        input: Option<String>,
    },
    /// Bitwise operation on integers (0x/0b/0o prefixes accepted)
    Bitwise {
        /// Operation: and | or | xor | not | shl | shr
        #[arg(short, long)]
        op: String,
        /// First operand
        a: String,
        /// Second operand (shift amount for shl/shr; omitted for not)
        b: Option<String>,
    },
    /// Convert chmod permissions between octal and symbolic notation
    Chmod {
        /// Octal (e.g. 755) or symbolic (e.g. rwxr-xr-x)
        input: Option<String>,
    },
    /// Generate ULIDs (time-ordered, lexicographically sortable)
    Ulid {
        /// Number of ULIDs to generate (one per line)
        #[arg(short, long, default_value_t = 1)]
        count: usize,
    },
    /// Generate Nano IDs (URL-safe random identifiers)
    Nanoid {
        /// ID length
        #[arg(short, long, default_value_t = 21)]
        length: usize,
        /// Number of IDs to generate (one per line)
        #[arg(short, long, default_value_t = 1)]
        count: usize,
    },
    /// Escape text as a JSON/code string literal (reverse with --unescape)
    StringEscape {
        /// Unescape a quoted string literal instead
        #[arg(short, long)]
        unescape: bool,
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
        DevCmd::Calc { input } => {
            let input = read_input(input)?;
            let result = exmex::eval_str::<f64>(&input)
                .map_err(|e| anyhow::anyhow!("invalid expression: {e}"))?;
            if result.fract() == 0.0 && result.abs() < 1e15 {
                println!("{}", result as i64);
            } else {
                println!("{result}");
            }
        }
        DevCmd::Bitwise { op, a, b } => {
            fn parse_int(raw: &str) -> Result<u64> {
                let raw = raw.trim().replace('_', "");
                let (digits, radix) = match raw.get(..2) {
                    Some("0x") | Some("0X") => (&raw[2..], 16),
                    Some("0b") | Some("0B") => (&raw[2..], 2),
                    Some("0o") | Some("0O") => (&raw[2..], 8),
                    _ => (raw.as_str(), 10),
                };
                u64::from_str_radix(digits, radix)
                    .with_context(|| format!("'{raw}' is not a valid integer"))
            }
            let a = parse_int(&a)?;
            let op = op.to_lowercase();
            let b = match (op.as_str(), b) {
                ("not", None) => 0,
                ("not", Some(_)) => bail!("'not' takes a single operand"),
                (_, Some(b)) => parse_int(&b)?,
                (_, None) => bail!("operation '{op}' needs a second operand"),
            };
            let result = match op.as_str() {
                "and" => a & b,
                "or" => a | b,
                "xor" => a ^ b,
                "not" => !a,
                "shl" => a.checked_shl(b as u32).context("shift amount too large")?,
                "shr" => a.checked_shr(b as u32).context("shift amount too large")?,
                other => {
                    bail!("unsupported operation: {other} (expected and, or, xor, not, shl or shr)")
                }
            };
            print_json(&serde_json::json!({
                "decimal": result,
                "hex": format!("0x{result:x}"),
                "binary": format!("0b{result:b}"),
                "octal": format!("0o{result:o}"),
            }))?;
        }
        DevCmd::Chmod { input } => {
            let input = read_input(input)?;
            let raw = input.trim();
            let octal: u32 = if raw.chars().all(|c| c.is_ascii_digit()) {
                let digits = raw.trim_start_matches('0');
                let digits = if digits.is_empty() { "0" } else { digits };
                if digits.len() > 4 {
                    bail!("'{raw}' is not a valid chmod value");
                }
                u32::from_str_radix(digits, 8)
                    .with_context(|| format!("'{raw}' is not a valid octal chmod value"))?
            } else if raw.len() == 9 {
                let mut value = 0u32;
                for (i, (c, expected)) in raw.chars().zip("rwxrwxrwx".chars()).enumerate() {
                    let bit = 1 << (8 - i);
                    match c {
                        '-' => {}
                        c if c == expected => value |= bit,
                        // setuid/setgid/sticky in the execute slots
                        's' | 'S' if i % 3 == 2 && i < 6 => {
                            value |= if i == 2 { 0o4000 } else { 0o2000 };
                            if c == 's' {
                                value |= bit;
                            }
                        }
                        't' | 'T' if i == 8 => {
                            value |= 0o1000;
                            if c == 't' {
                                value |= bit;
                            }
                        }
                        other => bail!(
                            "unexpected character '{other}' at position {i} (expected rwx pattern)"
                        ),
                    }
                }
                value
            } else {
                bail!("'{raw}' is neither octal (755) nor 9-character symbolic (rwxr-xr-x)");
            };
            if octal > 0o7777 {
                bail!("'{raw}' is out of range for chmod");
            }
            let mut symbolic = String::new();
            for i in 0..9 {
                let bit = 1 << (8 - i);
                let expected = ['r', 'w', 'x'][i % 3];
                let special = match i {
                    2 if octal & 0o4000 != 0 => Some(('s', 'S')),
                    5 if octal & 0o2000 != 0 => Some(('s', 'S')),
                    8 if octal & 0o1000 != 0 => Some(('t', 'T')),
                    _ => None,
                };
                let set = octal & bit != 0;
                symbolic.push(match (special, set) {
                    (Some((lower, _)), true) => lower,
                    (Some((_, upper)), false) => upper,
                    (None, true) => expected,
                    (None, false) => '-',
                });
            }
            print_json(&serde_json::json!({
                "octal": format!("{octal:o}"),
                "symbolic": symbolic,
                "command": format!("chmod {octal:o} <file>"),
            }))?;
        }
        DevCmd::Ulid { count } => {
            for _ in 0..count {
                println!("{}", ulid::Ulid::new());
            }
        }
        DevCmd::Nanoid { length, count } => {
            if length == 0 {
                bail!("length must be greater than zero");
            }
            for _ in 0..count {
                println!(
                    "{}",
                    nanoid::format(nanoid::rngs::default, &nanoid::alphabet::SAFE, length)
                );
            }
        }
        DevCmd::StringEscape { unescape, input } => {
            let input = read_input(input)?;
            if unescape {
                let quoted = if input.starts_with('"') && input.ends_with('"') && input.len() >= 2 {
                    input.clone()
                } else {
                    format!("\"{input}\"")
                };
                let unescaped: String = serde_json::from_str(&quoted)
                    .context("input is not a valid escaped string literal")?;
                println!("{unescaped}");
            } else {
                println!("{}", serde_json::to_string(&input)?);
            }
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
