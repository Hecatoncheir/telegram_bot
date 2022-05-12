use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use tokio::{fs, signal, task};

use teloxide_core::adaptors::{AutoSend, DefaultParseMode};
use teloxide_core::requests::{Request, Requester};
use teloxide_core::types::{ChatId, Message, Update};
use teloxide_core::Bot;

use teloxide::dispatching::{Dispatcher, UpdateFilterExt};
use teloxide::error_handlers::LoggingErrorHandler;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::DependencyMap;
use teloxide::{dptree, respond};
use teloxide_core::net::Download;

use crate::bloc_event::BlocEvent;
use crate::bloc_state::BlocState;

use crate::bloc::{BLoC, BotUpdateHandler};

use crate::webhook::webhook_with_tls_for_bot_with_default_parse_mode::webhook_with_tls_for_bot_with_default_parse_mode;
use crate::webhook::webhook_without_tls_for_bot_with_default_parse_mode::webhook_without_tls_for_bot_with_default_parse_mode;

#[derive(Clone)]
pub struct Bloc {
    bot: AutoSend<DefaultParseMode<Bot>>,
    event_controller: Sender<BlocEvent>,
    event_stream: Receiver<BlocEvent>,
    state_controller: Sender<BlocState>,
    state_stream: Receiver<BlocState>,
}

impl Bloc {
    pub fn new(bot: AutoSend<DefaultParseMode<Bot>>) -> Bloc {
        let (event_controller, event_stream) = async_channel::unbounded::<BlocEvent>();
        let (state_controller, state_stream) = async_channel::unbounded::<BlocState>();

        Bloc {
            bot,
            event_controller,
            event_stream,
            state_controller,
            state_stream,
        }
    }

