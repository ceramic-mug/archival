use base64::Engine as _;
use serde_json::{json, Value};
use crate::error::ArchivalError;

const INTERACTIONS_URL: &str = "https://generativelanguage.googleapis.com/v1beta/interactions";
const MODEL: &str = "gemini-3-flash-preview";

pub struct GeminiClient {
    api_key: String,
}

impl GeminiClient {
    pub fn new(api_key: &str) -> Self {
        Self { api_key: api_key.to_string() }
    }

    /// POST to the Interactions API. Returns the model's text output from the
    /// first model_output step.
    pub fn call(
        &self,
        input_parts: Vec<Value>,
        tools: Option<Vec<Value>>,
        response_schema: Option<Value>,
    ) -> Result<String, ArchivalError> {
        let mut body = json!({
            "model": MODEL,
            "input": input_parts,
        });

        if let Some(t) = tools {
            body["tools"] = json!(t);
        }

        if let Some(schema) = response_schema {
            body["response_format"] = json!({
                "type": "text",
                "mime_type": "application/json",
                "schema": schema,
            });
        }

        let body_str = serde_json::to_string(&body)
            .map_err(|e| ArchivalError::Json(format!("serialize request: {e}")))?;

        let response = ureq::post(INTERACTIONS_URL)
            .set("x-goog-api-key", &self.api_key)
            .set("Content-Type", "application/json")
            .send_string(&body_str)
            .map_err(|e| match e {
                ureq::Error::Status(code, resp) => {
                    let body = resp.into_string().unwrap_or_default();
                    // Try to extract error message from {"error":{"message":"..."}}
                    let msg = serde_json::from_str::<Value>(&body)
                        .ok()
                        .and_then(|v| v["error"]["message"].as_str().map(String::from))
                        .unwrap_or_else(|| body.chars().take(200).collect());
                    ArchivalError::Db(format!("Gemini API {code}: {msg}"))
                }
                ureq::Error::Transport(t) => {
                    ArchivalError::Io(format!("HTTP transport error: {t}"))
                }
            })?;

        let body_text = response
            .into_string()
            .map_err(|e| ArchivalError::Json(format!("read response body: {e}")))?;

        let json: Value = serde_json::from_str(&body_text)
            .map_err(|e| ArchivalError::Json(format!("parse response JSON: {e}")))?;

        extract_output_text(&json)
    }
}

/// Read a file and return its contents as a base64-encoded string.
pub fn read_image_b64(path: &str) -> Result<String, ArchivalError> {
    let bytes = std::fs::read(path)
        .map_err(|e| ArchivalError::Io(format!("read image '{path}': {e}")))?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
}

/// Extract the model's text output from an Interactions API response.
/// Finds the first step with type == "model_output" and returns content[0].text.
fn extract_output_text(response: &Value) -> Result<String, ArchivalError> {
    let steps = response["steps"]
        .as_array()
        .ok_or_else(|| ArchivalError::Json("response missing 'steps' array".into()))?;

    for step in steps {
        if step["type"].as_str() == Some("model_output") {
            let content = step["content"]
                .as_array()
                .ok_or_else(|| ArchivalError::Json("model_output step missing 'content'".into()))?;

            for part in content {
                if part["type"].as_str() == Some("text") {
                    if let Some(text) = part["text"].as_str() {
                        return Ok(text.to_string());
                    }
                }
            }
        }
    }

    // Fallback: if response has a top-level "output" string (alternative format)
    if let Some(text) = response["output"].as_str() {
        return Ok(text.to_string());
    }

    Err(ArchivalError::Json(format!(
        "no model_output found in response: {}",
        response.to_string().chars().take(500).collect::<String>()
    )))
}
