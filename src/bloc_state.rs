use teloxide_core::types::Message;

#[derive(Clone)]
pub enum BotBlocState {
    UpdateMessage { message: Box<Message> },
    TextToChatSendSuccessful { chat_id: i64, text: String },
}
