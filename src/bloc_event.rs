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
}
