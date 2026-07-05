use crate::util::read_input;
use anyhow::{bail, Context, Result};
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;
use clap::Subcommand;
use heck::{
    ToKebabCase, ToLowerCamelCase, ToShoutySnakeCase, ToSnakeCase, ToTitleCase, ToTrainCase,
    ToUpperCamelCase,
};

#[derive(Subcommand)]
pub enum ConverterCmd {
    /// Convert structured data between JSON, YAML and TOML
    Data {
        /// Source format: json | yaml | toml | xml
        #[arg(short, long)]
        from: String,
        /// Target format: json | yaml | toml | xml
        #[arg(short, long)]
        to: String,
        /// Input document (reads stdin if omitted)
        input: Option<String>,
    },
    /// Prettify or minify a JSON document
    JsonFormat {
        /// Emit minified JSON instead of pretty-printed
        #[arg(long)]
        minify: bool,
        /// Input JSON (reads stdin if omitted)
        input: Option<String>,
    },
    /// Encode text to Base64
    Base64Encode {
        /// Use URL-safe alphabet without padding
        #[arg(long)]
        url_safe: bool,
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
    /// Decode Base64 to text
    Base64Decode {
        /// Input Base64 (reads stdin if omitted); standard and URL-safe alphabets accepted
        input: Option<String>,
    },
    /// Encode text to a hex string
    HexEncode {
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
    /// Decode a hex string to text
    HexDecode {
        /// Input hex (reads stdin if omitted)
        input: Option<String>,
    },
    /// Convert a string between naming cases
    Case {
        /// Target case: camel | pascal | snake | constant | kebab | train | title | dot | path | lower | upper
        #[arg(short, long)]
        to: String,
        /// Input text (reads stdin if omitted)
        input: Option<String>,
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
    /// Convert CSV (header row required) to a JSON array of objects
    CsvToJson {
        /// Field delimiter
        #[arg(short, long, default_value = ",")]
        delimiter: String,
        /// CSV input (reads stdin if omitted)
        input: Option<String>,
    },
    /// Convert a JSON array of objects to CSV
    JsonToCsv {
        /// Field delimiter
        #[arg(short, long, default_value = ",")]
        delimiter: String,
        /// JSON input (reads stdin if omitted)
        input: Option<String>,
    },
    /// Validate JSON, YAML, TOML or XML syntax
    Lint {
        /// Format: json | yaml | toml | xml
        #[arg(short, long)]
        format: String,
        /// Input document (reads stdin if omitted)
        input: Option<String>,
    },
    /// Run a jq filter on JSON input
    JsonQuery {
        /// jq filter, e.g. '.items[].name'
        #[arg(short, long)]
        filter: String,
        /// JSON input (reads stdin if omitted)
        input: Option<String>,
    },
    /// Diff two JSON documents as an RFC 6902 patch (empty array = equal)
    JsonDiff {
        /// Old JSON document, or a file path with --files
        old: String,
        /// New JSON document, or a file path with --files
        new: String,
        /// Treat the two arguments as file paths
        #[arg(long)]
        files: bool,
    },
    /// Deep-merge a JSON merge patch into a base document (RFC 7396)
    JsonMerge {
        /// Base JSON document, or a file path with --files
        base: String,
        /// Merge patch JSON, or a file path with --files
        patch: String,
        /// Treat the two arguments as file paths
        #[arg(long)]
        files: bool,
    },
    /// Flatten nested JSON to dot-notation keys (reverse with --nestify)
    JsonFlatten {
        /// Rebuild nested JSON from flattened keys
        #[arg(long)]
        nestify: bool,
        /// Key separator
        #[arg(short, long, default_value = ".")]
        separator: String,
        /// JSON input (reads stdin if omitted)
        input: Option<String>,
    },
    /// Gzip-compress text and emit it as Base64
    GzipEncode {
        /// Input text (reads stdin if omitted)
        input: Option<String>,
    },
    /// Decompress Base64-encoded gzip data back to text
    GzipDecode {
        /// Input Base64 (reads stdin if omitted)
        input: Option<String>,
    },
}

fn parse_to_json(format: &str, input: &str) -> Result<serde_json::Value> {
    match format {
        "json" => serde_json::from_str(input).context("invalid JSON input"),
        "yaml" | "yml" => serde_yaml::from_str(input).context("invalid YAML input"),
        "toml" => toml::from_str(input).context("invalid TOML input"),
        "xml" => xml_to_json(input),
        other => bail!("unsupported source format: {other} (expected json, yaml, toml or xml)"),
    }
}

fn render_from_json(format: &str, value: &serde_json::Value) -> Result<String> {
    match format {
        "json" => Ok(serde_json::to_string_pretty(value)?),
        "yaml" | "yml" => Ok(serde_yaml::to_string(value)?.trim_end().to_string()),
        "toml" => toml::to_string_pretty(value)
            .context("value cannot be represented as TOML (e.g. null values or a non-table root)"),
        "xml" => json_to_xml(value),
        other => bail!("unsupported target format: {other} (expected json, yaml, toml or xml)"),
    }
}

/// XML -> JSON using the BadgerFish-like convention: attributes become
/// "@name" keys, text content becomes "#text" (or the value itself when the
/// element has no attributes/children), repeated sibling elements fold into
/// arrays.
fn xml_to_json(input: &str) -> Result<serde_json::Value> {
    use quick_xml::events::Event;
    use serde_json::{Map, Value};

    fn insert(map: &mut Map<String, Value>, key: String, value: Value) {
        match map.get_mut(&key) {
            None => {
                map.insert(key, value);
            }
            Some(Value::Array(arr)) => arr.push(value),
            Some(existing) => {
                let prev = existing.take();
                *existing = Value::Array(vec![prev, value]);
            }
        }
    }

    fn finalize(mut map: Map<String, Value>, text: String) -> Value {
        let text = text.trim().to_string();
        if map.is_empty() {
            return Value::String(text);
        }
        if !text.is_empty() {
            map.insert("#text".into(), Value::String(text));
        }
        Value::Object(map)
    }

    let mut reader = quick_xml::Reader::from_str(input);
    reader.config_mut().trim_text(true);
    // stack of (element name, attributes+children map, text buffer)
    let mut stack: Vec<(String, Map<String, Value>, String)> = Vec::new();
    let mut root = Map::new();

    let read_attrs = |e: &quick_xml::events::BytesStart| -> Result<Map<String, Value>> {
        let mut map = Map::new();
        for attr in e.attributes() {
            let attr = attr.context("invalid XML attribute")?;
            map.insert(
                format!("@{}", String::from_utf8_lossy(attr.key.as_ref())),
                serde_json::Value::String(
                    attr.normalized_value(quick_xml::XmlVersion::Implicit1_0)?
                        .into_owned(),
                ),
            );
        }
        Ok(map)
    };

    loop {
        match reader.read_event().context("invalid XML input")? {
            Event::Start(e) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                let attrs = read_attrs(&e)?;
                stack.push((name, attrs, String::new()));
            }
            Event::Empty(e) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                let attrs = read_attrs(&e)?;
                let value = finalize(attrs, String::new());
                match stack.last_mut() {
                    Some((_, parent, _)) => insert(parent, name, value),
                    None => insert(&mut root, name, value),
                }
            }
            Event::Text(t) => {
                if let Some((_, _, text)) = stack.last_mut() {
                    text.push_str(&t.xml10_content().context("invalid XML text")?);
                }
            }
            Event::CData(t) => {
                if let Some((_, _, text)) = stack.last_mut() {
                    text.push_str(&String::from_utf8_lossy(&t.into_inner()));
                }
            }
            Event::End(_) => {
                let (name, map, text) = stack.pop().context("unbalanced XML")?;
                let value = finalize(map, text);
                match stack.last_mut() {
                    Some((_, parent, _)) => insert(parent, name, value),
                    None => insert(&mut root, name, value),
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if root.is_empty() {
        bail!("invalid XML input: no root element found");
    }
    Ok(serde_json::Value::Object(root))
}

/// JSON -> XML, inverse of the convention above. A single-key object root
/// becomes the document element; anything else is wrapped in <root>.
fn json_to_xml(value: &serde_json::Value) -> Result<String> {
    use serde_json::Value;

    fn escape(text: &str) -> String {
        html_escape::encode_quoted_attribute(text).into_owned()
    }

    fn write(out: &mut String, name: &str, value: &Value, indent: usize) {
        let pad = "  ".repeat(indent);
        match value {
            Value::Array(items) => {
                for item in items {
                    write(out, name, item, indent);
                }
            }
            Value::Object(map) => {
                let attrs: String = map
                    .iter()
                    .filter_map(|(k, v)| {
                        k.strip_prefix('@')
                            .map(|a| format!(" {a}=\"{}\"", escape(v.as_str().unwrap_or_default())))
                    })
                    .collect();
                let children: Vec<(&String, &Value)> = map
                    .iter()
                    .filter(|(k, _)| !k.starts_with('@') && *k != "#text")
                    .collect();
                let text = map.get("#text").and_then(|t| t.as_str()).unwrap_or("");
                if children.is_empty() && text.is_empty() {
                    out.push_str(&format!("{pad}<{name}{attrs}/>\n"));
                } else if children.is_empty() {
                    out.push_str(&format!("{pad}<{name}{attrs}>{}</{name}>\n", escape(text)));
                } else {
                    out.push_str(&format!("{pad}<{name}{attrs}>\n"));
                    for (k, v) in children {
                        write(out, k, v, indent + 1);
                    }
                    if !text.is_empty() {
                        out.push_str(&format!("{}{}\n", "  ".repeat(indent + 1), escape(text)));
                    }
                    out.push_str(&format!("{pad}</{name}>\n"));
                }
            }
            scalar => {
                let text = match scalar {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                out.push_str(&format!("{pad}<{name}>{}</{name}>\n", escape(&text)));
            }
        }
    }

    let mut out = String::new();
    match value {
        Value::Object(map) if map.len() == 1 && !map.keys().next().unwrap().starts_with('@') => {
            let (name, inner) = map.iter().next().unwrap();
            write(&mut out, name, inner, 0);
        }
        other => write(&mut out, "root", other, 0),
    }
    Ok(out.trim_end().to_string())
}

fn csv_delimiter(delimiter: &str) -> Result<u8> {
    let bytes = match delimiter {
        "\\t" | "tab" => b"\t",
        other => other.as_bytes(),
    };
    if bytes.len() != 1 {
        bail!("delimiter must be a single byte (or 'tab')");
    }
    Ok(bytes[0])
}

/// Parse a CSV field into the most specific JSON scalar.
fn csv_scalar(field: &str) -> serde_json::Value {
    if let Ok(i) = field.parse::<i64>() {
        return serde_json::Value::from(i);
    }
    if let Ok(f) = field.parse::<f64>() {
        if field.contains('.') || field.contains('e') || field.contains('E') {
            return serde_json::Value::from(f);
        }
    }
    match field {
        "true" => serde_json::Value::Bool(true),
        "false" => serde_json::Value::Bool(false),
        other => serde_json::Value::String(other.to_string()),
    }
}

fn json_scalar_to_csv(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
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

fn arg_or_file(value: String, is_file: bool, what: &str) -> Result<String> {
    if is_file {
        std::fs::read_to_string(&value)
            .with_context(|| format!("cannot read {what} file '{value}'"))
    } else {
        Ok(value)
    }
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

pub fn run(cmd: ConverterCmd) -> Result<()> {
    match cmd {
        ConverterCmd::Data { from, to, input } => {
            let input = read_input(input)?;
            let value = parse_to_json(&from.to_lowercase(), &input)?;
            println!("{}", render_from_json(&to.to_lowercase(), &value)?);
        }
        ConverterCmd::JsonFormat { minify, input } => {
            let input = read_input(input)?;
            let value: serde_json::Value =
                serde_json::from_str(&input).context("invalid JSON input")?;
            if minify {
                println!("{}", serde_json::to_string(&value)?);
            } else {
                println!("{}", serde_json::to_string_pretty(&value)?);
            }
        }
        ConverterCmd::Base64Encode { url_safe, input } => {
            let input = read_input(input)?;
            let encoded = if url_safe {
                URL_SAFE_NO_PAD.encode(input.as_bytes())
            } else {
                STANDARD.encode(input.as_bytes())
            };
            println!("{encoded}");
        }
        ConverterCmd::Base64Decode { input } => {
            let input = read_input(input)?;
            let trimmed: String = input.split_whitespace().collect();
            let bytes = STANDARD
                .decode(&trimmed)
                .or_else(|_| URL_SAFE_NO_PAD.decode(&trimmed))
                .context("invalid Base64 input")?;
            let text =
                String::from_utf8(bytes).context("decoded Base64 payload is not valid UTF-8")?;
            println!("{text}");
        }
        ConverterCmd::HexEncode { input } => {
            let input = read_input(input)?;
            println!("{}", hex::encode(input.as_bytes()));
        }
        ConverterCmd::HexDecode { input } => {
            let input = read_input(input)?;
            let trimmed: String = input.split_whitespace().collect();
            let bytes =
                hex::decode(trimmed.trim_start_matches("0x")).context("invalid hex input")?;
            let text =
                String::from_utf8(bytes).context("decoded hex payload is not valid UTF-8")?;
            println!("{text}");
        }
        ConverterCmd::Case { to, input } => {
            let input = read_input(input)?;
            let out = match to.to_lowercase().as_str() {
                "camel" => input.to_lower_camel_case(),
                "pascal" => input.to_upper_camel_case(),
                "snake" => input.to_snake_case(),
                "constant" | "shouty" => input.to_shouty_snake_case(),
                "kebab" => input.to_kebab_case(),
                "train" => input.to_train_case(),
                "title" => input.to_title_case(),
                "dot" => input.to_snake_case().replace('_', "."),
                "path" => input.to_snake_case().replace('_', "/"),
                "lower" => input.to_lowercase(),
                "upper" => input.to_uppercase(),
                other => bail!(
                    "unsupported case: {other} (expected camel, pascal, snake, constant, kebab, train, title, dot, path, lower or upper)"
                ),
            };
            println!("{out}");
        }
        ConverterCmd::NumberBase { from, to, input } => {
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
        ConverterCmd::CsvToJson { delimiter, input } => {
            let input = read_input(input)?;
            let mut rdr = csv::ReaderBuilder::new()
                .delimiter(csv_delimiter(&delimiter)?)
                .from_reader(input.as_bytes());
            let headers = rdr.headers().context("invalid CSV input")?.clone();
            let mut rows = Vec::new();
            for record in rdr.records() {
                let record = record.context("invalid CSV input")?;
                let mut obj = serde_json::Map::new();
                for (header, field) in headers.iter().zip(record.iter()) {
                    obj.insert(header.to_string(), csv_scalar(field));
                }
                rows.push(serde_json::Value::Object(obj));
            }
            println!("{}", serde_json::to_string_pretty(&rows)?);
        }
        ConverterCmd::JsonToCsv { delimiter, input } => {
            let input = read_input(input)?;
            let value: serde_json::Value =
                serde_json::from_str(&input).context("invalid JSON input")?;
            let rows = value
                .as_array()
                .context("input must be a JSON array of objects")?;
            // Header: union of keys across all rows (alphabetical, since
            // serde_json maps are sorted in this build).
            let mut headers: Vec<String> = Vec::new();
            for row in rows {
                let obj = row
                    .as_object()
                    .context("every array item must be an object")?;
                for key in obj.keys() {
                    if !headers.iter().any(|h| h == key) {
                        headers.push(key.clone());
                    }
                }
            }
            let mut wtr = csv::WriterBuilder::new()
                .delimiter(csv_delimiter(&delimiter)?)
                .from_writer(Vec::new());
            wtr.write_record(&headers)?;
            for row in rows {
                let obj = row.as_object().expect("validated above");
                let record: Vec<String> = headers
                    .iter()
                    .map(|h| obj.get(h).map(json_scalar_to_csv).unwrap_or_default())
                    .collect();
                wtr.write_record(&record)?;
            }
            let bytes = wtr.into_inner().context("CSV write failed")?;
            print!(
                "{}",
                String::from_utf8(bytes).context("CSV output is not UTF-8")?
            );
        }
        ConverterCmd::Lint { format, input } => {
            let input = read_input(input)?;
            let format = format.to_lowercase();
            parse_to_json(&format, &input)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "valid": true,
                    "format": format,
                }))?
            );
        }
        ConverterCmd::JsonQuery { filter, input } => {
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
        ConverterCmd::JsonDiff { old, new, files } => {
            let old_text = arg_or_file(old, files, "old")?;
            let new_text = arg_or_file(new, files, "new")?;
            let old_value: serde_json::Value =
                serde_json::from_str(&old_text).context("invalid JSON in old document")?;
            let new_value: serde_json::Value =
                serde_json::from_str(&new_text).context("invalid JSON in new document")?;
            let patch = json_patch::diff(&old_value, &new_value);
            println!("{}", serde_json::to_string_pretty(&patch)?);
        }
        ConverterCmd::JsonMerge { base, patch, files } => {
            let base_text = arg_or_file(base, files, "base")?;
            let patch_text = arg_or_file(patch, files, "patch")?;
            let mut base_value: serde_json::Value =
                serde_json::from_str(&base_text).context("invalid JSON in base document")?;
            let patch_value: serde_json::Value =
                serde_json::from_str(&patch_text).context("invalid JSON in patch document")?;
            json_patch::merge(&mut base_value, &patch_value);
            println!("{}", serde_json::to_string_pretty(&base_value)?);
        }
        ConverterCmd::JsonFlatten {
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
        ConverterCmd::GzipEncode { input } => {
            use std::io::Write;
            let input = read_input(input)?;
            let mut encoder =
                flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(input.as_bytes())?;
            println!("{}", STANDARD.encode(encoder.finish()?));
        }
        ConverterCmd::GzipDecode { input } => {
            use std::io::Read;
            let input = read_input(input)?;
            let trimmed: String = input.split_whitespace().collect();
            let bytes = STANDARD
                .decode(&trimmed)
                .or_else(|_| URL_SAFE_NO_PAD.decode(&trimmed))
                .context("invalid Base64 input")?;
            let mut decoder = flate2::read::GzDecoder::new(bytes.as_slice());
            let mut out = String::new();
            decoder
                .read_to_string(&mut out)
                .context("input is not valid gzip data or not UTF-8 text")?;
            println!("{out}");
        }
    }
    Ok(())
}
