use std::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
};
use std::sync::{Arc, Mutex};

use tokio::runtime::Runtime;

use teloxide::payloads::SendMessageSetters;
use teloxide_core::adaptors::AutoSend;
use teloxide_core::requests::Requester;
use teloxide_core::types::{ChatId, ReplyMarkup};
use teloxide_core::Bot;

pub trait BLoC<Event, State> {
    fn get_controller(&self) -> Arc<Mutex<Sender<Event>>>;
    fn get_stream(&self) -> Arc<Mutex<Receiver<State>>>;
}

#[derive(Clone)]
pub struct BotBloc {
    bot: Arc<Mutex<AutoSend<Bot>>>,
    event_controller: Arc<Mutex<Sender<BotBlocEvent>>>,
    state_stream: Arc<Mutex<Receiver<BotBlocState>>>,
}

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

#[derive(Clone)]
pub enum BotBlocState {
    TextToChatSendSuccessful { chat_id: i64, text: String },
}

impl BotBloc {
    pub fn new(bot: Arc<Mutex<AutoSend<Bot>>>) -> BotBloc {
        let (event_controller, event_stream) = mpsc::channel::<BotBlocEvent>();
        let (state_controller, state_stream) = mpsc::channel::<BotBlocState>();

        let event_controller = Arc::new(Mutex::new(event_controller));
        let event_stream = event_stream;

        let state_controller = state_controller;
        let state_stream = Arc::new(Mutex::new(state_stream));

        for event in event_stream.iter() {
            let mut runtime = Runtime::new().expect("Unable to create the runtime");

            runtime.spawn(async move {
                tokio::spawn(async {}).await;

                match event {
                    BotBlocEvent::TextToChatSend { chat_id, text } => {
                        let bot = bot.clone().lock().unwrap();

                        bot.send_message(ChatId(chat_id), text.clone()).await;

                        let state = BotBlocState::TextToChatSendSuccessful { chat_id, text };
                        state_controller.send(state).unwrap();
                    }
                    BotBlocEvent::TextToChatSendWithMarkup {
                        chat_id,
                        text,
                        markup,
                    } => {
                        let bot = bot.clone().lock().unwrap();

                        bot.send_message(ChatId(chat_id), text.clone())
                            .reply_markup(markup)
                            .await
                            .unwrap();

                        let state = BotBlocState::TextToChatSendSuccessful { chat_id, text };

                        state_controller.send(state).unwrap();
                    }
                }
            });
        }

        BotBloc {
            bot,
            event_controller,
            state_stream,
        }
    }
}

impl BLoC<BotBlocEvent, BotBlocState> for BotBloc {
    fn get_controller(&self) -> Arc<Mutex<Sender<BotBlocEvent>>> {
        self.event_controller.clone()
    }

    fn get_stream(&self) -> Arc<Mutex<Receiver<BotBlocState>>> {
        self.state_stream.clone()
    }
}

#[cfg(test)]
mod bot_bloc_test {
    use crate::bot::bloc::{BLoC, BotBloc, BotBlocEvent, BotBlocState};
    use std::sync::{Arc, Mutex};
    use teloxide_core::requests::RequesterExt;

    #[test]
    fn can_send_event_and_get_state() {
        let token = "";
        let bot = teloxide::Bot::new(token).auto_send();
        let bot = Arc::new(Mutex::new(bot));

        let bot = BotBloc::new(bot);

        let event = BotBlocEvent::TextToChatSend {
            chat_id: 0,
            text: "Test message".to_string(),
        };

        bot.get_controller().lock().unwrap().send(event);

        if let Some(state) = bot.get_stream().lock().unwrap().iter().next() {
            match state {
                BotBlocState::TextToChatSendSuccessful { chat_id, text } => {
                    println!("{:?}", text);
                }
            }
        }
    }
}
