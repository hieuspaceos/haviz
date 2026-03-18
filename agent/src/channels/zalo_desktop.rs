use crate::channels::traits::{ChannelReader, ChannelSender};
use crate::message_parser::{self, ParsedMessage};
use crate::platform::macos::{accessibility, automation};

pub struct ZaloDesktopChannel {
    reader_path: String,
    my_name: String,
}

impl ZaloDesktopChannel {
    pub fn new(reader_path: String, my_name: String) -> Self {
        Self { reader_path, my_name }
    }
}

impl ChannelReader for ZaloDesktopChannel {
    fn read_messages(&self) -> Result<Vec<ParsedMessage>, String> {
        let json = accessibility::read_zalo_messages(&self.reader_path)?;
        message_parser::parse_snapshot(&json, &self.my_name)
    }
}

impl ChannelSender for ZaloDesktopChannel {
    fn send_message(&self, to: &str, message: &str) -> Result<(), String> {
        automation::send_message_zalo_desktop(to, message)
    }
}
