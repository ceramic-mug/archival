use serde_json::json;
use crate::error::ArchivalError;
use super::http::{GeminiClient, read_image_b64};

const CLASSIFY_PROMPT: &str =
    "Identify the category of this physical object. \
     Return exactly one JSON object with a single key 'category'. \
     Do not include any explanation or extra text.";

/// Classify an image using the Gemini Interactions API.
/// Returns JSON string: {"category": "<category name>"}
/// Where category is one of the 12 Archival categories.
pub fn classify_image(image_path: &str, api_key: &str) -> Result<String, ArchivalError> {
    let b64 = read_image_b64(image_path)?;
    let client = GeminiClient::new(api_key);

    let input = vec![
        json!({"type": "image", "data": b64, "mime_type": detect_mime(image_path)}),
        json!({"type": "text", "text": CLASSIFY_PROMPT}),
    ];

    let schema = json!({
        "type": "object",
        "properties": {
            "category": {
                "type": "string",
                "enum": [
                    "Book", "Music", "Movie", "Game", "PersonalMessage",
                    "Award", "Art", "Photograph", "Trinket", "Jewelry",
                    "Clothing", "Object"
                ]
            }
        },
        "required": ["category"]
    });

    client.call(input, None, Some(schema))
}

/// Infer MIME type from file extension; default to image/jpeg.
fn detect_mime(path: &str) -> &'static str {
    let lower = path.to_lowercase();
    if lower.ends_with(".png")  { "image/png"  }
    else if lower.ends_with(".gif")  { "image/gif"  }
    else if lower.ends_with(".webp") { "image/webp" }
    else if lower.ends_with(".heic") { "image/heic" }
    else                             { "image/jpeg" }
}
