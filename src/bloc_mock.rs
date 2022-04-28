use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use teloxide::prelude::DependencyMap;

use crate::bloc::BLoC;
use crate::bloc_event::BotBlocEvent;
use crate::bloc_state::BotBlocState;

#[derive(Clone)]
pub struct BotBlocMock {
    event_controller: Sender<BotBlocEvent>,
    event_stream: Receiver<BotBlocEvent>,
    state_controller: Sender<BotBlocState>,
    state_stream: Receiver<BotBlocState>,
}

impl BotBlocMock {
    pub fn new() -> BotBlocMock {
        let (event_controller, event_stream) = async_channel::unbounded::<BotBlocEvent>();
        let (state_controller, state_stream) = async_channel::unbounded::<BotBlocState>();

        BotBlocMock {
            event_controller,
            event_stream,
            state_controller,
            state_stream,
        }
    }

    pub fn get_state_controller(&self) -> Sender<BotBlocState> {
        self.state_controller.clone()
    }
    pub fn get_event_stream(&self) -> Receiver<BotBlocEvent> {
        self.event_stream.clone()
    }
}

impl Default for BotBlocMock {
    fn default() -> Self {
        BotBlocMock::new()
    }
}

#[async_trait]
impl BLoC<BotBlocEvent, BotBlocState> for BotBlocMock {
    fn get_controller(&self) -> Sender<BotBlocEvent> {
        self.event_controller.clone()
    }
    fn get_stream(&self) -> Receiver<BotBlocState> {
        self.state_stream.clone()
    }

    async fn run(&self) {
        unimplemented!()
    }

    async fn run_with_handler(&self, _: crate::bloc::BotUpdateHandler) {
        unimplemented!()
    }

    async fn run_with_webhook(&self, _: String, _: String) {
        unimplemented!()
    }

    async fn run_with_handler_and_webhook(
        &self,
        _: crate::bloc::BotUpdateHandler,
        _: DependencyMap,
        _: String,
        _: String,
    ) {
        unimplemented!()
    }

    async fn run_with_webhook_tls(&self, _: String, _: String, _: String, _: String) {
        unimplemented!()
    }

    async fn run_with_handler_and_webhook_tls(
        &self,
        _: crate::bloc::BotUpdateHandler,
        _: DependencyMap,
        _: String,
        _: String,
        _: String,
        _: String,
    ) {
        unimplemented!()
    }
}
