use std::fmt;
use teloxide_core::types::{File, Message};

#[derive(Clone)]
pub enum BlocState {
    Message {
        message: Box<Message>,
    },
    Command {
        message: Box<Message>,
    },
    TextToChatSendSuccessful {
        chat_id: i64,
        text: String,
    },
    GetFileSuccessful {
        file_id: String,
        file: File,
    },
    GetFileUnsuccessful {
        file_id: String,
    },
    DownloadFileSuccessful {
        file_path: String,
        destination_path: String,
    },
    DownloadFileUnsuccessful {
        file_path: String,
        destination_path: String,
    },
}

impl fmt::Display for BlocState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            BlocState::Message { message } => {
                f.write_str(&format!("Message{{message:{:?}}}", message))
            }
            BlocState::Command { message } => {
                f.write_str(&format!("Command{{message:{:?}}}", message))
            }
            BlocState::TextToChatSendSuccessful { chat_id, text } => f.write_str(&format!(
                "TextToChatSendSuccessful{{chat_id:{}, text:{}}}",
                chat_id, text
            )),
            BlocState::GetFileSuccessful { file_id, file } => f.write_str(&format!(
                "GetFileSuccessful{{file_id:{}, file: {:?}}}",
                file_id, file
            )),
            BlocState::GetFileUnsuccessful { file_id } => {
                f.write_str(&format!("GetFileUnsuccessful{{file_id:{}}}", file_id))
            }
            BlocState::DownloadFileSuccessful {
                file_path,
                destination_path,
            } => f.write_str(&format!(
                "DownloadFileSuccessful{{file_path:{}, destination_path:{}}}",
                file_path, destination_path
            )),
            BlocState::DownloadFileUnsuccessful {
                file_path,
                destination_path,
            } => f.write_str(&format!(
                "DownloadFileUnsuccessful{{file_path:{}, destination_path:{}}}",
                file_path, destination_path
            )),
        }
    }
}
