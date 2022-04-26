#[cfg(test)]
mod bot_bloc_test {
    use crate::bloc::{BLoC, BotBloc, BotBlocEvent, BotBlocState};
    use std::sync::Arc;
    use teloxide_core::requests::RequesterExt;

    #[tokio::test]
    async fn can_send_event_and_get_state() {
        let token = "1380495565:AAEqMVRo78_3RJgmWmPQ8HZnJAuKZaRLUXU";
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

                        let _ = bloc_for_spawn.get_controller().send(event).await;
                    }
                    BotBlocState::TextToChatSendSuccessful { .. } => {}
                }
            }
        });

        let bloc_for_run = bloc_reference_counter.clone();
        bloc_for_run.run().await;
    }
}
