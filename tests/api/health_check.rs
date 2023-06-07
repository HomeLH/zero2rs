use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check() {
    let host = spawn_app().await;

    let client = reqwest::Client::new();
    let url = format!("{}/healthcheck", host.address);
    // use cargo test -- --nocapture command shows all println info.
    println!("{}", url);
    let response = client.get(&url).send().await.expect("Failed to get");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}


