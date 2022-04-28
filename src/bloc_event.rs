use std::fmt;
use teloxide_core::types::ReplyMarkup;

#[derive(Clone)]
pub enum BotBlocEvent {
    TextToChatSend {
        chat_id: i64,
        text: String,
    },
    TextToChatSendWithMarkup {
        chat_id: i64,
        text: String,
        markup: ReplyMarkup,
    },

    GetFile {
        file_id: String,
    },

    DownloadFile {
        file_path: String,
        destination_path: String,
    },
}

impl fmt::Display for BotBlocEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            BotBlocEvent::TextToChatSend { chat_id, text } => f.write_str(&format!(
                "TextToChatSend{{chat_id:{}, text:{}}}",
                chat_id, text
            )),
            BotBlocEvent::TextToChatSendWithMarkup { chat_id, text, .. } => f.write_str(&format!(
                "TextToChatSendWithMarkup{{chat_id:{}, text:{}}}",
                chat_id, text
            )),
            BotBlocEvent::GetFile { file_id } => {
                f.write_str(&format!("GetFile{{file_id:{}}}", file_id))
            }
            BotBlocEvent::DownloadFile {
                file_path,
                destination_path,
            } => f.write_str(&format!(
                "GetFile{{file_path:{}, destination_path:{}}}",
                file_path, destination_path
            )),
        }
    }
}
