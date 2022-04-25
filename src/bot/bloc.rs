use std::sync::Arc;

use async_channel::{Receiver, Sender};
use tokio::task;

use teloxide_core::adaptors::AutoSend;
use teloxide_core::requests::Requester;
use teloxide_core::types::{ChatId, Message, ReplyMarkup, Update};
use teloxide_core::Bot;

use teloxide::dispatching::{Dispatcher, UpdateFilterExt};
use teloxide::error_handlers::LoggingErrorHandler;
use teloxide::payloads::SendMessageSetters;
use teloxide::{dptree, respond};

pub trait BLoC<Event, State> {
    fn get_controller(&self) -> Arc<Sender<Event>>;
    fn get_stream(&self) -> Arc<Receiver<State>>;
}

#[derive(Clone)]
pub struct BotBloc {
    bot: AutoSend<Bot>,
    event_controller: Arc<Sender<BotBlocEvent>>,
    event_stream: Arc<Receiver<BotBlocEvent>>,
    state_controller: Arc<Sender<BotBlocState>>,
    state_stream: Arc<Receiver<BotBlocState>>,
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
    Update { chat_id: i64, text: String },
    TextToChatSendSuccessful { chat_id: i64, text: String },
}

impl BotBloc {
    pub fn new(bot: AutoSend<Bot>) -> BotBloc {
        let (event_controller, event_stream) = async_channel::unbounded::<BotBlocEvent>();
        let (state_controller, state_stream) = async_channel::unbounded::<BotBlocState>();

        let event_controller = Arc::new(event_controller);
        let event_stream = Arc::new(event_stream);

        let state_controller = Arc::new(state_controller);
        let state_stream = Arc::new(state_stream);

        BotBloc {
            bot,
            event_controller,
            event_stream,
            state_controller,
            state_stream,
        }
    }

    pub async fn run(&self) {
        let bot = self.bot.clone();
        let state_controller = self.state_controller.clone();

        let handler = Update::filter_message().branch(
            dptree::filter(|msg: Message| msg.text().is_some()).endpoint(
                |msg: Message, state_controller: Arc<Sender<BotBlocState>>| async move {
                    let state = BotBlocState::Update {
                        chat_id: msg.chat.id.0,
                        text: msg.text().unwrap().to_string(),
                    };
                    state_controller
                        .send(state)
                        .await
                        .expect("Can't send update state.");

                    respond(())
                },
            ),
        );

        let dispatch_handler = task::spawn(async move {
            let ignore_update = |_upd| Box::pin(async {});

            Dispatcher::builder(bot.clone(), handler)
                .dependencies(dptree::deps![bot.clone(), state_controller.clone()])
                .default_handler(ignore_update)
                .error_handler(LoggingErrorHandler::with_custom_text(
                    "An error has occurred in the dispatcher",
                ))
                .build()
                .setup_ctrlc_handler()
                .dispatch()
                .await;
        });

        tokio::join!(self.subscribe_on_events(), dispatch_handler);
    }

    async fn run_with_handler(&self) {
        // TODO
    }

    async fn subscribe_on_events(&self) {
        let event_stream = self.event_stream.clone();
        let state_controller = self.state_controller.clone();
        let bot = self.bot.clone();

        for event in event_stream.recv().await {
            match event {
                BotBlocEvent::TextToChatSend { chat_id, text } => {
                    bot.send_message(ChatId(chat_id), text.clone()).await;

                    let state = BotBlocState::TextToChatSendSuccessful { chat_id, text };
                    state_controller.send(state).await;
                }
                BotBlocEvent::TextToChatSendWithMarkup {
                    chat_id,
                    text,
                    markup,
                } => {
                    bot.send_message(ChatId(chat_id), text.clone())
                        .reply_markup(markup)
                        .await
                        .unwrap();

                    let state = BotBlocState::TextToChatSendSuccessful { chat_id, text };

                    state_controller.send(state).await;
                }
            }
        }
    }
}

impl BLoC<BotBlocEvent, BotBlocState> for BotBloc {
    fn get_controller(&self) -> Arc<Sender<BotBlocEvent>> {
        self.event_controller.clone()
    }
    fn get_stream(&self) -> Arc<Receiver<BotBlocState>> {
        self.state_stream.clone()
    }
}

#[cfg(test)]
mod bot_bloc_test {
    use crate::bot::bloc::{BLoC, BotBloc, BotBlocEvent, BotBlocState};
    use std::borrow::BorrowMut;
    use std::sync::{Arc, Mutex};
    use teloxide_core::requests::RequesterExt;
    use tokio::sync::mpsc;
    use tokio::task;

    #[tokio::test]
    async fn can_send_event_and_get_state() {
        let token = "";
        let bot = teloxide::Bot::new(token).auto_send();

        let bloc = BotBloc::new(bot);
        let bloc_reference_counter = Arc::new(bloc);

        let bloc_for_spawn = bloc_reference_counter.clone();
        tokio::spawn(async move {
            while let Ok(state) = bloc_for_spawn.get_stream().recv().await {
                match state {
                    BotBlocState::Update { chat_id, text } => {
                        let event = BotBlocEvent::TextToChatSend {
                            chat_id,
                            text: text.to_string(),
                        };

                        bloc_for_spawn.get_controller().send(event).await;
                    }
                    BotBlocState::TextToChatSendSuccessful { .. } => {}
                }
            }
        });

        let bloc_for_run = bloc_reference_counter.clone();
        bloc_for_run.run().await;
    }
}
