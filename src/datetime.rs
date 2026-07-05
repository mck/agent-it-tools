use crate::util::{print_json, read_input};
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Local, TimeZone, Utc};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum DatetimeCmd {
    /// Current time in all common formats (JSON)
    Now,
    /// Convert unix seconds/milliseconds, RFC 3339 or RFC 2822 into all common formats (JSON)
    Convert {
        /// Timestamp or date string (reads stdin if omitted)
        input: Option<String>,
    },
}

fn report(dt: DateTime<Utc>) -> serde_json::Value {
    serde_json::json!({
        "unix": dt.timestamp(),
        "unix_ms": dt.timestamp_millis(),
        "iso8601": dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "rfc2822": dt.to_rfc2822(),
        "local": dt.with_timezone(&Local).to_rfc3339_opts(chrono::SecondsFormat::Secs, false),
    })
}

fn parse(raw: &str) -> Result<DateTime<Utc>> {
    if raw.eq_ignore_ascii_case("now") {
        return Ok(Utc::now());
    }
    if let Ok(num) = raw.parse::<i64>() {
        // Heuristic: 13+ digit magnitudes are unix milliseconds.
        let ts = if num.abs() >= 100_000_000_000 {
            Utc.timestamp_millis_opt(num)
        } else {
            Utc.timestamp_opt(num, 0)
        };
        return match ts.single() {
            Some(dt) => Ok(dt),
            None => bail!("'{raw}' is out of range for a unix timestamp"),
        };
    }
    DateTime::parse_from_rfc3339(raw)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| DateTime::parse_from_rfc2822(raw).map(|dt| dt.with_timezone(&Utc)))
        .with_context(|| format!("'{raw}' is not a unix timestamp, RFC 3339 or RFC 2822"))
}

pub fn run(cmd: DatetimeCmd) -> Result<()> {
    match cmd {
        DatetimeCmd::Now => print_json(&report(Utc::now())),
        DatetimeCmd::Convert { input } => {
            let input = read_input(input)?;
            print_json(&report(parse(input.trim())?))
        }
    }
}
