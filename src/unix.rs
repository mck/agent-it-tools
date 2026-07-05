use crate::util::{print_json, read_input};
use anyhow::{bail, Result};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum UnixCmd {
    /// Convert chmod permissions between octal and symbolic notation
    Chmod {
        /// Octal (e.g. 755) or symbolic (e.g. rwxr-xr-x)
        input: Option<String>,
    },
}

pub fn run(cmd: UnixCmd) -> Result<()> {
    match cmd {
        UnixCmd::Chmod { input } => {
            let input = read_input(input)?;
            let raw = input.trim();
            let octal: u32 = if raw.chars().all(|c| c.is_ascii_digit()) {
                let digits = raw.trim_start_matches('0');
                let digits = if digits.is_empty() { "0" } else { digits };
                if digits.len() > 4 {
                    bail!("'{raw}' is not a valid chmod value");
                }
                u32::from_str_radix(digits, 8)
                    .map_err(|_| anyhow::anyhow!("'{raw}' is not a valid octal chmod value"))?
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
    }
    Ok(())
}
