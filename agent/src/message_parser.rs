use sha2::{Digest, Sha256};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParsedMessage {
    pub sender: String,
    pub content: String,
    pub timestamp: String,
    pub direction: String, // "inbound" | "outbound"
    pub content_hash: String,
}

pub fn compute_hash(sender: &str, content: &str, timestamp: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}|{}|{}", sender, content, timestamp));
    hex::encode(hasher.finalize())
}

pub fn determine_direction(sender: &str, my_name: &str) -> String {
    if my_name.is_empty() {
        return "inbound".to_string();
    }
    // Fuzzy: if sender contains my_name or vice versa
    let sender_lower = sender.to_lowercase();
    let my_lower = my_name.to_lowercase();
    if sender_lower.contains(&my_lower) || my_lower.contains(&sender_lower) {
        "outbound".to_string()
    } else {
        "inbound".to_string()
    }
}

/// Parse the raw AX API snapshot JSON from the Swift helper
pub fn parse_snapshot(json_str: &str, my_name: &str) -> Result<Vec<ParsedMessage>, String> {
    let snapshot: ZaloSnapshot =
        serde_json::from_str(json_str).map_err(|e| format!("JSON parse error: {}", e))?;

    let mut messages = Vec::new();
    for raw in snapshot.messages {
        let direction = determine_direction(&raw.sender, my_name);
        let hash = compute_hash(&raw.sender, &raw.content, &raw.timestamp);
        messages.push(ParsedMessage {
            sender: raw.sender,
            content: raw.content,
            timestamp: raw.timestamp,
            direction,
            content_hash: hash,
        });
    }
    Ok(messages)
}

#[derive(Debug, serde::Deserialize)]
pub struct ZaloSnapshot {
    pub conversation_name: Option<String>,
    pub messages: Vec<RawMessage>,
}

#[derive(Debug, serde::Deserialize)]
pub struct RawMessage {
    pub sender: String,
    pub content: String,
    pub timestamp: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash_is_deterministic() {
        let h1 = compute_hash("Alice", "Hello", "2024-01-01T10:00:00Z");
        let h2 = compute_hash("Alice", "Hello", "2024-01-01T10:00:00Z");
        assert_eq!(h1, h2);
        // Different inputs produce different hashes
        let h3 = compute_hash("Bob", "Hello", "2024-01-01T10:00:00Z");
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_determine_direction_inbound() {
        assert_eq!(determine_direction("Customer A", "Alice"), "inbound");
    }

    #[test]
    fn test_determine_direction_outbound() {
        assert_eq!(determine_direction("Alice", "Alice"), "outbound");
    }

    #[test]
    fn test_determine_direction_empty_my_name() {
        // When my_name is empty, always inbound
        assert_eq!(determine_direction("Alice", ""), "inbound");
    }

    #[test]
    fn test_parse_snapshot_valid_json() {
        let json = r#"{
            "conversation_name": "Test Chat",
            "messages": [
                {"sender": "Customer", "content": "Xin chào", "timestamp": "10:00"},
                {"sender": "Alice", "content": "Chào bạn!", "timestamp": "10:01"}
            ]
        }"#;
        let result = parse_snapshot(json, "Alice");
        assert!(result.is_ok());
        let msgs = result.unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].direction, "inbound");
        assert_eq!(msgs[1].direction, "outbound");
        assert_eq!(msgs[0].sender, "Customer");
        assert!(!msgs[0].content_hash.is_empty());
    }

    #[test]
    fn test_parse_snapshot_empty_messages() {
        let json = r#"{"conversation_name": null, "messages": []}"#;
        let result = parse_snapshot(json, "Alice");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_parse_snapshot_invalid_json() {
        let result = parse_snapshot("not valid json", "Alice");
        assert!(result.is_err());
    }
}
