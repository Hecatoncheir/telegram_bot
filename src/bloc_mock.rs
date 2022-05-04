use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use teloxide::prelude::DependencyMap;

use crate::bloc::BLoC;
use crate::bloc_event::BlocEvent;
use crate::bloc_state::BlocState;

#[derive(Clone)]
pub struct BlocMock {
    event_controller: Sender<BlocEvent>,
    event_stream: Receiver<BlocEvent>,
    state_controller: Sender<BlocState>,
    state_stream: Receiver<BlocState>,
}

impl BlocMock {
    pub fn new() -> BlocMock {
        let (event_controller, event_stream) = async_channel::unbounded::<BlocEvent>();
        let (state_controller, state_stream) = async_channel::unbounded::<BlocState>();

        BlocMock {
            event_controller,
            event_stream,
            state_controller,
            state_stream,
        }
    }

    pub fn get_state_controller(&self) -> Sender<BlocState> {
        self.state_controller.clone()
    }
    pub fn get_event_stream(&self) -> Receiver<BlocEvent> {
        self.event_stream.clone()
    }
}

impl Default for BlocMock {
    fn default() -> Self {
        BlocMock::new()
    }
}

#[async_trait]
impl BLoC<BlocEvent, BlocState> for BlocMock {
    fn get_controller(&self) -> Sender<BlocEvent> {
        self.event_controller.clone()
    }
    fn get_stream(&self) -> Receiver<BlocState> {
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
