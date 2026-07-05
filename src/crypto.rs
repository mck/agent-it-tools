use crate::util::{print_json, read_input};
use anyhow::{bail, Result};
use clap::Subcommand;
use md5::Md5;
use rand::Rng;
use sha1::Sha1;
use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};

#[derive(Subcommand)]
pub enum CryptoCmd {
    /// Compute a hash digest (hex) of the input text
    Hash {
        /// Algorithm: md5 | sha1 | sha224 | sha256 | sha384 | sha512
        #[arg(short, long, default_value = "sha256")]
        algo: String,
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
    /// Compute an HMAC signature (hex) of the input text
    Hmac {
        /// Algorithm: md5 | sha1 | sha256 | sha512
        #[arg(short, long, default_value = "sha256")]
        algo: String,
        /// Secret key
        #[arg(short, long)]
        key: String,
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
}

fn hex_digest<D: Digest>(data: &[u8]) -> String {
    hex::encode(D::digest(data))
}

macro_rules! hmac_hex {
    ($alg:ty, $key:expr, $data:expr) => {{
        use hmac::{Hmac, Mac};
        let mut mac = Hmac::<$alg>::new_from_slice($key)?;
        mac.update($data);
        hex::encode(mac.finalize().into_bytes())
    }};
}

pub fn run(cmd: CryptoCmd) -> Result<()> {
    match cmd {
        CryptoCmd::Hash { algo, input } => {
            let data = read_input(input)?;
            let data = data.as_bytes();
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
        CryptoCmd::Hmac { algo, key, input } => {
            let data = read_input(input)?;
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
    }
    Ok(())
}
