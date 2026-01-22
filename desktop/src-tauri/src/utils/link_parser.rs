use crate::error::{AppError, Result};
use serde_json::Value;

/// Represents a link extracted from content
#[derive(Debug, Clone)]
pub struct ExtractedLink {
    pub target_type: String,
    pub target_id: String,
    pub link_text: Option<String>,
}

/// Extract links from Lexical JSON content
/// This function looks for ResourceLinkNode instances in the Lexical JSON structure
pub fn extract_links_from_lexical_content(content: &str) -> Result<Vec<ExtractedLink>> {
    if content.is_empty() {
        return Ok(Vec::new());
    }

    let json: Value = serde_json::from_str(content)
        .map_err(|e| AppError::BadRequest(format!("Invalid JSON content: {}", e)))?;

    let mut links = Vec::new();
    extract_links_from_node(&json, &mut links)?;
    Ok(links)
}

/// Recursively extract links from a Lexical node
fn extract_links_from_node(node: &Value, links: &mut Vec<ExtractedLink>) -> Result<()> {
    match node {
        Value::Object(map) => {
            // Check if this is a ResourceLinkNode
            if let Some(node_type) = map.get("type").and_then(|v| v.as_str()) {
                if node_type == "resourceLink" {
                    if let Some(target_type) = map.get("targetType").and_then(|v| v.as_str()) {
                        if let Some(target_id) = map.get("targetId").and_then(|v| v.as_str()) {
                            let link_text = map
                                .get("linkText")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());

                            links.push(ExtractedLink {
                                target_type: target_type.to_string(),
                                target_id: target_id.to_string(),
                                link_text,
                            });
                        }
                    }
                }
            }

            // Recursively process children
            if let Some(children) = map.get("children") {
                if let Value::Array(children_array) = children {
                    for child in children_array {
                        extract_links_from_node(child, links)?;
                    }
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                extract_links_from_node(item, links)?;
            }
        }
        _ => {}
    }

    Ok(())
}

/// Extract plain text from Lexical JSON content for searching [[links]] patterns
/// This is a fallback method if ResourceLinkNode is not used
pub fn extract_text_from_lexical_content(content: &str) -> Result<String> {
    if content.is_empty() {
        return Ok(String::new());
    }

    let json: Value = serde_json::from_str(content)
        .map_err(|e| AppError::BadRequest(format!("Invalid JSON content: {}", e)))?;

    let mut text = String::new();
    extract_text_from_node(&json, &mut text)?;
    Ok(text)
}

/// Recursively extract text from a Lexical node
fn extract_text_from_node(node: &Value, text: &mut String) -> Result<()> {
    match node {
        Value::Object(map) => {
            // Extract text from text nodes
            if let Some(node_type) = map.get("type").and_then(|v| v.as_str()) {
                if node_type == "text" {
                    if let Some(text_content) = map.get("text").and_then(|v| v.as_str()) {
                        text.push_str(text_content);
                    }
                }
            }

            // Recursively process children
            if let Some(children) = map.get("children") {
                if let Value::Array(children_array) = children {
                    for child in children_array {
                        extract_text_from_node(child, text)?;
                    }
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                extract_text_from_node(item, text)?;
            }
        }
        _ => {}
    }

    Ok(())
}

/// Find [[link]] patterns in plain text
/// This is used as a fallback to extract links from text content
pub fn find_link_patterns_in_text(text: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut chars = text.chars().peekable();
    let mut buffer = String::new();
    let mut in_link = false;
    let mut bracket_count = 0;

    while let Some(ch) = chars.next() {
        if ch == '[' {
            bracket_count += 1;
            if bracket_count == 2 && !in_link {
                in_link = true;
                buffer.clear();
                continue;
            }
        } else if ch == ']' {
            if in_link && bracket_count >= 2 {
                bracket_count -= 1;
                if bracket_count == 0 {
                    // Found a complete [[link]]
                    if !buffer.is_empty() {
                        links.push(buffer.trim().to_string());
                    }
                    buffer.clear();
                    in_link = false;
                    continue;
                }
            } else {
                bracket_count = bracket_count.saturating_sub(1);
            }
        }

        if in_link {
            buffer.push(ch);
        }
    }

    links
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_link_patterns_in_text() {
        let text = "This is a [[test link]] and another [[link here]]";
        let links = find_link_patterns_in_text(text);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0], "test link");
        assert_eq!(links[1], "link here");
    }

    #[test]
    fn test_extract_links_from_lexical_content() {
        let content = r#"{
            "root": {
                "children": [
                    {
                        "type": "resourceLink",
                        "targetType": "entry",
                        "targetId": "entry-123",
                        "linkText": "My Entry"
                    }
                ]
            }
        }"#;
        let links = extract_links_from_lexical_content(content).unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target_type, "entry");
        assert_eq!(links[0].target_id, "entry-123");
        assert_eq!(links[0].link_text, Some("My Entry".to_string()));
    }
}
