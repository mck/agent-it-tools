//! Self-description layer.
//!
//! Every tool has a hand-authored spec file under `specs/<category>/<tool>.toml`
//! that is embedded into the binary at compile time. The spec carries what clap
//! cannot know (when to use the tool, what the output means, verified examples);
//! clap introspection contributes the argument schema. The merged result is the
//! *catalog*: the single source from which the skill file, the Claude Code
//! plugin, the OpenAI tool definitions and the golden test suite are derived.

use anyhow::{bail, Context, Result};
use clap::CommandFactory;
use clap::Subcommand;
use include_dir::{include_dir, Dir};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

static SPEC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/specs");

/// Categories that carry user-facing tools. `meta` itself is infrastructure
/// and intentionally excluded from the catalog.
const TOOL_CATEGORIES: [&str; 5] = ["crypto", "converter", "web", "development", "datetime"];

#[derive(Subcommand)]
pub enum MetaCmd {
    /// Print the full machine-readable tool catalog (JSON)
    Catalog,
    /// Print schema, flags and verified examples for one tool (JSON)
    Describe {
        /// Category, e.g. crypto
        category: String,
        /// Tool name, e.g. hash
        tool: String,
    },
    /// Compile distribution artifacts (skill file, plugin, OpenAI tools) from the catalog
    Export {
        /// Target: catalog | skill | plugin | openai | all
        #[arg(short, long, default_value = "all")]
        target: String,
        /// Output directory
        #[arg(short, long, default_value = "dist")]
        out: PathBuf,
    },
    /// Coverage report against the upstream sharevb/it-tools taxonomy
    Parity {
        /// List every upstream tool id with its status
        #[arg(long)]
        full: bool,
    },
}

// ---------------------------------------------------------------------------
// Spec files (hand-authored data, embedded at compile time)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    /// Command path, e.g. "crypto hash"
    pub path: String,
    /// One-line purpose, written for an agent deciding whether to use it
    pub summary: String,
    /// When to reach for this tool (and when not to)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when_to_use: Option<String>,
    pub output: OutputSpec,
    /// Upstream it-tools ids this covers (parity tracking)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub it_tools: Vec<String>,
    /// Verified examples: these are executed as golden tests in CI
    #[serde(default)]
    pub examples: Vec<Example>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSpec {
    /// "text" (plain lines) or "json" (pretty-printed object)
    pub kind: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    /// Full argv after the binary name
    pub argv: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdin: Option<String>,
    /// Exact expected stdout (trailing newlines ignored)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    /// For non-deterministic output: a regex the stdout must match
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdout_regex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

fn collect_spec_files<'a>(dir: &'a Dir<'a>, out: &mut Vec<&'a include_dir::File<'a>>) {
    for file in dir.files() {
        if file.path().extension().is_some_and(|e| e == "toml") {
            out.push(file);
        }
    }
    for sub in dir.dirs() {
        collect_spec_files(sub, out);
    }
}

/// Load all embedded tool specs, keyed by command path ("crypto hash").
pub fn load_specs() -> Result<BTreeMap<String, ToolSpec>> {
    let mut files = Vec::new();
    collect_spec_files(&SPEC_DIR, &mut files);
    let mut specs = BTreeMap::new();
    for file in files {
        if file.path() == Path::new("parity.toml") {
            continue;
        }
        let text = file
            .contents_utf8()
            .with_context(|| format!("spec {} is not UTF-8", file.path().display()))?;
        let spec: ToolSpec = toml::from_str(text)
            .with_context(|| format!("invalid spec file {}", file.path().display()))?;
        if specs.insert(spec.path.clone(), spec).is_some() {
            bail!("duplicate spec for path in {}", file.path().display());
        }
    }
    Ok(specs)
}

