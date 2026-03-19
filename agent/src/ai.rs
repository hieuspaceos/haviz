use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct GroqRequest {
    model: String,
    messages: Vec<GroqMessage>,
    temperature: f64,
    max_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroqMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct GroqResponse {
    choices: Option<Vec<GroqChoice>>,
    error: Option<GroqError>,
}

#[derive(Debug, Deserialize)]
struct GroqChoice {
    message: GroqMessage,
}

#[derive(Debug, Deserialize)]
struct GroqError {
    message: Option<String>,
}

const SYSTEM_PROMPT: &str = r#"Bạn là trợ lý bán hàng AI cho doanh nghiệp Việt Nam. Vai trò:
- Soạn tin nhắn trả lời khách hàng bằng tiếng Việt tự nhiên
- Phong cách: thân thiện, chuyên nghiệp, phù hợp văn hóa Việt Nam
- Dùng emoji vừa phải, xưng hô phù hợp (anh/chị/em)
- Trả lời ngắn gọn, đúng trọng tâm
- KHÔNG giải thích, chỉ trả lời nội dung tin nhắn

Chỉ trả về nội dung tin nhắn reply, không giải thích gì thêm."#;

/// Chat message from conversation (for building AI context)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub sender: String,
    pub content: String,
    pub direction: String, // "inbound" | "outbound"
}

/// Generate AI draft reply for a conversation
pub async fn generate_draft(
    api_key: &str,
    messages: &[ChatMessage],
    org_context: Option<&str>,
) -> Result<String, String> {
    let client = Client::new();

    // Build system prompt
    let mut system = SYSTEM_PROMPT.to_string();
    if let Some(ctx) = org_context {
        system.push_str(&format!("\n\nBối cảnh doanh nghiệp: {}", ctx));
    }

    // Build message history (last 5 messages for smart context window)
    let recent = if messages.len() > 5 {
        &messages[messages.len() - 5..]
    } else {
        messages
    };

    let mut groq_messages = vec![GroqMessage {
        role: "system".to_string(),
        content: system,
    }];

    for msg in recent {
        let role = if msg.direction == "inbound" {
            "user"
        } else {
            "assistant"
        };
        groq_messages.push(GroqMessage {
            role: role.to_string(),
            content: format!("{}: {}", msg.sender, msg.content),
        });
    }

    let request = GroqRequest {
        model: "meta-llama/llama-4-scout-17b-16e-instruct".to_string(),
        messages: groq_messages,
        temperature: 0.7,
        max_tokens: 300,
    };

    let response = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Groq request failed: {}", e))?;

    let body: GroqResponse = response
        .json()
        .await
        .map_err(|e| format!("Groq parse failed: {}", e))?;

    if let Some(choices) = body.choices {
        if let Some(choice) = choices.first() {
            return Ok(choice.message.content.clone());
        }
    }

    if let Some(err) = body.error {
        return Err(format!("Groq error: {}", err.message.unwrap_or_default()));
    }

    Err("No response from Groq".to_string())
}

/// Build the system prompt string (pure logic, no I/O — testable).
pub fn build_system_prompt(org_context: Option<&str>) -> String {
    let mut system = SYSTEM_PROMPT.to_string();
    if let Some(ctx) = org_context {
        system.push_str(&format!("\n\nBối cảnh doanh nghiệp: {}", ctx));
    }
    system
}

/// Format conversation messages into Groq API format (pure logic, no I/O).
pub fn format_messages_for_groq(messages: &[ChatMessage]) -> Vec<GroqMessage> {
    let recent = if messages.len() > 5 {
        &messages[messages.len() - 5..]
    } else {
        messages
    };
    recent
        .iter()
        .map(|msg| {
            let role = if msg.direction == "inbound" { "user" } else { "assistant" };
            GroqMessage {
                role: role.to_string(),
                content: format!("{}: {}", msg.sender, msg.content),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_system_prompt_without_context() {
        let prompt = build_system_prompt(None);
        assert!(prompt.contains("trợ lý bán hàng"));
        assert!(!prompt.contains("Bối cảnh doanh nghiệp"));
    }

    #[test]
    fn test_build_system_prompt_with_context() {
        let prompt = build_system_prompt(Some("Cửa hàng thời trang ABC"));
        assert!(prompt.contains("Bối cảnh doanh nghiệp: Cửa hàng thời trang ABC"));
    }

    #[test]
    fn test_format_messages_assigns_roles_correctly() {
        let messages = vec![
            ChatMessage { sender: "Customer".to_string(), content: "Hello".to_string(), direction: "inbound".to_string() },
            ChatMessage { sender: "Me".to_string(), content: "Hi there".to_string(), direction: "outbound".to_string() },
        ];
        let groq_msgs = format_messages_for_groq(&messages);
        assert_eq!(groq_msgs.len(), 2);
        assert_eq!(groq_msgs[0].role, "user");
        assert_eq!(groq_msgs[1].role, "assistant");
        assert!(groq_msgs[0].content.contains("Customer: Hello"));
    }

    #[test]
    fn test_format_messages_limits_to_last_5() {
        let messages: Vec<ChatMessage> = (0..8)
            .map(|i| ChatMessage {
                sender: "User".to_string(),
                content: format!("msg {}", i),
                direction: "inbound".to_string(),
            })
            .collect();
        let groq_msgs = format_messages_for_groq(&messages);
        assert_eq!(groq_msgs.len(), 5);
        // Should be the last 5: msg 3..7
        assert!(groq_msgs[0].content.contains("msg 3"));
    }
}
