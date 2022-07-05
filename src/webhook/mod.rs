mod handle_rejection;
mod health_test;
pub mod webhook_with_tls;
pub mod webhook_with_tls_for_bot_with_default_parse_mode;
pub mod webhook_without_tls;
pub mod webhook_without_tls_for_bot_with_default_parse_mode;

use serde_derive::{Deserialize, Serialize};
use warp::Filter;

#[derive(Deserialize, Serialize)]
struct HealthStatus {
    status: String,
}

fn health() -> impl Filter<Extract = (warp::reply::Json,), Error = warp::Rejection> + Copy {
    warp::get().and(warp::path("health").map(|| {
        let status = HealthStatus {
            status: "OK".to_string(),
        };
        warp::reply::json(&status)
    }))
}
