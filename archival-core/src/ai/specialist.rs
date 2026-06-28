use serde_json::{json, Map, Value};
use crate::categories::definitions::find_category;
use crate::error::ArchivalError;
use super::http::{GeminiClient, read_image_b64};

/// Fill category fields from an image using the specialist Gemini agent with Google Search grounding.
/// Returns JSON object string of field values (sql_column names as keys).
pub fn fill_fields(image_path: &str, category: &str, api_key: &str) -> Result<String, ArchivalError> {
    let cat_def = find_category(category)
        .ok_or_else(|| ArchivalError::InvalidInput(format!("unknown category: {category}")))?;

    let b64 = read_image_b64(image_path)?;
    let client = GeminiClient::new(api_key);

    // Build JSON Schema from the category's field definitions
    let mut properties = Map::new();
    for field in cat_def.fields {
        properties.insert(
            field.sql_column.to_string(),
            json!({"type": "string", "description": field.name}),
        );
    }
    let schema = Value::Object({
        let mut m = Map::new();
        m.insert("type".into(), json!("object"));
        m.insert("properties".into(), Value::Object(properties));
        m
    });

    // Field names for the prompt
    let ai_fields: Vec<&str> = cat_def.fields
        .iter()
        .filter(|f| f.ai_fillable)
        .map(|f| f.name)
        .collect();

    let prompt = format!(
        "You are a {}. Examine this {} carefully. \
         Fill in as many of the following fields as you can identify: {}. \
         Use your knowledge and any available context. \
         Set fields you cannot determine to null. \
         Do not include any explanation outside the JSON.",
        cat_def.specialist,
        cat_def.name,
        ai_fields.join(", ")
    );

    let input = vec![
        json!({"type": "image", "data": b64, "mime_type": detect_mime(image_path)}),
        json!({"type": "text", "text": prompt}),
    ];

    // Google Search grounding for factual lookups (ISBN, pub dates, etc.)
    let tools = vec![json!({"type": "google_search"})];

    client.call(input, Some(tools), Some(schema))
}

fn detect_mime(path: &str) -> &'static str {
    let lower = path.to_lowercase();
    if lower.ends_with(".png")  { "image/png"  }
    else if lower.ends_with(".gif")  { "image/gif"  }
    else if lower.ends_with(".webp") { "image/webp" }
    else if lower.ends_with(".heic") { "image/heic" }
    else                             { "image/jpeg" }
}
