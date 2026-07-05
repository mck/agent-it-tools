pub mod codec;
pub mod color;
pub mod crypto;
pub mod data;
pub mod generate;
pub mod http;
pub mod json;
pub mod markdown;
pub mod math;
pub mod meta;
pub mod network;
pub mod regex;
pub mod text;
pub mod time;
pub mod unix;
pub mod url;
pub mod util;

use clap::{Parser, Subcommand};

/// Agent-first CLI port of the it-tools utility suite.
///
/// Success output goes to stdout. Errors are emitted as JSON
/// (`{"error": "..."}`) on stderr with a non-zero exit code.
#[derive(Parser)]
#[command(name = "ait", version, about, propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub category: Category,
}

#[derive(Subcommand)]
pub enum Category {
    /// JSON operations: format, query (jq), diff, merge, flatten, escape
    #[command(subcommand)]
    Json(json::JsonCmd),
    /// Structured-data format conversion and linting (JSON/YAML/TOML/XML/CSV)
    #[command(subcommand)]
    Data(data::DataCmd),
    /// Encode text: base64, hex, gzip, punycode, html entities
    #[command(subcommand)]
    Encode(codec::EncodeCmd),
    /// Decode text: base64, hex, gzip, punycode, html entities
    #[command(subcommand)]
    Decode(codec::DecodeCmd),
    /// URL operations: percent-encoding, parsing, building
    #[command(subcommand)]
    Url(url::UrlCmd),
    /// JSON Web Token inspection
    #[command(subcommand)]
    Jwt(http::JwtCmd),
    /// Hashing, HMAC signatures, bcrypt, TOTP
    #[command(subcommand)]
    Crypto(crypto::CryptoCmd),
    /// Generate identifiers and secrets: uuid, ulid, nanoid, token
    #[command(subcommand)]
    Generate(generate::GenerateCmd),
    /// Text operations: case, slugs, stats, similarity, masking, diff
    #[command(subcommand)]
    Text(text::TextCmd),
    /// Regular expression testing
    #[command(subcommand)]
    Regex(regex::RegexCmd),
    /// Time: now, conversion, duration arithmetic, timezones, cron
    #[command(subcommand)]
    Time(time::TimeCmd),
    /// HTTP helpers: basic auth, user-agent parsing, MIME types
    #[command(subcommand)]
    Http(http::HttpCmd),
    /// CSS colors: parsing and WCAG contrast
    #[command(subcommand)]
    Color(color::ColorCmd),
    /// Markdown/HTML conversion
    #[command(subcommand)]
    Markdown(markdown::MarkdownCmd),
    /// Subnets, CIDR math and IP representation
    #[command(subcommand)]
    Network(network::NetworkCmd),
    /// Math: expression evaluation, bitwise operations, number bases
    #[command(subcommand)]
    Math(math::MathCmd),
    /// Unix helpers: chmod notation
    #[command(subcommand)]
    Unix(unix::UnixCmd),
    /// Self-description: tool catalog, per-tool schemas, artifact export, upstream parity
    #[command(subcommand)]
    Meta(meta::MetaCmd),
}

pub fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.category {
        Category::Json(cmd) => json::run(cmd),
        Category::Data(cmd) => data::run(cmd),
        Category::Encode(cmd) => codec::run_encode(cmd),
        Category::Decode(cmd) => codec::run_decode(cmd),
        Category::Url(cmd) => url::run(cmd),
        Category::Jwt(cmd) => http::run_jwt(cmd),
        Category::Crypto(cmd) => crypto::run(cmd),
        Category::Generate(cmd) => generate::run(cmd),
        Category::Text(cmd) => text::run(cmd),
        Category::Regex(cmd) => regex::run(cmd),
        Category::Time(cmd) => time::run(cmd),
        Category::Http(cmd) => http::run_http(cmd),
        Category::Color(cmd) => color::run(cmd),
        Category::Markdown(cmd) => markdown::run(cmd),
        Category::Network(cmd) => network::run(cmd),
        Category::Math(cmd) => math::run(cmd),
        Category::Unix(cmd) => unix::run(cmd),
        Category::Meta(cmd) => meta::run(cmd),
    }
}
