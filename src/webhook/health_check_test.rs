#[tokio::test]
async fn test_filter() {
    let filter = super::filter();
    assert!(!warp::test::request().path("health").matches(&filter).await);

    // let response = warp::test::request()
    //     .path("/health")
    //     .filter(&filter)
    //     .await
    //     .unwrap();

    // assert_eq!(
    //     res.body(),
    //     crate::webhook::HealthStatus {
    //         status: "OK".to_string()
    //     }
    // );
}
