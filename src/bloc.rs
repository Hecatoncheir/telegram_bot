#[path = "bloc_test.rs"]
mod bloc_test;

use std::sync::Arc;

use async_channel::{Receiver, Sender};
use tokio::task;

use teloxide_core::adaptors::AutoSend;
use teloxide_core::requests::Requester;
use teloxide_core::types::{ChatId, Message, Update};
use teloxide_core::Bot;

use teloxide::dispatching::{Dispatcher, UpdateFilterExt};
use teloxide::error_handlers::LoggingErrorHandler;
use teloxide::payloads::SendMessageSetters;
use teloxide::{dptree, respond};

use crate::bloc_event::BotBlocEvent;
use crate::bloc_state::BotBlocState;

pub trait BLoC<Event, State> {
    fn get_controller(&self) -> Arc<Sender<Event>>;
    fn get_stream(&self) -> Arc<Receiver<State>>;

    fn run(&self);
    fn run_with_handler(&self);

    fn run_with_webhook(&self);
    fn run_with_webhook_tls(&self);
}

#[derive(Clone)]
pub struct BotBloc {
    bot: AutoSend<Bot>,
    event_controller: Arc<Sender<BotBlocEvent>>,
    event_stream: Arc<Receiver<BotBlocEvent>>,
    state_controller: Arc<Sender<BotBlocState>>,
    state_stream: Arc<Receiver<BotBlocState>>,
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
                    let _ = bot.send_message(ChatId(chat_id), text.clone()).await;

                    let state = BotBlocState::TextToChatSendSuccessful { chat_id, text };
                    let _ = state_controller.send(state).await;
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

                    let _ = state_controller.send(state).await;
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
