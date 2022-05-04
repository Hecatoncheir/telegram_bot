## Telegram Bot

### Example

#### Just run:
```rust

use std::sync::Arc;

use telegram_bot::bloc::{BLoC, Bloc};
use telegram_bot::bloc_event::BlocEvent;
use telegram_bot::bloc_event::BlocEvent::GetFile;
use telegram_bot::bloc_state::BlocState;

use teloxide_core::requests::RequesterExt;
use teloxide_core::types::{MediaKind, MessageKind};

#[tokio::main]
async fn can_send_event_and_get_state_throw_webhook() {
    let token = "";
    let bot = teloxide::Bot::new(token).auto_send();

    let bloc = Bloc::new(bot);
    let bloc_reference_counter = Arc::new(bloc);

    let bloc_for_spawn = bloc_reference_counter.clone();
    tokio::spawn(async move {
        while let Ok(state) = bloc_for_spawn.get_stream().recv().await {
            match state {
                BlocState::Message { message } => {
                    let chat_id = message.chat.id.0;
                    let text = message.text().unwrap().to_string();

                    let event = BlocEvent::TextToChatSend { chat_id, text };
                    let _ = bloc_for_spawn.get_controller().send(event).await;
                }
                _ => {}
            }
        }
    });

    let bloc_for_run = bloc_reference_counter.clone();
    bloc_for_run.run().await;
}
```

#### Run with webhook:

```rust

#[tokio::main]
async fn can_send_event_and_get_state_throw_webhook() {
    let token = "";
    let bot = teloxide::Bot::new(token).auto_send();

    let bloc = Bloc::new(bot);
    let bloc_reference_counter = Arc::new(bloc);

    let bloc_for_spawn = bloc_reference_counter.clone();
    tokio::spawn(async move {
        while let Ok(state) = bloc_for_spawn.get_stream().recv().await {
            match state {
                BlocState::Message { message } => {
                    let chat_id = message.chat.id.0;
                    let text = message.text().unwrap().to_string();

                    let event = BlocEvent::TextToChatSend { chat_id, text };
                    let _ = bloc_for_spawn.get_controller().send(event).await;
                }
                _ => {}
            }
        }
    });

    let webhook = "https://b48d-128-69-253-220.ngrok.io".to_string();
    let host = "127.0.0.1:8000".to_string();

    let bloc_for_run = bloc_reference_counter.clone();
    bloc_for_run.run_with_webhook(webhook, host).await;
}
```


#### Get file:

```rust
use std::sync::Arc;

use teloxide_core::requests::RequesterExt;
use teloxide_core::types::{MediaKind, MessageKind};

use telegram_bot::bloc::{BLoC, Bloc};
use telegram_bot::bloc_event::BlocEvent;
use telegram_bot::bloc_event::BlocEvent::GetFile;
use telegram_bot::bloc_state::BlocState;

#[tokio::main]
async fn main() {
    let token = "";
    let bot = teloxide::Bot::new(token).auto_send();

    let bloc = Bloc::new(bot);
    let bloc_reference_counter = Arc::new(bloc);

    let bloc_for_spawn = bloc_reference_counter.clone();
    tokio::spawn(async move {
        while let Ok(state) = bloc_for_spawn.get_stream().recv().await {
            match state {
                BlocState::Message { message } => match message.clone().kind {
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
                                    BlocState::GetFileSuccessful {
                                        file_id: state_file_id,
                                        file: state_file,
                                    } => {
                                        if *file_id != state_file_id {
                                            continue;
                                        }

                                        file = Some(state_file);
                                        break;
                                    }
                                    BlocState::GetFileUnsuccessful {
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

    let bloc_for_run = bloc_reference_counter.clone();
    bloc_for_run.run().await;
}
```