// ---------------------------------------------------------------------------
// Parity data
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParityFile {
    pub upstream: Vec<ParityEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParityEntry {
    /// Upstream tool id (directory name in sharevb/it-tools src/tools)
    pub id: String,
    /// implemented | planned | not-applicable
    pub status: String,
    /// Our command path(s) covering it, if implemented
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

pub fn load_parity() -> Result<ParityFile> {
    let file = SPEC_DIR
        .get_file("parity.toml")
        .context("embedded specs/parity.toml missing")?;
    let text = file.contents_utf8().context("parity.toml is not UTF-8")?;
    toml::from_str(text).context("invalid parity.toml")
}

// ---------------------------------------------------------------------------
// Catalog (clap introspection + specs merged)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct CatalogTool {
    pub path: String,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when_to_use: Option<String>,
    pub output: OutputSpec,
    /// True when the main input may be piped via stdin instead of the
    /// positional argument
    pub stdin_fallback: bool,
    pub args: Vec<ArgSchema>,
    pub examples: Vec<Example>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub it_tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArgSchema {
    pub name: String,
    /// positional | option | flag
    pub kind: String,
    /// string | integer | boolean
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short: Option<char>,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

/// All leaf command paths of the four tool categories, e.g. "crypto hash".
pub fn leaf_paths() -> Vec<String> {
    let root = crate::Cli::command();
    let mut paths = Vec::new();
    for cat in root.get_subcommands() {
        if !TOOL_CATEGORIES.contains(&cat.get_name()) {
            continue;
        }
        for leaf in cat.get_subcommands() {
            if leaf.get_name() == "help" {
                continue;
            }
            paths.push(format!("{} {}", cat.get_name(), leaf.get_name()));
        }
    }
    paths
}

fn arg_schema(arg: &clap::Arg) -> ArgSchema {
    let is_bool = matches!(
        arg.get_action(),
        clap::ArgAction::SetTrue | clap::ArgAction::SetFalse
    );
    let default = arg
        .get_default_values()
        .first()
        .map(|v| v.to_string_lossy().into_owned());
    let value_type = if is_bool {
        "boolean"
    } else if default.as_deref().is_some_and(|d| d.parse::<i64>().is_ok()) {
        "integer"
    } else {
        "string"
    };
    ArgSchema {
        name: arg.get_id().to_string(),
        kind: if arg.is_positional() {
            "positional"
        } else if is_bool {
            "flag"
        } else {
            "option"
        }
        .to_string(),
        value_type: value_type.to_string(),
        long: arg.get_long().map(str::to_string),
        short: arg.get_short(),
        required: arg.is_required_set(),
        default,
        help: arg.get_help().map(|h| h.to_string()),
    }
}

/// Build the merged catalog. Fails loudly if any CLI leaf lacks a spec:
/// the same invariant the test suite enforces.
pub fn build_catalog() -> Result<Vec<CatalogTool>> {
    let specs = load_specs()?;
    let root = crate::Cli::command();
    let mut tools = Vec::new();
    for cat in root.get_subcommands() {
        if !TOOL_CATEGORIES.contains(&cat.get_name()) {
            continue;
        }
        for leaf in cat.get_subcommands() {
            if leaf.get_name() == "help" {
                continue;
            }
            let path = format!("{} {}", cat.get_name(), leaf.get_name());
            let spec = specs
                .get(&path)
                .with_context(|| format!("missing spec file for tool '{path}'"))?;
            let args: Vec<ArgSchema> = leaf
                .get_arguments()
                .filter(|a| !matches!(a.get_id().as_str(), "help" | "version"))
                .map(arg_schema)
                .collect();
            let stdin_fallback = args
                .iter()
                .any(|a| a.kind == "positional" && a.name == "input" && !a.required);
            tools.push(CatalogTool {
                path,
                summary: spec.summary.clone(),
                when_to_use: spec.when_to_use.clone(),
                output: spec.output.clone(),
                stdin_fallback,
                args,
                examples: spec.examples.clone(),
                it_tools: spec.it_tools.clone(),
            });
        }
    }
    Ok(tools)
}

fn catalog_document() -> Result<serde_json::Value> {
    let tools = build_catalog()?;
    let parity = load_parity()?;
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for e in &parity.upstream {
        *counts.entry(e.status.as_str()).or_default() += 1;
    }
    Ok(serde_json::json!({
        "name": "agent-it-tools",
        "version": env!("CARGO_PKG_VERSION"),
        "io_contract": {
            "success": "result on stdout (plain text or pretty JSON, see each tool's output.kind)",
            "failure": "{\"error\": \"reason\"} on stderr, non-zero exit code",
            "input": "tools with stdin_fallback=true accept the main input as the trailing positional argument or piped via stdin"
        },
        "tools": tools,
        "parity": {
            "upstream": "https://github.com/sharevb/it-tools",
            "upstream_total": parity.upstream.len(),
            "counts": counts,
        },
    }))
}

// ---------------------------------------------------------------------------
// Artifact generation
// ---------------------------------------------------------------------------

fn skill_markdown(tools: &[CatalogTool]) -> String {
    let mut by_cat: BTreeMap<&str, Vec<&CatalogTool>> = BTreeMap::new();
    for t in tools {
        let cat = t.path.split(' ').next().unwrap_or_default();
        by_cat.entry(cat).or_default().push(t);
    }

    let mut md = String::new();
    md.push_str("---\nname: agent-it-tools\n");
    md.push_str("description: MUST BE USED for any request involving hashes (md5/sha), HMAC, bcrypt, random tokens, UUIDs, base64, hex, URL/HTML encoding or decoding, JSON/YAML/TOML conversion, JSON formatting, case conversion, number bases, unix timestamps, JWT decoding, URL or user-agent parsing, slugs, cron expressions, regex testing, or text diffs. Never answer these from memory, even when the answer seems obvious. Language models get encodings, digests and slugs subtly wrong; this local CLI computes them exactly.\n---\n\n");
    md.push_str("# agent-it-tools\n\n");
    md.push_str("Run: `agent-it-tools <category> <tool> [flags] [input]`\n\n");
    md.push_str("Rules:\n");
    md.push_str("- Pass the main input as the FINAL argument, in quotes: `agent-it-tools crypto hmac --algo sha256 --key K \"message\"`. Only pipe via stdin for multiline data; the command must always start with `agent-it-tools`.\n");
    md.push_str("- Success: result on stdout. Failure: `{\"error\":\"...\"}` on stderr with non-zero exit: read stderr, fix the call.\n");
    md.push_str(
        "- Never compute hashes, encodings, slugs or conversions yourself, even trivial-looking ones. Always run the tool and report its exact output.\n",
    );
    md.push_str("- Need flags or an example for a tool? Run `agent-it-tools meta describe <category> <tool>`: it returns the full JSON schema with verified examples. Do this instead of guessing.\n\n");
    md.push_str("## Tools\n");
    for cat in TOOL_CATEGORIES {
        let Some(list) = by_cat.get(cat) else {
            continue;
        };
        md.push_str(&format!("\n### {cat}\n"));
        for t in list {
            md.push_str(&format!("- `{}`: {}\n", t.path, t.summary));
        }
    }
    md.push_str("\n## Canonical examples\n\n```sh\n");
    md.push_str("agent-it-tools crypto hash --algo sha256 \"hello\"\n");
    md.push_str("cat data.json | agent-it-tools converter data --from json --to yaml\n");
    md.push_str(
        "agent-it-tools web jwt \"$TOKEN\"          # decode header/payload, no verification\n",
    );
    md.push_str("agent-it-tools development cron \"*/15 9-17 * * 1-5\" --count 3\n");
    md.push_str("agent-it-tools meta describe converter case   # full schema for one tool\n");
    md.push_str("```\n");
    md
}

fn openai_tools(tools: &[CatalogTool]) -> serde_json::Value {
    let items: Vec<serde_json::Value> = tools
        .iter()
        .map(|t| {
            let mut properties = serde_json::Map::new();
            let mut required = Vec::new();
            for a in &t.args {
                let mut prop = serde_json::Map::new();
                prop.insert(
                    "type".into(),
                    serde_json::Value::String(match a.value_type.as_str() {
                        "boolean" => "boolean".into(),
                        "integer" => "integer".into(),
                        _ => "string".to_string(),
                    }),
                );
                let mut desc = a.help.clone().unwrap_or_default();
                if let Some(d) = &a.default {
                    desc.push_str(&format!(" (default: {d})"));
                }
                if !desc.is_empty() {
                    prop.insert("description".into(), serde_json::Value::String(desc));
                }
                properties.insert(a.name.clone(), serde_json::Value::Object(prop));
                // Function calling has no stdin, so the main input is required here.
                if a.required || (a.kind == "positional" && a.name == "input") {
                    required.push(serde_json::Value::String(a.name.clone()));
                }
            }
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": t.path.replace([' ', '-'], "_"),
                    "description": format!("{} Returns: {}", t.summary, t.output.description),
                    "parameters": {
                        "type": "object",
                        "properties": properties,
                        "required": required,
                    },
                    "x-argv-template": t.path,
                }
            })
        })
        .collect();
    serde_json::Value::Array(items)
}

