use std::sync::Arc;

use teloxide_core::requests::RequesterExt;

use telegram_bot::bloc::{BLoC, BotBloc};
use telegram_bot::bloc_event::BotBlocEvent;
use telegram_bot::bloc_state::BotBlocState;

#[tokio::main]
async fn main() {
    let token = "1380495565:AAEqMVRo78_3RJgmWmPQ8HZnJAuKZaRLUXU";
    let bot = teloxide::Bot::new(token).auto_send();

    let bloc = BotBloc::new(bot);
    let bloc_reference_counter = Arc::new(bloc);

    let bloc_for_spawn = bloc_reference_counter.clone();
    tokio::spawn(async move {
        while let Ok(state) = bloc_for_spawn.get_stream().recv().await {
            match state {
                BotBlocState::UpdateMessage { message } => {
                    let chat_id = message.chat.id.0;
                    let text = message.text().unwrap().to_string();

                    let event = BotBlocEvent::TextToChatSend { chat_id, text };

                    let _ = bloc_for_spawn.get_controller().send(event).await;
                }
                BotBlocState::TextToChatSendSuccessful { .. } => {}
            }
        }
    });

    let bloc_for_run = bloc_reference_counter.clone();
    bloc_for_run.run().await;
}
