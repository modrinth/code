use eyre::{Context, Result};
use serde_json::{Map, Value, json};

pub(super) fn generate<T: utoipa::ToSchema>() -> Result<Value> {
    let schema = serde_json::to_value(T::schema())
        .wrap_err("serializing input schema")?;
    let mut schemas = Vec::new();
    T::schemas(&mut schemas);
    let components = schemas
        .into_iter()
        .map(|(name, schema)| {
            serde_json::to_value(schema)
                .map(|schema| (name, schema))
                .wrap_err("serializing component schema")
        })
        .collect::<Result<Map<String, Value>>>()?;
    Ok(example(&schema, &components, &mut Vec::new(), 0))
}

fn example(
    schema: &Value,
    components: &Map<String, Value>,
    visited: &mut Vec<String>,
    depth: usize,
) -> Value {
    if depth >= 16 {
        return Value::Null;
    }
    if let Some(value) = [
        schema.get("example"),
        schema
            .get("examples")
            .and_then(Value::as_array)
            .and_then(|v| v.first()),
        schema.get("default"),
        schema.get("const"),
    ]
    .into_iter()
    .flatten()
    .next()
    {
        return value.clone();
    }
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        let Some(name) = reference.strip_prefix("#/components/schemas/") else {
            return Value::Null;
        };
        let name = name.replace("~1", "/").replace("~0", "~");
        if visited.contains(&name) {
            return Value::Null;
        }
        let Some(target) = components.get(&name) else {
            return Value::Null;
        };
        visited.push(name);
        let value = example(target, components, visited, depth + 1);
        visited.pop();
        return value;
    }
    if let Some(values) = schema.get("enum").and_then(Value::as_array) {
        return values
            .iter()
            .find(|v| !v.is_null())
            .cloned()
            .unwrap_or(Value::Null);
    }
    for key in ["oneOf", "anyOf"] {
        if let Some(parts) = schema.get(key).and_then(Value::as_array) {
            return parts
                .iter()
                .find(|part| part["type"] != "null")
                .or_else(|| parts.first())
                .map(|part| example(part, components, visited, depth + 1))
                .unwrap_or(Value::Null);
        }
    }
    if let Some(parts) = schema.get("allOf").and_then(Value::as_array) {
        let mut result = Value::Null;
        for part in parts {
            let value = example(part, components, visited, depth + 1);
            if let (Some(target), Some(source)) =
                (result.as_object_mut(), value.as_object())
            {
                target.extend(source.clone());
            } else if !value.is_null() {
                result = value;
            }
        }
        return result;
    }
    let kind = match schema.get("type") {
        Some(Value::Array(types)) => {
            types.iter().find(|t| *t != "null").and_then(Value::as_str)
        }
        value => value.and_then(Value::as_str),
    };
    if kind == Some("object")
        || schema.get("properties").is_some()
        || schema.get("additionalProperties").is_some()
    {
        let mut object = Map::new();
        if let Some(properties) =
            schema.get("properties").and_then(Value::as_object)
        {
            for (name, property) in properties {
                object.insert(
                    name.clone(),
                    example(property, components, visited, depth + 1),
                );
            }
        }
        if let Some(additional) =
            schema.get("additionalProperties").filter(|v| v.is_object())
        {
            object.entry("key").or_insert_with(|| {
                example(additional, components, visited, depth + 1)
            });
        }
        return Value::Object(object);
    }
    match kind {
        Some("array") => schema
            .get("items")
            .filter(|v| v.is_object())
            .map(|items| {
                json!([example(items, components, visited, depth + 1)])
            })
            .unwrap_or_else(|| json!([])),
        Some("integer" | "number") => schema
            .get("minimum")
            .filter(|v| v.is_number())
            .cloned()
            .unwrap_or(json!(0)),
        Some("boolean") => json!(false),
        Some("string") => {
            json!(match schema.get("format").and_then(Value::as_str) {
                Some("date-time") => "2026-01-01T00:00:00Z",
                Some("date") => "2026-01-01",
                Some("uri" | "url") => "https://example.com/",
                _ => "string",
            })
        }
        _ => Value::Null,
    }
}
