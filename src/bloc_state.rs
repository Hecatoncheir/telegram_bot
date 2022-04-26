#[derive(Clone)]
pub enum BotBlocState {
    Update { chat_id: i64, text: String },
    TextToChatSendSuccessful { chat_id: i64, text: String },
}
