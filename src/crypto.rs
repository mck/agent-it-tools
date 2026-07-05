use crate::util::{print_json, read_input};
use anyhow::{bail, Context, Result};
use clap::Subcommand;
use md5::Md5;
use rand::Rng;
use sha1::Sha1;
use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum CryptoCmd {
    /// Compute a hash digest (hex) of the input text or a file
    Hash {
        /// Algorithm: md5 | sha1 | sha224 | sha256 | sha384 | sha512
        #[arg(short, long, default_value = "sha256")]
        algo: String,
        /// Hash the raw bytes of this file instead of text input
        #[arg(short, long, conflicts_with = "input")]
        file: Option<PathBuf>,
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
    /// Compute an HMAC signature (hex) of the input text
    Hmac {
        /// Algorithm: md5 | sha1 | sha256 | sha512
        #[arg(short, long, default_value = "sha256")]
        algo: String,
        /// Secret key as a literal argument (fine for test values; real secrets
        /// should use --key-env or --key-file so they never appear in argv)
        #[arg(short, long, conflicts_with_all = ["key_env", "key_file"])]
        key: Option<String>,
        /// Read the key from this environment variable
        #[arg(long, conflicts_with = "key_file")]
        key_env: Option<String>,
        /// Read the key from a file (one trailing newline stripped)
        #[arg(long)]
        key_file: Option<PathBuf>,
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
    /// Generate random token strings
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
    /// Hash a password with bcrypt
    BcryptHash {
        /// Cost factor (4-31)
        #[arg(short, long, default_value_t = 10)]
        cost: u32,
        /// Password (reads stdin if omitted)
        input: Option<String>,
    },
    /// Verify a password against a bcrypt hash
    BcryptVerify {
        /// Existing bcrypt hash to check against
        #[arg(long)]
        hash: String,
        /// Password (reads stdin if omitted)
        input: Option<String>,
    },
    /// Generate or verify a TOTP code (RFC 6238: SHA1, 6 digits, 30s period)
    Otp {
        /// Base32 secret as a literal (real secrets should use --secret-env)
        #[arg(short, long, conflicts_with = "secret_env")]
        secret: Option<String>,
        /// Read the Base32 secret from this environment variable
        #[arg(long)]
        secret_env: Option<String>,
        /// Verify this code instead of generating one (exit 2 on mismatch)
        #[arg(long)]
        check: Option<String>,
    },
}

fn hex_digest<D: Digest>(data: &[u8]) -> String {
    hex::encode(D::digest(data))
}

macro_rules! hmac_hex {
    ($alg:ty, $key:expr, $data:expr) => {{
        use hmac::{Hmac, KeyInit, Mac};
        let mut mac = Hmac::<$alg>::new_from_slice($key)?;
        mac.update($data);
        hex::encode(mac.finalize().into_bytes())
    }};
}

pub fn run(cmd: CryptoCmd) -> Result<()> {
    match cmd {
        CryptoCmd::Hash { algo, file, input } => {
            let data = match file {
                Some(path) => std::fs::read(&path)
                    .with_context(|| format!("cannot read file '{}'", path.display()))?,
                None => read_input(input)?.into_bytes(),
            };
            let data = data.as_slice();
            let digest = match algo.to_lowercase().as_str() {
                "md5" => hex_digest::<Md5>(data),
                "sha1" => hex_digest::<Sha1>(data),
                "sha224" => hex_digest::<Sha224>(data),
                "sha256" => hex_digest::<Sha256>(data),
                "sha384" => hex_digest::<Sha384>(data),
                "sha512" => hex_digest::<Sha512>(data),
                other => bail!("unsupported hash algorithm: {other} (expected md5, sha1, sha224, sha256, sha384 or sha512)"),
            };
            println!("{digest}");
        }
        CryptoCmd::Hmac {
            algo,
            key,
            key_env,
            key_file,
            input,
        } => {
            let data = read_input(input)?;
            let key = match (key, key_env, key_file) {
                (Some(k), None, None) => k,
                (None, Some(var), None) => std::env::var(&var)
                    .with_context(|| format!("environment variable '{var}' is not set"))?,
                (None, None, Some(path)) => {
                    let mut k = std::fs::read_to_string(&path)
                        .with_context(|| format!("cannot read key file '{}'", path.display()))?;
                    if k.ends_with('\n') {
                        k.pop();
                        if k.ends_with('\r') {
                            k.pop();
                        }
                    }
                    k
                }
                _ => bail!("provide exactly one of --key, --key-env or --key-file"),
            };
            let key = key.as_bytes();
            let data = data.as_bytes();
            let sig = match algo.to_lowercase().as_str() {
                "md5" => hmac_hex!(Md5, key, data),
                "sha1" => hmac_hex!(Sha1, key, data),
                "sha256" => hmac_hex!(Sha256, key, data),
                "sha512" => hmac_hex!(Sha512, key, data),
                other => bail!(
                    "unsupported HMAC algorithm: {other} (expected md5, sha1, sha256 or sha512)"
                ),
            };
            println!("{sig}");
        }
        CryptoCmd::Token {
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
        CryptoCmd::BcryptHash { cost, input } => {
            let password = read_input(input)?;
            println!("{}", bcrypt::hash(password, cost)?);
        }
        CryptoCmd::BcryptVerify { hash, input } => {
            let password = read_input(input)?;
            let valid = bcrypt::verify(password, &hash)?;
            print_json(&serde_json::json!({ "valid": valid }))?;
            if !valid {
                std::process::exit(2);
            }
        }
        CryptoCmd::Otp {
            secret,
            secret_env,
            check,
        } => {
            let secret = match (secret, secret_env) {
                (Some(s), None) => s,
                (None, Some(var)) => std::env::var(&var)
                    .with_context(|| format!("environment variable '{var}' is not set"))?,
                _ => bail!("provide exactly one of --secret or --secret-env"),
            };
            let bytes = totp_rs::Secret::Encoded(secret.trim().to_string())
                .to_bytes()
                .map_err(|e| anyhow::anyhow!("invalid Base32 secret: {e:?}"))?;
            let totp = totp_rs::TOTP::new_unchecked(totp_rs::Algorithm::SHA1, 6, 1, 30, bytes);
            match check {
                Some(code) => {
                    let valid = totp
                        .check_current(code.trim())
                        .context("system clock error")?;
                    print_json(&serde_json::json!({ "valid": valid }))?;
                    if !valid {
                        std::process::exit(2);
                    }
                }
                None => {
                    let code = totp.generate_current().context("system clock error")?;
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .context("system clock error")?
                        .as_secs();
                    print_json(&serde_json::json!({
                        "code": code,
                        "seconds_remaining": 30 - (now % 30),
                    }))?;
                }
            }
        }
    }
    Ok(())
}
