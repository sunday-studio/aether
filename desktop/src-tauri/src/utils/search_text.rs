use crate::error::{AppError, Result};
use serde_json::Value;

pub fn normalize_search_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn extract_text_from_lexical_document(document: &str) -> Result<String> {
    if document.trim().is_empty() {
        return Ok(String::new());
    }

    let json: Value = serde_json::from_str(document)
        .map_err(|e| AppError::BadRequest(format!("Invalid Lexical JSON: {}", e)))?;
    let mut parts = Vec::new();
    collect_lexical_text(&json, &mut parts);
    Ok(normalize_search_text(&parts.join(" ")))
}

pub fn first_search_line(text: &str) -> String {
    normalize_search_text(text)
        .split('.')
        .next()
        .unwrap_or_default()
        .trim()
        .to_string()
}

pub fn truncate_preview(text: &str, max_chars: usize) -> String {
    let normalized = normalize_search_text(text);
    if normalized.chars().count() <= max_chars {
        return normalized;
    }

    let mut preview: String = normalized.chars().take(max_chars).collect();
    preview = preview.trim_end().to_string();
    preview.push_str("...");
    preview
}

fn collect_lexical_text(node: &Value, parts: &mut Vec<String>) {
    match node {
        Value::Object(map) => {
            if map.get("type").and_then(|v| v.as_str()) == Some("text") {
                if let Some(text) = map.get("text").and_then(|v| v.as_str()) {
                    if !text.trim().is_empty() {
                        parts.push(text.to_string());
                    }
                }
            }

            if let Some(children) = map.get("children").and_then(|v| v.as_array()) {
                for child in children {
                    collect_lexical_text(child, parts);
                }
            } else {
                for value in map.values() {
                    collect_lexical_text(value, parts);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_lexical_text(item, parts);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_visible_text_from_lexical_document() {
        let document = r#"{
            "root": {
                "children": [
                    {
                        "type": "paragraph",
                        "children": [
                            { "type": "text", "text": "Launch" },
                            { "type": "text", "text": " pricing" }
                        ]
                    },
                    {
                        "type": "paragraph",
                        "children": [
                            { "type": "text", "text": "Feeling blocked today." }
                        ]
                    }
                ]
            }
        }"#;

        let text = extract_text_from_lexical_document(document).unwrap();
        assert_eq!(text, "Launch pricing Feeling blocked today.");
    }

    #[test]
    fn normalizes_search_text_whitespace() {
        assert_eq!(normalize_search_text("  one\n two\tthree  "), "one two three");
    }
}
