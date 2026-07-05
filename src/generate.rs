use anyhow::{bail, Result};
use clap::Subcommand;
use rand::Rng;

#[derive(Subcommand)]
pub enum GenerateCmd {
    /// Generate UUIDs (v4 random, v7 time-ordered, nil)
    #[command(disable_version_flag = true)]
    Uuid {
        /// UUID version: v4 | v7 | nil
        #[arg(short, long, default_value = "v4")]
        version: String,
        /// Number of UUIDs to generate (one per line)
        #[arg(short, long, default_value_t = 1)]
        count: usize,
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
    /// Generate cryptographically random token strings (configurable charset)
    Token {
        /// Token length
        #[arg(short, long, default_value_t = 64)]
        length: usize,
        /// Include symbols (!@#$%^&*()-_=+)
        #[arg(long)]
        symbols: bool,
        /// Exclude digits
        #[arg(long)]
        no_numbers: bool,
        /// Exclude uppercase letters
        #[arg(long)]
        no_uppercase: bool,
        /// Exclude lowercase letters
        #[arg(long)]
        no_lowercase: bool,
        /// Number of tokens to generate (one per line)
        #[arg(short, long, default_value_t = 1)]
        count: usize,
    },
}

pub fn run(cmd: GenerateCmd) -> Result<()> {
    match cmd {
        GenerateCmd::Uuid { version, count } => {
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
        GenerateCmd::Ulid { count } => {
            for _ in 0..count {
                println!("{}", ulid::Ulid::new());
            }
        }
        GenerateCmd::Nanoid { length, count } => {
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
        GenerateCmd::Token {
            length,
            symbols,
            no_numbers,
            no_uppercase,
            no_lowercase,
            count,
        } => {
            let mut charset = String::new();
            if !no_lowercase {
                charset.push_str("abcdefghijklmnopqrstuvwxyz");
            }
            if !no_uppercase {
                charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
            }
            if !no_numbers {
                charset.push_str("0123456789");
            }
            if symbols {
                charset.push_str("!@#$%^&*()-_=+");
            }
            if charset.is_empty() {
                bail!("token charset is empty: at least one character class must be enabled");
            }
            if length == 0 {
                bail!("token length must be greater than zero");
            }
            let chars: Vec<char> = charset.chars().collect();
            let mut rng = rand::thread_rng();
            for _ in 0..count {
                let token: String = (0..length)
                    .map(|_| chars[rng.gen_range(0..chars.len())])
                    .collect();
                println!("{token}");
            }
        }
    }
    Ok(())
}
