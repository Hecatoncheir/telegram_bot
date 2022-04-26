use std::sync::Arc;
use telegram_bot::bloc::{BLoC, BotBloc};
use telegram_bot::bloc_event::BotBlocEvent;
use telegram_bot::bloc_state::BotBlocState;
use teloxide_core::requests::RequesterExt;

#[tokio::main]
async fn main() {
    let token = "1380495565:AAEqMVRo78_3RJgmWmPQ8HZnJAuKZaRLUXU";
    let bot = teloxide::Bot::new(token).auto_send();

    let bloc = BotBloc::new(bot);
    let bloc_reference_counter = Arc::new(bloc);

    let bloc_for_spawn = bloc_reference_counter.clone();
    tokio::spawn(async move {
        for state in bloc_for_spawn.get_stream().recv().await {
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

        // while let Ok(state) = bloc_for_spawn.get_stream().recv().await {
        //     match state {
        //         BotBlocState::Update { chat_id, text } => {
        //             let event = BotBlocEvent::TextToChatSend {
        //                 chat_id,
        //                 text: text.to_string(),
        //             };
        //
        //             let _ = bloc_for_spawn.get_controller().send(event).await;
        //         }
        //         BotBlocState::TextToChatSendSuccessful { .. } => {}
        //     }
        // }
    });

    let bloc_for_run = bloc_reference_counter.clone();
    bloc_for_run.run().await;
}
