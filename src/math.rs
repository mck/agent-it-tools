use crate::util::{print_json, read_input};
use anyhow::{bail, Context, Result};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum MathCmd {
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

pub fn run(cmd: MathCmd) -> Result<()> {
    match cmd {
        MathCmd::Calc { input } => {
            let input = read_input(input)?;
            let result = exmex::eval_str::<f64>(&input)
                .map_err(|e| anyhow::anyhow!("invalid expression: {e}"))?;
            if result.fract() == 0.0 && result.abs() < 1e15 {
                println!("{}", result as i64);
            } else {
                println!("{result}");
            }
        }
        MathCmd::Bitwise { op, a, b } => {
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
        MathCmd::NumberBase { from, to, input } => {
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
    }
    Ok(())
}
