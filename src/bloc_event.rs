use std::fmt;
use teloxide_core::types::{InputMedia, ReplyMarkup};

#[derive(Clone)]
pub enum BlocEvent {
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
    MediaToChatSend {
        chat_id: i64,
        media: Vec<InputMedia>,
    },
}

impl fmt::Display for BlocEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            BlocEvent::TextToChatSend { chat_id, text } => f.write_str(&format!(
                "TextToChatSend{{chat_id:{}, text:{}}}",
                chat_id, text
            )),
            BlocEvent::TextToChatSendWithMarkup { chat_id, text, .. } => f.write_str(&format!(
                "TextToChatSendWithMarkup{{chat_id:{}, text:{}}}",
                chat_id, text
            )),
            BlocEvent::GetFile { file_id } => {
                f.write_str(&format!("GetFile{{file_id:{}}}", file_id))
            }
            BlocEvent::DownloadFile {
                file_path,
                destination_path,
            } => f.write_str(&format!(
                "GetFile{{file_path:{}, destination_path:{}}}",
                file_path, destination_path
            )),
            BlocEvent::MediaToChatSend { chat_id, .. } => {
                f.write_str(&format!("MediaToChatSend{{chat_id:{}}}", chat_id))
            }
        }
    }
}
