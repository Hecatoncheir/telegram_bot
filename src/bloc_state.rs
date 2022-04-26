use teloxide_core::types::{File, Message};

#[derive(Clone)]
pub enum BotBlocState {
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