fn plugin_manifest() -> serde_json::Value {
    serde_json::json!({
        "name": "agent-it-tools",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Language models predict; this binary computes. Deterministic developer utilities (hashing, encoding, conversion, JWT, cron, regex, diff) as a fast local CLI built for agents",
        "author": { "name": "Marco-Christian Krenn", "email": "hey@mck.systems" },
        "homepage": "https://github.com/mck/agent-it-tools",
        "keywords": ["it-tools", "cli", "utilities", "hashing", "encoding"]
    })
}

fn write_artifact(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content).with_context(|| format!("cannot write {}", path.display()))?;
    println!("wrote {}", path.display());
    Ok(())
}

pub fn run(cmd: MetaCmd) -> Result<()> {
    match cmd {
        MetaCmd::Catalog => {
            println!("{}", serde_json::to_string_pretty(&catalog_document()?)?);
        }
        MetaCmd::Describe { category, tool } => {
            let path = format!("{category} {tool}");
            let tools = build_catalog()?;
            match tools.iter().find(|t| t.path == path) {
                Some(t) => println!("{}", serde_json::to_string_pretty(t)?),
                None => bail!(
                    "unknown tool '{path}': run 'agent-it-tools meta catalog' for the full list"
                ),
            }
        }
        MetaCmd::Export { target, out } => {
            let tools = build_catalog()?;
            let want = |t: &str| target == t || target == "all";
            let mut matched = false;
            if want("catalog") {
                matched = true;
                write_artifact(
                    &out.join("catalog.json"),
                    &serde_json::to_string_pretty(&catalog_document()?)?,
                )?;
            }
            if want("skill") {
                matched = true;
                write_artifact(
                    &out.join("skill/agent-it-tools/SKILL.md"),
                    &skill_markdown(&tools),
                )?;
            }
            if want("plugin") {
                matched = true;
                let base = out.join("claude-plugin");
                write_artifact(
                    &base.join(".claude-plugin/plugin.json"),
                    &serde_json::to_string_pretty(&plugin_manifest())?,
                )?;
                write_artifact(
                    &base.join("skills/agent-it-tools/SKILL.md"),
                    &skill_markdown(&tools),
                )?;
            }
            if want("openai") {
                matched = true;
                write_artifact(
                    &out.join("openai-tools.json"),
                    &serde_json::to_string_pretty(&openai_tools(&tools))?,
                )?;
            }
            if !matched {
                bail!("unknown export target '{target}' (expected catalog, skill, plugin, openai or all)");
            }
        }
        MetaCmd::Parity { full } => {
            let parity = load_parity()?;
            let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
            for e in &parity.upstream {
                *counts.entry(e.status.as_str()).or_default() += 1;
            }
            if full {
                println!("{}", serde_json::to_string_pretty(&parity.upstream)?);
            } else {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "upstream_total": parity.upstream.len(),
                        "counts": counts,
                    }))?
                );
            }
        }
    }
    Ok(())
}
