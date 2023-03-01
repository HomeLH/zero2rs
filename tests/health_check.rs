use zero2rs::run;

#[tokio::test]
async fn health_check() {
    let host = spawn_app();

    let client = reqwest::Client::new();
    let url = format!("{}/healthcheck", host);
    // use cargo test -- --nocapture command shows all println info.
    println!("{}", url);
    let response = client.get(&url).send().await.expect("Failed to get");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() -> String{
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server =  run(listener).expect("Failed to bind server");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}