use crate::util::read_input;
use anyhow::{Context, Result};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum JsonCmd {
    /// Prettify or minify a JSON document (also validates it)
    Format {
        /// Emit minified JSON instead of pretty-printed
        #[arg(long)]
        minify: bool,
        /// Input JSON (reads stdin if omitted)
        input: Option<String>,
    },
    /// Run a jq filter on JSON input
    #[command(visible_alias = "jq")]
    Query {
        /// jq filter, e.g. '.items[].name'
        #[arg(short, long)]
        filter: String,
        /// JSON input (reads stdin if omitted)
        input: Option<String>,
    },
    /// Diff two JSON documents as an RFC 6902 patch (empty array = equal)
    Diff {
        /// Old JSON document, or a file path with --files
        old: String,
        /// New JSON document, or a file path with --files
        new: String,
        /// Treat the two arguments as file paths
        #[arg(long)]
        files: bool,
    },
    /// Deep-merge a JSON merge patch into a base document (RFC 7396)
    Merge {
        /// Base JSON document, or a file path with --files
        base: String,
        /// Merge patch JSON, or a file path with --files
        patch: String,
        /// Treat the two arguments as file paths
        #[arg(long)]
        files: bool,
    },
    /// Flatten nested JSON to dot-notation keys (reverse with --nestify)
    Flatten {
        /// Rebuild nested JSON from flattened keys
        #[arg(long)]
        nestify: bool,
        /// Key separator
        #[arg(short, long, default_value = ".")]
        separator: String,
        /// JSON input (reads stdin if omitted)
        input: Option<String>,
    },
    /// Escape text as a JSON string literal (reverse with --unescape)
    Escape {
        /// Unescape a quoted string literal instead
        #[arg(short, long)]
        unescape: bool,
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
}

fn arg_or_file(value: String, is_file: bool, what: &str) -> Result<String> {
    if is_file {
        std::fs::read_to_string(&value)
            .with_context(|| format!("cannot read {what} file '{value}'"))
    } else {
        Ok(value)
    }
}

fn flatten_value(
    prefix: &str,
    sep: &str,
    value: &serde_json::Value,
    out: &mut serde_json::Map<String, serde_json::Value>,
) {
    match value {
        serde_json::Value::Object(map) if !map.is_empty() => {
            for (k, v) in map {
                let key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}{sep}{k}")
                };
                flatten_value(&key, sep, v, out);
            }
        }
        serde_json::Value::Array(items) if !items.is_empty() => {
            for (i, v) in items.iter().enumerate() {
                let key = if prefix.is_empty() {
                    i.to_string()
                } else {
                    format!("{prefix}{sep}{i}")
                };
                flatten_value(&key, sep, v, out);
            }
        }
        other => {
            out.insert(prefix.to_string(), other.clone());
        }
    }
}

fn nestify_value(
    flat: &serde_json::Map<String, serde_json::Value>,
    sep: &str,
) -> serde_json::Value {
    fn insert_path(node: &mut serde_json::Value, segments: &[&str], value: &serde_json::Value) {
        if !node.is_object() {
            *node = serde_json::Value::Object(serde_json::Map::new());
        }
        let map = node.as_object_mut().expect("just ensured object");
        match segments {
            [] => {}
            [last] => {
                map.insert(last.to_string(), value.clone());
            }
            [head, rest @ ..] => {
                let entry = map
                    .entry(head.to_string())
                    .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
                insert_path(entry, rest, value);
            }
        }
    }

    let mut root = serde_json::Value::Object(serde_json::Map::new());
    for (path, value) in flat {
        let segments: Vec<&str> = path.split(sep).collect();
        insert_path(&mut root, &segments, value);
    }
    // Fold objects whose keys are exactly 0..n into arrays.
    fn fold_arrays(value: serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => {
                let folded: serde_json::Map<String, serde_json::Value> =
                    map.into_iter().map(|(k, v)| (k, fold_arrays(v))).collect();
                let is_index_run = !folded.is_empty()
                    && folded
                        .keys()
                        .enumerate()
                        .all(|(i, k)| k.parse::<usize>() == Ok(i));
                if is_index_run {
                    serde_json::Value::Array(folded.into_iter().map(|(_, v)| v).collect())
                } else {
                    serde_json::Value::Object(folded)
                }
            }
            other => other,
        }
    }
    fold_arrays(root)
}

