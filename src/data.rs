use crate::util::read_input;
use anyhow::{bail, Context, Result};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum DataCmd {
    /// Convert structured data between JSON, YAML, TOML, XML and CSV
    Convert {
        /// Source format: json | yaml | toml | xml | csv
        #[arg(short, long)]
        from: String,
        /// Target format: json | yaml | toml | xml | csv
        #[arg(short, long)]
        to: String,
        /// CSV field delimiter (or 'tab')
        #[arg(short, long, default_value = ",")]
        delimiter: String,
        /// Input document (reads stdin if omitted)
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
}

pub fn parse_to_json(format: &str, input: &str, delimiter: &str) -> Result<serde_json::Value> {
    match format {
        "json" => serde_json::from_str(input).context("invalid JSON input"),
        "yaml" | "yml" => serde_yaml::from_str(input).context("invalid YAML input"),
        "toml" => toml::from_str(input).context("invalid TOML input"),
        "xml" => xml_to_json(input),
        "csv" => csv_to_json(input, delimiter),
        other => {
            bail!("unsupported source format: {other} (expected json, yaml, toml, xml or csv)")
        }
    }
}

pub fn render_from_json(
    format: &str,
    value: &serde_json::Value,
    delimiter: &str,
) -> Result<String> {
    match format {
        "json" => Ok(serde_json::to_string_pretty(value)?),
        "yaml" | "yml" => Ok(serde_yaml::to_string(value)?.trim_end().to_string()),
        "toml" => toml::to_string_pretty(value)
            .context("value cannot be represented as TOML (e.g. null values or a non-table root)"),
        "xml" => json_to_xml(value),
        "csv" => json_to_csv(value, delimiter),
        other => {
            bail!("unsupported target format: {other} (expected json, yaml, toml, xml or csv)")
        }
    }
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

fn csv_to_json(input: &str, delimiter: &str) -> Result<serde_json::Value> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(csv_delimiter(delimiter)?)
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
    Ok(serde_json::Value::Array(rows))
}

fn json_scalar_to_csv(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn json_to_csv(value: &serde_json::Value, delimiter: &str) -> Result<String> {
    let rows = value
        .as_array()
        .context("CSV output needs a JSON array of objects")?;
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
        .delimiter(csv_delimiter(delimiter)?)
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
    Ok(String::from_utf8(bytes)
        .context("CSV output is not UTF-8")?
        .trim_end()
        .to_string())
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

pub fn run(cmd: DataCmd) -> Result<()> {
    match cmd {
        DataCmd::Convert {
            from,
            to,
            delimiter,
            input,
        } => {
            let input = read_input(input)?;
            let value = parse_to_json(&from.to_lowercase(), &input, &delimiter)?;
            println!(
                "{}",
                render_from_json(&to.to_lowercase(), &value, &delimiter)?
            );
        }
        DataCmd::Lint { format, input } => {
            let input = read_input(input)?;
            let format = format.to_lowercase();
            if format == "csv" {
                bail!("unsupported format: csv (lint covers json, yaml, toml and xml)");
            }
            parse_to_json(&format, &input, ",")?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "valid": true,
                    "format": format,
                }))?
            );
        }
    }
    Ok(())
}
