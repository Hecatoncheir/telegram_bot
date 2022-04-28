#[path = "bloc_test.rs"]
mod bloc_test;

use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use tokio::{fs, task};

use teloxide_core::adaptors::AutoSend;
use teloxide_core::requests::{Request, Requester};
use teloxide_core::types::{ChatId, Message, Update};
use teloxide_core::Bot;

use teloxide::dispatching::{Dispatcher, DpHandlerDescription, UpdateFilterExt};
use teloxide::error_handlers::LoggingErrorHandler;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{DependencyMap, Handler};
use teloxide::{dptree, respond};
use teloxide_core::net::Download;

use crate::bloc_event::BotBlocEvent;
use crate::bloc_state::BotBlocState;

use crate::webhook::webhook_with_tls::webhook_with_tls;
use crate::webhook::webhook_without_tls::webhook_without_tls;

pub(crate) type BotUpdateHandler =
    Handler<'static, DependencyMap, Result<(), teloxide_core::RequestError>, DpHandlerDescription>;

#[async_trait]
pub trait BLoC<Event, State> {
    fn get_controller(&self) -> Sender<Event>;
    fn get_stream(&self) -> Receiver<State>;

    async fn run(&self);
    async fn run_with_handler(&self, handler: BotUpdateHandler);

    async fn run_with_webhook(&self, webhook: String, host: String);
    async fn run_with_handler_and_webhook(
        &self,
        handler: BotUpdateHandler,
        webhook: String,
        host: String,
    );

    async fn run_with_webhook_tls(
        &self,
        webhook: String,
        host: String,
        cert_path: String,
        key_path: String,
    );
    async fn run_with_handler_and_webhook_tls(
        &self,
        handler: BotUpdateHandler,
        webhook: String,
        host: String,
        cert_path: String,
        key_path: String,
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
            let state = BotBlocState::Message {
                message: Box::new(message),
            };

            state_controller
                .send(state)
                .await
                .expect("Can't send update state.");

            respond(())
        };

        let command_filter =
            |message: Message| message.text().is_some() && message.text().unwrap().starts_with('/');

        let command_handler = |message: Message, state_controller: Sender<BotBlocState>| async move {
            let state = BotBlocState::Command {
                message: Box::new(message),
            };

            state_controller
                .send(state)
                .await
                .expect("Can't send update state.");

            respond(())
        };

        dptree::entry().branch(
            Update::filter_message()
                .branch(dptree::filter(command_filter).endpoint(command_handler))
                .branch(dptree::endpoint(message_handler)),
        )
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
                BotBlocEvent::GetFile { file_id } => {
                    match bot.get_file(&file_id).send().await {
                        Ok(file) => {
                            let state = BotBlocState::GetFileSuccessful { file_id, file };
                            let _ = state_controller.send(state).await;
                        }
                        Err(error) => {
                            let log_message =
                                format!("Can't get file details. Error: {:?}.", error);
                            log::warn!("{}", log_message);

                            let state = BotBlocState::GetFileUnsuccessful { file_id };
                            let _ = state_controller.send(state).await;

                            return;
                        }
                    };
                }
                BotBlocEvent::DownloadFile {
                    file_path,
                    destination_path,
                } => {
                    let mut file = match fs::File::create(&file_path).await {
                        Ok(file) => file,
                        Err(error) => {
                            let log_message = format!("Can't create file. Error: {:?}.", error);
                            log::warn!("{}", log_message);

                            let state = BotBlocState::DownloadFileUnsuccessful {
                                file_path,
                                destination_path,
                            };
                            let _ = state_controller.send(state).await;

                            return;
                        }
                    };

                    match bot.download_file(&destination_path, &mut file).await {
                        Ok(_) => {
                            let state = BotBlocState::DownloadFileSuccessful {
                                file_path,
                                destination_path,
                            };
                            let _ = state_controller.send(state).await;
                        }
                        Err(error) => {
                            let log_message = format!("Can't download file. Error: {:?}.", error);
                            log::warn!("{}", log_message);

                            let state = BotBlocState::DownloadFileUnsuccessful {
                                file_path,
                                destination_path,
                            };
                            let _ = state_controller.send(state).await;
                        }
                    };
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

    async fn run_with_webhook(&self, webhook: String, host: String) {
        let handler = BotBloc::default_update_handler();
        self.run_with_handler_and_webhook(handler, webhook, host)
            .await;
    }

    async fn run_with_handler_and_webhook(
        &self,
        handler: BotUpdateHandler,
        webhook: String,
        host: String,
    ) {
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
                .dispatch_with_listener(
                    webhook_without_tls(bot, &host, &webhook).await,
                    LoggingErrorHandler::with_custom_text("An error from the update listener"),
                )
                .await;
        });

        let _ = tokio::join!(dispatch_handler, self.subscribe_on_events());
    }

    async fn run_with_webhook_tls(
        &self,
        webhook: String,
        host: String,
        cert_path: String,
        key_path: String,
    ) {
        let handler = BotBloc::default_update_handler();
        self.run_with_handler_and_webhook_tls(handler, webhook, host, cert_path, key_path)
            .await;
    }

    async fn run_with_handler_and_webhook_tls(
        &self,
        handler: BotUpdateHandler,
        webhook: String,
        host: String,
        cert_path: String,
        key_path: String,
    ) {
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
                .dispatch_with_listener(
                    webhook_with_tls(bot, &host, &webhook, &cert_path, &key_path).await,
                    LoggingErrorHandler::with_custom_text("An error from the update listener"),
                )
                .await;
        });

        let _ = tokio::join!(dispatch_handler, self.subscribe_on_events());
    }
}
