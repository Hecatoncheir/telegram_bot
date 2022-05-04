#[cfg(test)]
mod bot_bloc_test {
    use crate::bloc::{BLoC, Bloc, BlocEvent, BlocState};
    use std::sync::Arc;
    use teloxide_core::requests::RequesterExt;

    #[tokio::test]
    async fn can_send_event_and_get_state() {
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
                    BlocState::Command { message } => {
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

    #[tokio::test]
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
}
