use crate::util::{print_json, read_input};
use anyhow::{bail, Context, Result};
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;
use chrono::{DateTime, Local, TimeZone, Utc};
use clap::Subcommand;
use heck::{
    ToKebabCase, ToLowerCamelCase, ToShoutySnakeCase, ToSnakeCase, ToTitleCase, ToTrainCase,
    ToUpperCamelCase,
};

#[derive(Subcommand)]
pub enum ConverterCmd {
    /// Convert structured data between JSON, YAML and TOML
    Data {
        /// Source format: json | yaml | toml
        #[arg(short, long)]
        from: String,
        /// Target format: json | yaml | toml
        #[arg(short, long)]
        to: String,
        /// Input document (reads stdin if omitted)
        input: Option<String>,
    },
    /// Prettify or minify a JSON document
    JsonFormat {
        /// Emit minified JSON instead of pretty-printed
        #[arg(long)]
        minify: bool,
        /// Input JSON (reads stdin if omitted)
        input: Option<String>,
    },
    /// Encode text to Base64
    Base64Encode {
        /// Use URL-safe alphabet without padding
        #[arg(long)]
        url_safe: bool,
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
    /// Decode Base64 to text
    Base64Decode {
        /// Input Base64 (reads stdin if omitted); standard and URL-safe alphabets accepted
        input: Option<String>,
    },
    /// Encode text to a hex string
    HexEncode {
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
    /// Decode a hex string to text
    HexDecode {
        /// Input hex (reads stdin if omitted)
        input: Option<String>,
    },
    /// Convert a string between naming cases
    Case {
        /// Target case: camel | pascal | snake | constant | kebab | train | title | dot | path | lower | upper
        #[arg(short, long)]
        to: String,
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
    /// Convert an integer between numeral bases (2-36)
    NumberBase {
        /// Source base
        #[arg(short, long, default_value_t = 10)]
        from: u32,
        /// Target base
        #[arg(short, long)]
        to: u32,
        /// Input number (reads stdin if omitted)
        input: Option<String>,
    },
    /// Convert a date-time between unix epoch, ISO 8601 and RFC 2822
    Datetime {
        /// "now", unix seconds, unix milliseconds, or an RFC 3339 / ISO 8601 string (defaults to now)
        input: Option<String>,
    },
}

fn parse_to_json(format: &str, input: &str) -> Result<serde_json::Value> {
    match format {
        "json" => serde_json::from_str(input).context("invalid JSON input"),
        "yaml" | "yml" => serde_yaml::from_str(input).context("invalid YAML input"),
        "toml" => toml::from_str(input).context("invalid TOML input"),
        other => bail!("unsupported source format: {other} (expected json, yaml or toml)"),
    }
}

fn render_from_json(format: &str, value: &serde_json::Value) -> Result<String> {
    match format {
        "json" => Ok(serde_json::to_string_pretty(value)?),
        "yaml" | "yml" => Ok(serde_yaml::to_string(value)?.trim_end().to_string()),
        "toml" => toml::to_string_pretty(value)
            .context("value cannot be represented as TOML (e.g. null values or a non-table root)"),
        other => bail!("unsupported target format: {other} (expected json, yaml or toml)"),
    }
}

fn to_radix(mut value: u128, radix: u32) -> String {
    const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    if value == 0 {
        return "0".into();
    }
    let mut out = Vec::new();
    while value > 0 {
        out.push(DIGITS[(value % radix as u128) as usize]);
        value /= radix as u128;
    }
    out.reverse();
    String::from_utf8(out).expect("radix digits are ASCII")
}

fn datetime_report(dt: DateTime<Utc>) -> serde_json::Value {
    serde_json::json!({
        "unix": dt.timestamp(),
        "unix_ms": dt.timestamp_millis(),
        "iso8601": dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "rfc2822": dt.to_rfc2822(),
        "local": dt.with_timezone(&Local).to_rfc3339_opts(chrono::SecondsFormat::Secs, false),
    })
}

pub fn run(cmd: ConverterCmd) -> Result<()> {
    match cmd {
        ConverterCmd::Data { from, to, input } => {
            let input = read_input(input)?;
            let value = parse_to_json(&from.to_lowercase(), &input)?;
            println!("{}", render_from_json(&to.to_lowercase(), &value)?);
        }
        ConverterCmd::JsonFormat { minify, input } => {
            let input = read_input(input)?;
            let value: serde_json::Value =
                serde_json::from_str(&input).context("invalid JSON input")?;
            if minify {
                println!("{}", serde_json::to_string(&value)?);
            } else {
                println!("{}", serde_json::to_string_pretty(&value)?);
            }
        }
        ConverterCmd::Base64Encode { url_safe, input } => {
            let input = read_input(input)?;
            let encoded = if url_safe {
                URL_SAFE_NO_PAD.encode(input.as_bytes())
            } else {
                STANDARD.encode(input.as_bytes())
            };
            println!("{encoded}");
        }
        ConverterCmd::Base64Decode { input } => {
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
        ConverterCmd::HexEncode { input } => {
            let input = read_input(input)?;
            println!("{}", hex::encode(input.as_bytes()));
        }
        ConverterCmd::HexDecode { input } => {
            let input = read_input(input)?;
            let trimmed: String = input.split_whitespace().collect();
            let bytes =
                hex::decode(trimmed.trim_start_matches("0x")).context("invalid hex input")?;
            let text =
                String::from_utf8(bytes).context("decoded hex payload is not valid UTF-8")?;
            println!("{text}");
        }
        ConverterCmd::Case { to, input } => {
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
        ConverterCmd::NumberBase { from, to, input } => {
            let input = read_input(input)?;
            if !(2..=36).contains(&from) || !(2..=36).contains(&to) {
                bail!("bases must be between 2 and 36");
            }
            let raw = input.trim().to_lowercase();
            let (negative, digits) = match raw.strip_prefix('-') {
                Some(rest) => (true, rest),
                None => (false, raw.as_str()),
            };
            let value = u128::from_str_radix(digits, from)
                .with_context(|| format!("'{raw}' is not a valid base-{from} integer"))?;
            let rendered = to_radix(value, to);
            println!("{}{rendered}", if negative { "-" } else { "" });
        }
        ConverterCmd::Datetime { input } => {
            let input = input.unwrap_or_else(|| "now".to_string());
            let raw = input.trim();
            let dt: DateTime<Utc> = if raw.eq_ignore_ascii_case("now") {
                Utc::now()
            } else if let Ok(num) = raw.parse::<i64>() {
                // Heuristic: 13+ digit magnitudes are unix milliseconds.
                let ts = if num.abs() >= 100_000_000_000 {
                    Utc.timestamp_millis_opt(num)
                } else {
                    Utc.timestamp_opt(num, 0)
                };
                match ts.single() {
                    Some(dt) => dt,
                    None => bail!("'{raw}' is out of range for a unix timestamp"),
                }
            } else {
                DateTime::parse_from_rfc3339(raw)
                    .map(|dt| dt.with_timezone(&Utc))
                    .or_else(|_| DateTime::parse_from_rfc2822(raw).map(|dt| dt.with_timezone(&Utc)))
                    .with_context(|| {
                        format!("'{raw}' is not 'now', a unix timestamp, RFC 3339 or RFC 2822")
                    })?
            };
            print_json(&datetime_report(dt))?;
        }
    }
    Ok(())
}
