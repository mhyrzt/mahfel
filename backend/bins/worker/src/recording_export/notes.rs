use anyhow::{Result, ensure};
use serde_json::{Value, json};
use yrs::{Doc, Map, Out, ReadTxn, Text, Transact, Xml, XmlTextRef, types::text::YChange};

fn value(out: Out) -> Value {
    match out {
        Out::Any(yrs::Any::Number(number)) if number.fract() == 0.0 => json!(number as i64),
        Out::Any(value) => serde_json::to_value(value).unwrap_or(Value::Null),
        _ => Value::Null,
    }
}

pub(super) fn blocks(doc: &Doc, assets: &Value) -> Result<Vec<Value>> {
    let text = doc.get_or_insert_text("root");
    let root: &yrs::XmlTextRef = text.as_ref();
    let txn = doc.transact();
    let mut blocks = Vec::new();
    walk(root, &txn, assets, &mut blocks, 0, None, None)?;
    ensure!(blocks.len() <= 1000, "notes block limit");
    Ok(blocks)
}

fn attr<T: ReadTxn>(node: &XmlTextRef, txn: &T, key: &str) -> Value {
    node.get_attribute(txn, key)
        .map(value)
        .unwrap_or(Value::Null)
}

fn walk<T: ReadTxn>(
    node: &XmlTextRef,
    txn: &T,
    assets: &Value,
    blocks: &mut Vec<Value>,
    depth: usize,
    list: Option<&str>,
    list_number: Option<u64>,
) -> Result<()> {
    ensure!(depth <= 32 && blocks.len() <= 1000, "notes nesting limit");
    let kind = attr(node, txn, "__type");
    let kind = kind.as_str().unwrap_or("root");
    let own_list = attr(node, txn, "__listType");
    let list = own_list.as_str().or(list);
    let mut number = attr(node, txn, "__start").as_u64().unwrap_or(1);
    let mut runs = Vec::new();
    let mut format = 0i64;
    for chunk in node.diff(txn, YChange::identity) {
        match chunk.insert {
            Out::Any(yrs::Any::String(text)) => runs.push(json!({"text":text.as_ref(),"bold":format&1!=0,"italic":format&2!=0,"strike":format&4!=0,"underline":format&8!=0,"code":format&16!=0})),
            Out::YMap(map) => {
                let node_kind = map.get(txn,"__type").map(value).unwrap_or(Value::Null);
                if node_kind == "text" { format = map.get(txn,"__format").map(value).and_then(|value| value.as_i64()).unwrap_or_default(); }
                else if node_kind == "linebreak" { runs.push(json!({"text":"\n"})); }
                else if node_kind == "collaboration-image" {
                    let id = map.get(txn,"__fileId").map(value).unwrap_or(Value::Null);
                    if let Some(reference) = id.as_str().and_then(|id| assets.get(id)).filter(|asset| !asset["render"].is_null()) {
                        blocks.push(json!({"type":"image","asset":reference["render"],"alt":map.get(txn,"__altText").map(value).and_then(|value|value.as_str().map(str::to_owned)).unwrap_or_default()}));
                    } else { runs.push(json!({"text":"Image unavailable"})); }
                } else { ensure!(node_kind == "link" || node_kind == "autolink", "unsupported recorded notes node"); }
            }
            Out::YXmlText(child) => {
                let mut children = Vec::new();
                let item = kind == "list" && attr(&child, txn, "__type") == "listitem";
                walk(&child, txn, assets, &mut children, depth + 1, list, item.then_some(number))?;
                if item { number += 1; }
                if matches!(kind,"paragraph"|"heading"|"quote"|"listitem") && matches!(attr(&child,txn,"__type").as_str(),Some("link"|"autolink")) {
                    for block in children { if let Some(items) = block["runs"].as_array() { runs.extend(items.iter().cloned()); } }
                } else { blocks.extend(children); }
            }
            _ => anyhow::bail!("unsupported recorded notes content"),
        }
    }
    if !runs.is_empty() || matches!(kind, "paragraph" | "heading" | "quote" | "listitem") {
        let block_kind = match kind {
            "heading" => "heading",
            "quote" => "quote",
            "listitem" => "list_item",
            _ => "paragraph",
        };
        let tag = attr(node, txn, "__tag");
        let level = tag
            .as_str()
            .and_then(|tag| tag.strip_prefix('h'))
            .and_then(|level| level.parse::<u8>().ok())
            .unwrap_or(1);
        let marker = match list {
            Some("number") => format!("{}.", list_number.unwrap_or(1)),
            Some("check") => {
                if attr(node, txn, "__checked") == true {
                    "☑".into()
                } else {
                    "☐".into()
                }
            }
            _ => "•".into(),
        };
        let format = attr(node, txn, "__format");
        let align = match format.as_i64() {
            Some(2) => "center",
            Some(3) => "right",
            _ => "left",
        };
        blocks.push(
            json!({"type":block_kind,"runs":runs,"level":level,"marker":marker,"align":align,"direction":if attr(node,txn,"__dir")=="rtl" {"rtl"}else{"ltr"}}),
        );
    }
    Ok(())
}
