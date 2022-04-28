## Telegram Bot

### Example

#### Just run:
```rust
#[tokio::main]
async fn can_send_event_and_get_state_throw_webhook() {
    let token = "";
    let bot = teloxide::Bot::new(token).auto_send();

    let bloc = BotBloc::new(bot);
    let bloc_reference_counter = Arc::new(bloc);

    let bloc_for_spawn = bloc_reference_counter.clone();
    tokio::spawn(async move {
        while let Ok(state) = bloc_for_spawn.get_stream().recv().await {
            match state {
                BotBlocState::Message { message } => {
                    let chat_id = message.chat.id.0;
                    let text = message.text().unwrap().to_string();

                    let event = BotBlocEvent::TextToChatSend { chat_id, text };
                    let _ = bloc_for_spawn.get_controller().send(event).await;
                }
                _ => {}
            }
        }
    });

    tokio::spawn(async move {
        let bloc_for_run = bloc_reference_counter.clone();
        bloc_for_run.run().await;
    });

    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }
}
```

#### Run with webhook:

```rust

#[tokio::main]
async fn can_send_event_and_get_state_throw_webhook() {
    let token = "";
    let bot = teloxide::Bot::new(token).auto_send();

    let bloc = BotBloc::new(bot);
    let bloc_reference_counter = Arc::new(bloc);

    let bloc_for_spawn = bloc_reference_counter.clone();
    tokio::spawn(async move {
        while let Ok(state) = bloc_for_spawn.get_stream().recv().await {
            match state {
                BotBlocState::Message { message } => {
                    let chat_id = message.chat.id.0;
                    let text = message.text().unwrap().to_string();

                    let event = BotBlocEvent::TextToChatSend { chat_id, text };
                    let _ = bloc_for_spawn.get_controller().send(event).await;
                }
                _ => {}
            }
        }
    });

    let webhook = "https://b48d-128-69-253-220.ngrok.io".to_string();
    let host = "127.0.0.1:8000".to_string();

    tokio::spawn(async move {
        let bloc_for_run = bloc_reference_counter.clone();
        bloc_for_run.run_with_webhook(webhook, host).await;
    });

    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }
}
```


#### Get file:

```rust
use std::sync::Arc;

use teloxide_core::requests::RequesterExt;
use teloxide_core::types::{MediaKind, MessageKind};

use telegram_bot::bloc::{BLoC, BotBloc};
use telegram_bot::bloc_event::BotBlocEvent;
use telegram_bot::bloc_event::BotBlocEvent::GetFile;
use telegram_bot::bloc_state::BotBlocState;

#[tokio::main]
async fn main() {
    let token = "";
    let bot = teloxide::Bot::new(token).auto_send();

    let bloc = BotBloc::new(bot);
    let bloc_reference_counter = Arc::new(bloc);

    let bloc_for_spawn = bloc_reference_counter.clone();
    tokio::spawn(async move {
        while let Ok(state) = bloc_for_spawn.get_stream().recv().await {
            match state {
                BotBlocState::Message { message } => match message.clone().kind {
                    MessageKind::Common(common_message) => match common_message.media_kind {
                        MediaKind::Photo(media) => {
                            let file_id = &media.photo.last().unwrap().file_id;
                            
                            let event = GetFile {
                                file_id: file_id.clone(),
                            };
                            let _ = bloc_for_spawn.get_controller().send(event).await;

                            let mut file = None;
                            while let Ok(state) = bloc_for_spawn.get_stream().recv().await {

                                match state {
                                    BotBlocState::GetFileSuccessful {
                                        file_id: state_file_id,
                                        file: state_file,
                                    } => {
                                        if *file_id != state_file_id {
                                            continue;
                                        }

                                        file = Some(state_file);
                                        break;
                                    }
                                    BotBlocState::GetFileUnsuccessful {
                                        file_id: state_file_id,
                                    } => {
                                        if *file_id != state_file_id {
                                            continue;
                                        }

                                        let log_message = "Can't get file.".to_string();
                                        log::warn!("{}", log_message);
                                        return;
                                    }
                                    _ => {}
                                }
                            }

                        }
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            }
        }
    });

    tokio::spawn(async move {
        let bloc_for_run = bloc_reference_counter.clone();
        bloc_for_run.run().await;
    });

    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }
}
```