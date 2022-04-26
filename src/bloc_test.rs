#[cfg(test)]
mod bot_bloc_test {
    use crate::bloc::{BLoC, BotBloc, BotBlocEvent, BotBlocState};
    use std::sync::Arc;
    use teloxide_core::requests::RequesterExt;

    #[tokio::test]
    async fn can_send_event_and_get_state() {
        let token = "1380495565:AAEqMVRo78_3RJgmWmPQ8HZnJAuKZaRLUXU";
        let bot = teloxide::Bot::new(token).auto_send();

        // let me = bot.get_me().send().await.unwrap();
        // let bot_name = me.user.username.unwrap();

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
                    BotBlocState::TextToChatSendSuccessful { .. } => {}
                    _ => {}
                }
            }
        });

        let bloc_for_run = bloc_reference_counter.clone();
        bloc_for_run.run().await;
    }

    #[tokio::test]
    async fn can_send_event_and_get_state_throw_webhook() {
        let token = "1380495565:AAEqMVRo78_3RJgmWmPQ8HZnJAuKZaRLUXU";
        let bot = teloxide::Bot::new(token).auto_send();

        // let me = bot.get_me().send().await.unwrap();
        // let bot_name = me.user.username.unwrap();

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
                    BotBlocState::TextToChatSendSuccessful { .. } => {}
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
