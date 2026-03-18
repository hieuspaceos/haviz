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
