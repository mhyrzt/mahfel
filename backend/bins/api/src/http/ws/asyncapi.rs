use anyhow::{Context, Result};
use asyncapi_rust::AsyncApi;
use serde_json::{Value, json};

use super::envelope::{ClientMessage, PROTOCOL_VERSION, ServerMessage};

#[allow(clippy::duplicated_attributes)]
#[derive(AsyncApi)]
#[asyncapi(
    title = "Mahfel WebSocket API",
    version = "1.0.0",
    description = "JSON signaling over /v1/ws. The API receives client messages and sends server messages. Authenticate with a bearer token on the upgrade request or an auth message before other messages."
)]
#[asyncapi_channel(name = "signaling", address = "/v1/ws")]
#[asyncapi_operation(
    name = "receiveClientMessage",
    action = "receive",
    channel = "signaling"
)]
#[asyncapi_operation(name = "sendServerMessage", action = "send", channel = "signaling")]
#[asyncapi_messages(ClientMessage, ServerMessage)]
struct WebSocketApi;

pub(crate) fn document() -> Result<Value> {
    let mut document = serde_json::to_value(WebSocketApi::asyncapi_spec())
        .context("serialize AsyncAPI document")?;
    let client = ClientMessage::asyncapi_message_names();
    let server = ServerMessage::asyncapi_message_names();
    let messages = document
        .pointer_mut("/components/messages")
        .and_then(Value::as_object_mut)
        .context("AsyncAPI message components missing")?;
    set_variant_payloads(messages, &client, false)?;
    set_variant_payloads(messages, &server, true)?;

    let channel = document
        .pointer_mut("/channels/signaling")
        .and_then(Value::as_object_mut)
        .context("AsyncAPI signaling channel missing")?;
    channel.insert(
        "messages".to_owned(),
        Value::Object(
            client
                .iter()
                .chain(server.iter())
                .map(|name| ((*name).to_owned(), message_reference(name)))
                .collect(),
        ),
    );

    for (operation, names) in [
        ("receiveClientMessage", client),
        ("sendServerMessage", server),
    ] {
        let operation = document
            .pointer_mut(&format!("/operations/{operation}"))
            .and_then(Value::as_object_mut)
            .context("AsyncAPI operation missing")?;
        operation.insert(
            "messages".to_owned(),
            Value::Array(names.iter().map(|name| message_reference(name)).collect()),
        );
    }

    Ok(document)
}

fn message_reference(name: &str) -> Value {
    json!({"$ref": format!("#/components/messages/{name}")})
}

fn set_variant_payloads(
    messages: &mut serde_json::Map<String, Value>,
    names: &[&str],
    require_version: bool,
) -> Result<()> {
    let variants = messages
        .get(*names.first().context("message enum is empty")?)
        .and_then(|message| message.pointer("/payload/oneOf"))
        .and_then(Value::as_array)
        .context("tagged enum schema missing variants")?
        .clone();
    anyhow::ensure!(
        variants.len() == names.len(),
        "message variant count mismatch"
    );

    for (name, mut payload) in names.iter().zip(variants) {
        let properties = payload
            .get_mut("properties")
            .and_then(Value::as_object_mut)
            .with_context(|| format!("variant properties missing for {name}"))?;
        properties.insert(
            "v".to_owned(),
            json!({"type": "integer", "const": PROTOCOL_VERSION, "default": PROTOCOL_VERSION}),
        );
        properties.insert("id".to_owned(), json!({"type": "string"}));
        if require_version {
            payload
                .get_mut("required")
                .and_then(Value::as_array_mut)
                .with_context(|| format!("required fields missing for {name}"))?
                .push(json!("v"));
        }
        let message = messages
            .get_mut(*name)
            .with_context(|| format!("message missing for {name}"))?;
        message["payload"] = payload;
    }
    Ok(())
}
