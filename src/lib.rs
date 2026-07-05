pub mod converter;
pub mod crypto;
pub mod datetime;
pub mod development;
pub mod meta;
pub mod network;
pub mod text;
pub mod util;
pub mod web;

use clap::{Parser, Subcommand};

/// Agent-first CLI port of the it-tools utility suite.
///
/// Success output goes to stdout. Errors are emitted as JSON
/// (`{"error": "..."}`) on stderr with a non-zero exit code.
#[derive(Parser)]
#[command(name = "agent-it-tools", version, about, propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub category: Category,
}

#[derive(Subcommand)]
pub enum Category {
    /// Hashing, HMAC signatures, token generation, bcrypt
    #[command(subcommand)]
    Crypto(crypto::CryptoCmd),
    /// JSON/YAML/TOML, Base64, hex, case, number base, date-time
    #[command(subcommand)]
    Converter(converter::ConverterCmd),
    /// URL encoding/parsing, JWT, user-agent, HTML entities, slugs
    #[command(subcommand)]
    Web(web::WebCmd),
    /// Crontab, UUIDs, regex testing, text diff
    #[command(subcommand)]
    Development(development::DevCmd),
    /// Current time and timestamp/date conversion
    #[command(subcommand)]
    Datetime(datetime::DatetimeCmd),
    /// Subnets, CIDR math and IP representation
    #[command(subcommand)]
    Network(network::NetworkCmd),
    /// Text statistics, similarity and sensitive-data masking
    #[command(subcommand)]
    Text(text::TextCmd),
    /// Self-description: tool catalog, per-tool schemas, artifact export, upstream parity
    #[command(subcommand)]
    Meta(meta::MetaCmd),
}

pub fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.category {
        Category::Crypto(cmd) => crypto::run(cmd),
        Category::Converter(cmd) => converter::run(cmd),
        Category::Web(cmd) => web::run(cmd),
        Category::Development(cmd) => development::run(cmd),
        Category::Datetime(cmd) => datetime::run(cmd),
        Category::Network(cmd) => network::run(cmd),
        Category::Text(cmd) => text::run(cmd),
        Category::Meta(cmd) => meta::run(cmd),
    }
}
