use crate::message_parser::ParsedMessage;

pub trait ChannelReader {
    fn read_messages(&self) -> Result<Vec<ParsedMessage>, String>;
}

pub trait ChannelSender {
    fn send_message(&self, to: &str, message: &str) -> Result<(), String>;
}
