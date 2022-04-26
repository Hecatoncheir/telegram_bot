#[path = "bloc_test.rs"]
mod bloc_test;

use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use tokio::task;

use teloxide_core::adaptors::AutoSend;
use teloxide_core::requests::Requester;
use teloxide_core::types::{ChatId, Message, Update};
use teloxide_core::Bot;

use teloxide::dispatching::{Dispatcher, DpHandlerDescription, UpdateFilterExt};
use teloxide::error_handlers::LoggingErrorHandler;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{DependencyMap, Handler};
use teloxide::{dptree, respond};

use crate::bloc_event::BotBlocEvent;
use crate::bloc_state::BotBlocState;

pub(crate) type BotUpdateHandler =
    Handler<'static, DependencyMap, Result<(), teloxide_core::RequestError>, DpHandlerDescription>;

#[async_trait]
pub trait BLoC<Event, State> {
    fn get_controller(&self) -> Sender<Event>;
    fn get_stream(&self) -> Receiver<State>;

    async fn run(&self);
    async fn run_with_handler(&self, handler: BotUpdateHandler);

    async fn run_with_handler_and_webhook(
        &self,
        handler: BotUpdateHandler,
        webhook: &str,
        host: &str,
    );
    async fn run_with_webhook(&self, webhook: &str, host: &str);

    async fn run_with_webhook_tls(
        &self,
        webhook: &str,
        host: &str,
        cert_path: &str,
        key_path: &str,
    );
    async fn run_with_handler_and_webhook_tls(
        &self,
        handler: BotUpdateHandler,
        webhook: &str,
        host: &str,
        cert_path: &str,
        key_path: &str,
    );
}

#[derive(Clone)]
pub struct BotBloc {
    bot: AutoSend<Bot>,
    event_controller: Sender<BotBlocEvent>,
    event_stream: Receiver<BotBlocEvent>,
    state_controller: Sender<BotBlocState>,
    state_stream: Receiver<BotBlocState>,
}

impl BotBloc {
    pub fn new(bot: AutoSend<Bot>) -> BotBloc {
        let (event_controller, event_stream) = async_channel::unbounded::<BotBlocEvent>();
        let (state_controller, state_stream) = async_channel::unbounded::<BotBlocState>();

        BotBloc {
            bot,
            event_controller,
            event_stream,
            state_controller,
            state_stream,
        }
    }

    pub fn default_update_handler() -> BotUpdateHandler {
        let message_handler = |message: Message, state_controller: Sender<BotBlocState>| async move {
            let state = BotBlocState::UpdateMessage {
                message: Box::new(message),
            };

            state_controller
                .send(state)
                .await
                .expect("Can't send update state.");

            respond(())
        };

        dptree::entry().branch(Update::filter_message().endpoint(message_handler))
    }

    async fn subscribe_on_events(&self) {
        let event_stream = self.event_stream.clone();
        let state_controller = self.state_controller.clone();
        let bot = self.bot.clone();

        while let Ok(event) = event_stream.recv().await {
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

#[async_trait]
impl BLoC<BotBlocEvent, BotBlocState> for BotBloc {
    fn get_controller(&self) -> Sender<BotBlocEvent> {
        self.event_controller.clone()
    }
    fn get_stream(&self) -> Receiver<BotBlocState> {
        self.state_stream.clone()
    }

    async fn run(&self) {
        let handler = BotBloc::default_update_handler();
        self.run_with_handler(handler).await;
    }

    async fn run_with_handler(&self, handler: BotUpdateHandler) {
        let bot = self.bot.clone();
        let state_controller = self.state_controller.clone();

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

        let _ = tokio::join!(dispatch_handler, self.subscribe_on_events());
    }

    async fn run_with_handler_and_webhook(&self, _: BotUpdateHandler, _: &str, _: &str) {
        todo!()
    }

    async fn run_with_webhook(&self, webhook: &str, host: &str) {
        todo!()
    }

    async fn run_with_webhook_tls(
        &self,
        webhook: &str,
        host: &str,
        cert_path: &str,
        key_path: &str,
    ) {
        todo!()
    }

    async fn run_with_handler_and_webhook_tls(
        &self,
        handler: BotUpdateHandler,
        webhook: &str,
        host: &str,
        cert_path: &str,
        key_path: &str,
    ) {
        todo!()
    }
}