    pub fn default_update_handler() -> BotUpdateHandler {
        let message_handler = |message: Message, state_controller: Sender<BlocState>| async move {
            let state = BlocState::Message {
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

        let command_handler = |message: Message, state_controller: Sender<BlocState>| async move {
            let state = BlocState::Command {
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
                BlocEvent::TextToChatSend { chat_id, text } => {
                    let _ = bot.send_message(ChatId(chat_id), text.clone()).await;

                    let state = BlocState::TextToChatSendSuccessful { chat_id, text };
                    let _ = state_controller.send(state).await;
                }
                BlocEvent::TextToChatSendWithMarkup {
                    chat_id,
                    text,
                    markup,
                } => {
                    bot.send_message(ChatId(chat_id), text.clone())
                        .reply_markup(markup)
                        .await
                        .unwrap();

                    let state = BlocState::TextToChatSendSuccessful { chat_id, text };
                    let _ = state_controller.send(state).await;
                }
                BlocEvent::GetFile { file_id } => {
                    match bot.get_file(&file_id).send().await {
                        Ok(file) => {
                            let state = BlocState::GetFileSuccessful { file_id, file };
                            let _ = state_controller.send(state).await;
                        }
                        Err(error) => {
                            let log_message =
                                format!("Can't get file details. Error: {:?}.", error);
                            log::warn!("{}", log_message);

                            let state = BlocState::GetFileUnsuccessful { file_id };
                            let _ = state_controller.send(state).await;

                            return;
                        }
                    };
                }
                BlocEvent::DownloadFile {
                    file_path,
                    destination_path,
                } => {
                    let mut file = match fs::File::create(&destination_path).await {
                        Ok(file) => file,
                        Err(error) => {
                            let log_message = format!("Can't create file. Error: {:?}.", error);
                            log::warn!("{}", log_message);

                            let state = BlocState::DownloadFileUnsuccessful {
                                file_path,
                                destination_path,
                            };
                            let _ = state_controller.send(state).await;

                            return;
                        }
                    };

                    match bot.download_file(&file_path, &mut file).await {
                        Ok(_) => {
                            let state = BlocState::DownloadFileSuccessful {
                                file_path,
                                destination_path,
                            };
                            let _ = state_controller.send(state).await;
                        }
                        Err(error) => {
                            let log_message = format!("Can't download file. Error: {:?}.", error);
                            log::warn!("{}", log_message);

                            let state = BlocState::DownloadFileUnsuccessful {
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
impl BLoC<BlocEvent, BlocState> for Bloc {
    fn get_controller(&self) -> Sender<BlocEvent> {
        self.event_controller.clone()
    }
    fn get_stream(&self) -> Receiver<BlocState> {
        self.state_stream.clone()
    }

    async fn run(&self) {
        let handler = Bloc::default_update_handler();
        self.run_with_handler(handler).await;
    }

    async fn run_with_handler(&self, handler: BotUpdateHandler) {
        let that = self.clone();

        tokio::spawn(async move {
            let bot = that.bot.clone();
            let state_controller = that.state_controller.clone();

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

            let _ = tokio::join!(dispatch_handler, that.subscribe_on_events());
        });

        match signal::ctrl_c().await {
            Ok(()) => {}
            Err(err) => {
                eprintln!("Unable to listen for shutdown signal: {}", err);
            }
        }
    }

    async fn run_with_webhook(&self, webhook: String, host: String) {
        let handler = Bloc::default_update_handler();
        let dependencies = dptree::deps![self.bot.clone(), self.state_controller.clone()];

        self.run_with_handler_and_webhook(handler, dependencies, webhook, host)
            .await;
    }

    /// dependencies(dptree::deps![])
    async fn run_with_handler_and_webhook(
        &self,
        handler: BotUpdateHandler,
        mut dependencies: DependencyMap,
        webhook: String,
        host: String,
    ) {
        let that = self.clone();

        tokio::spawn(async move {
            let bot = that.bot.clone();
            let state_controller = that.state_controller.clone();

            dependencies.insert(bot.clone());
            dependencies.insert(state_controller.clone());

            let dispatch_handler = task::spawn(async move {
                let ignore_update = |_upd| Box::pin(async {});

                Dispatcher::builder(bot.clone(), handler)
                    .dependencies(dependencies)
                    .default_handler(ignore_update)
                    .error_handler(LoggingErrorHandler::with_custom_text(
                        "An error has occurred in the dispatcher",
                    ))
                    .build()
                    .setup_ctrlc_handler()
                    .dispatch_with_listener(
                        webhook_without_tls_for_bot_with_default_parse_mode(bot, &host, &webhook)
                            .await,
                        LoggingErrorHandler::with_custom_text("An error from the update listener"),
                    )
                    .await;
            });

            let _ = tokio::join!(dispatch_handler, that.subscribe_on_events());
        });

        match signal::ctrl_c().await {
            Ok(()) => {}
            Err(err) => {
                eprintln!("Unable to listen for shutdown signal: {}", err);
            }
        }
    }

    async fn run_with_webhook_tls(
        &self,
        webhook: String,
        host: String,
        cert_path: String,
        key_path: String,
    ) {
        let handler = Bloc::default_update_handler();
        let dependencies = dptree::deps![self.bot.clone(), self.state_controller.clone()];

        self.run_with_handler_and_webhook_tls(
            handler,
            dependencies,
            webhook,
            host,
            cert_path,
            key_path,
        )
        .await;
    }

    async fn run_with_handler_and_webhook_tls(
        &self,
        handler: BotUpdateHandler,
        mut dependencies: DependencyMap,
        webhook: String,
        host: String,
        cert_path: String,
        key_path: String,
    ) {
        let bot = self.bot.clone();
        let state_controller = self.state_controller.clone();

        dependencies.insert(bot.clone());
        dependencies.insert(state_controller.clone());

        let dispatch_handler = task::spawn(async move {
            let ignore_update = |_upd| Box::pin(async {});

            Dispatcher::builder(bot.clone(), handler)
                .dependencies(dependencies)
                .default_handler(ignore_update)
                .error_handler(LoggingErrorHandler::with_custom_text(
                    "An error has occurred in the dispatcher",
                ))
                .build()
                .setup_ctrlc_handler()
                .dispatch_with_listener(
                    webhook_with_tls_for_bot_with_default_parse_mode(
                        bot, &host, &webhook, &cert_path, &key_path,
                    )
                    .await,
                    LoggingErrorHandler::with_custom_text("An error from the update listener"),
                )
                .await;
        });

        let _ = tokio::join!(dispatch_handler, self.subscribe_on_events());
    }
}
