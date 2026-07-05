use anyhow::{Context, Result};
use std::io::Read;

/// Resolve a tool's main body input: an explicit argument wins,
/// otherwise the full stdin pipe is consumed.
pub fn read_input(arg: Option<String>) -> Result<String> {
    match arg {
        Some(value) => Ok(value),
        None => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .context("failed to read input from stdin")?;
            if buf.is_empty() {
                anyhow::bail!("no input provided (pass an argument or pipe data via stdin)");
            }
            // Strip a single trailing newline added by shells/pipes.
            if buf.ends_with('\n') {
                buf.pop();
                if buf.ends_with('\r') {
                    buf.pop();
                }
            }
            Ok(buf)
        }
    }
}

/// Print a serde-serializable value as pretty JSON to stdout.
pub fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