fn run_jq(filter_str: &str, input: &str) -> Result<Vec<serde_json::Value>> {
    use jaq_core::load::{Arena, File, Loader};
    use jaq_core::{data, unwrap_valr, Compiler, Ctx, Vars};
    use jaq_json::{read, Val};

    let input_val = read::parse_single(input.as_bytes())
        .map_err(|e| anyhow::anyhow!("invalid JSON input: {e}"))?;
    let program = File {
        code: filter_str,
        path: (),
    };
    let defs = jaq_core::defs()
        .chain(jaq_std::defs())
        .chain(jaq_json::defs());
    let funs = jaq_core::funs()
        .chain(jaq_std::funs())
        .chain(jaq_json::funs());
    let loader = Loader::new(defs);
    let arena = Arena::default();
    let modules = loader
        .load(&arena, program)
        .map_err(|_| anyhow::anyhow!("invalid jq filter: parse error in '{filter_str}'"))?;
    let filter = Compiler::default()
        .with_funs(funs)
        .compile(modules)
        .map_err(|_| {
            anyhow::anyhow!("invalid jq filter: unknown function or binding in '{filter_str}'")
        })?;
    let ctx = Ctx::<data::JustLut<Val>>::new(&filter.lut, Vars::new([]));
    let mut out = Vec::new();
    for result in filter.id.run((ctx, input_val)).map(unwrap_valr) {
        let val = result.map_err(|e| anyhow::anyhow!("jq runtime error: {e}"))?;
        let rendered = val.to_string();
        out.push(serde_json::from_str(&rendered).unwrap_or(serde_json::Value::String(rendered)));
    }
    Ok(out)
}

pub fn run(cmd: JsonCmd) -> Result<()> {
    match cmd {
        JsonCmd::Format { minify, input } => {
            let input = read_input(input)?;
            let value: serde_json::Value =
                serde_json::from_str(&input).context("invalid JSON input")?;
            if minify {
                println!("{}", serde_json::to_string(&value)?);
            } else {
                println!("{}", serde_json::to_string_pretty(&value)?);
            }
        }
        JsonCmd::Query { filter, input } => {
            let input = read_input(input)?;
            let results = run_jq(&filter, &input)?;
            match results.as_slice() {
                [single] => println!("{}", serde_json::to_string_pretty(single)?),
                many => {
                    for v in many {
                        println!("{}", serde_json::to_string(v)?);
                    }
                }
            }
        }
        JsonCmd::Diff { old, new, files } => {
            let old_text = arg_or_file(old, files, "old")?;
            let new_text = arg_or_file(new, files, "new")?;
            let old_value: serde_json::Value =
                serde_json::from_str(&old_text).context("invalid JSON in old document")?;
            let new_value: serde_json::Value =
                serde_json::from_str(&new_text).context("invalid JSON in new document")?;
            let patch = json_patch::diff(&old_value, &new_value);
            println!("{}", serde_json::to_string_pretty(&patch)?);
        }
        JsonCmd::Merge { base, patch, files } => {
            let base_text = arg_or_file(base, files, "base")?;
            let patch_text = arg_or_file(patch, files, "patch")?;
            let mut base_value: serde_json::Value =
                serde_json::from_str(&base_text).context("invalid JSON in base document")?;
            let patch_value: serde_json::Value =
                serde_json::from_str(&patch_text).context("invalid JSON in patch document")?;
            json_patch::merge(&mut base_value, &patch_value);
            println!("{}", serde_json::to_string_pretty(&base_value)?);
        }
        JsonCmd::Flatten {
            nestify,
            separator,
            input,
        } => {
            let input = read_input(input)?;
            let value: serde_json::Value =
                serde_json::from_str(&input).context("invalid JSON input")?;
            if nestify {
                let flat = value
                    .as_object()
                    .context("nestify expects a flat JSON object")?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&nestify_value(flat, &separator))?
                );
            } else {
                let mut out = serde_json::Map::new();
                flatten_value("", &separator, &value, &mut out);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::Value::Object(out))?
                );
            }
        }
        JsonCmd::Escape { unescape, input } => {
            let input = read_input(input)?;
            if unescape {
                let quoted = if input.starts_with('"') && input.ends_with('"') && input.len() >= 2 {
                    input.clone()
                } else {
                    format!("\"{input}\"")
                };
                let unescaped: String = serde_json::from_str(&quoted)
                    .context("input is not a valid escaped string literal")?;
                println!("{unescaped}");
            } else {
                println!("{}", serde_json::to_string(&input)?);
            }
        }
    }
    Ok(())
}
