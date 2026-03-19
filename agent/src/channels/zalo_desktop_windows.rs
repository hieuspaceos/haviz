/// Windows implementation of ChannelReader + ChannelSender for Zalo Desktop.
///
/// Delegates to platform::windows::uiautomation for reading and
/// platform::windows::input for sending — keeping channel logic thin.
use crate::channels::traits::{ChannelReader, ChannelSender};
use crate::message_parser::{compute_hash, determine_direction, ParsedMessage};
use crate::platform::windows::{input, uiautomation};

pub struct WindowsZaloDesktop {
    pub my_name: String,
}

impl WindowsZaloDesktop {
    pub fn new(my_name: String) -> Self {
        Self { my_name }
    }
}

impl ChannelReader for WindowsZaloDesktop {
    fn read_messages(&self) -> Result<Vec<ParsedMessage>, String> {
        let raw = uiautomation::read_zalo_messages()?;

        let messages = raw
            .into_iter()
            .map(|m| {
                let direction = determine_direction(&m.sender, &self.my_name);
                let hash = compute_hash(&m.sender, &m.content, &m.timestamp);
                ParsedMessage {
                    sender: m.sender,
                    content: m.content,
                    timestamp: m.timestamp,
                    direction,
                    content_hash: hash,
                }
            })
            .collect();

        Ok(messages)
    }
}

impl ChannelSender for WindowsZaloDesktop {
    fn send_message(&self, to: &str, message: &str) -> Result<(), String> {
        input::send_message_to_zalo(to, message)
    }
}
