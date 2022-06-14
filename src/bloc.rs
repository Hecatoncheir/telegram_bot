#[path = "bloc_test.rs"]
mod bloc_test;

use async_channel::{Receiver, Sender};
use async_trait::async_trait;

use teloxide::RequestError;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::prelude::{DependencyMap, Handler};

pub(crate) type BotUpdateHandler =
    Handler<'static, DependencyMap, Result<(), RequestError>, DpHandlerDescription>;

#[async_trait]
pub trait BLoC<Event, State> {
    fn get_controller(&self) -> Sender<Event>;
    fn get_stream(&self) -> Receiver<State>;

    async fn run(&self);
    async fn run_with_handler(&self, handler: BotUpdateHandler);

    async fn run_with_webhook(&self, webhook: String, host: String);
    async fn run_with_handler_and_webhook(
        &self,
        handler: BotUpdateHandler,
        mut dependencies: DependencyMap,
        webhook: String,
        host: String,
    );

    async fn run_with_webhook_tls(
        &self,
        webhook: String,
        host: String,
        cert_path: String,
        key_path: String,
    );
    async fn run_with_handler_and_webhook_tls(
        &self,
        handler: BotUpdateHandler,
        mut dependencies: DependencyMap,
        webhook: String,
        host: String,
        cert_path: String,
        key_path: String,
    );
}
