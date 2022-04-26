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
