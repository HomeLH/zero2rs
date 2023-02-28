use zero2rs::run;

#[tokio::test]
async fn health_check() {
    spawn_app();

    let client = reqwest::Client::new();
    let url = "http://127.0.0.1:8080/healthcheck";
    let response = client.get(url).send().await.expect("Failed to get");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() {
    let server =  run().expect("Failed to bind server");
    let _ = tokio::spawn(server);
}