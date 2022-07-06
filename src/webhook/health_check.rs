#[path = "health_check_test.rs"]
mod health_check_test;

use serde_derive::{Deserialize, Serialize};
use warp::Filter;

#[derive(Deserialize, Serialize)]
struct HealthStatus {
    status: String,
}

/// Defining a function that returns a filter.
pub fn filter() -> impl Filter<Extract = (warp::reply::Json,), Error = warp::Rejection> + Copy {
    warp::get().and(warp::path("health").map(|| {
        let status = HealthStatus {
            status: "OK".to_string(),
        };
        warp::reply::json(&status)
    }))
}
