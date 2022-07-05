use std::{convert::Infallible, net::SocketAddr};

use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use url::Url;

use warp::http::StatusCode;
use warp::Filter;

use teloxide::adaptors::{AutoSend, DefaultParseMode};
use teloxide::dispatching::stop_token::AsyncStopToken;
use teloxide::dispatching::update_listeners;
use teloxide::dispatching::update_listeners::StatefulListener;
use teloxide::prelude::Requester;
use teloxide::types::Update;

use super::handle_rejection::handle_rejection;

pub async fn webhook_with_tls_for_bot_with_default_parse_mode(
    bot: AutoSend<DefaultParseMode<teloxide::Bot>>,
    http_host: &str,
    web_hook_url: &str,
    cert_path: &str,
    key_path: &str,
) -> impl update_listeners::UpdateListener<Infallible> {
    let url = Url::parse(web_hook_url).unwrap();
    bot.set_webhook(url).await.expect("Cannot setup a webhook");

    let (sender, receiver) = mpsc::unbounded_channel();

    let server = warp::post()
        .and(warp::body::json())
        .map(move |update: Update| {
            sender
                .send(Ok(update))
                .expect("Cannot send an incoming update from the webhook");
            StatusCode::OK
        })
        .or(crate::webhook::health())
        .recover(handle_rejection);

    let (stop_token, stop_flag) = AsyncStopToken::new_pair();

    let addr = http_host.parse::<SocketAddr>().unwrap();

    let server = warp::serve(server)
        .tls()
        .cert_path(cert_path)
        .key_path(key_path);

    let (_addr, future) = server.bind_with_graceful_shutdown(addr, stop_flag);

    tokio::spawn(future);
    let stream = UnboundedReceiverStream::new(receiver);

    fn streamf<S, T>(state: &mut (S, T)) -> &mut S {
        &mut state.0
    }

    StatefulListener::new(
        (stream, stop_token),
        streamf,
        |state: &mut (_, AsyncStopToken)| state.1.clone(),
    )
}
