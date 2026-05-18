use uuid::Uuid;

/// Generates a new ID with the given prefix
/// Format: `{prefix}_{uuid}`
pub fn generate_id(prefix: &str) -> String {
    format!("{}_{}", prefix, Uuid::new_v4())
}

/// Validates an ID format
/// Checks if the ID starts with the expected prefix and contains a valid UUID
pub fn is_valid_id(value: &str, prefix: &str) -> bool {
    let value = value.trim();
    let expected_prefix = format!("{}_", prefix);

    if !value.starts_with(&expected_prefix) {
        return false;
    }

    let uuid_part = value.strip_prefix(&expected_prefix).unwrap_or("");
    Uuid::parse_str(uuid_part).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let id = generate_id("entry");
        assert!(id.starts_with("entry_"));
        assert!(is_valid_id(&id, "entry"));
    }

    #[test]
    fn test_is_valid_id() {
        assert!(is_valid_id(
            "entry_123e4567-e89b-12d3-a456-426614174000",
            "entry"
        ));
        assert!(!is_valid_id(
            "entry_123e4567-e89b-12d3-a456-426614174000",
            "task"
        ));
        assert!(!is_valid_id("invalid", "entry"));
    }
}
