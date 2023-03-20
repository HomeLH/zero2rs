use std::mem::zeroed;

use sqlx::{PgConnection, Connection};
use zero2rs::startup::run;
use zero2rs::configuration::get_configuration;

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

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app_address = spawn_app();

    let cfg = get_configuration().expect("fail to read configuration file");
    let connection_string = cfg.database.connection_database();

    let connection = PgConnection::connect(&connection_string).await.expect("fail to connect to postgres");

    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(&format!("{}/subscriptions", app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(200, response.status().as_u16());

}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the mail"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("","missing both name and email"),
    ];
    for (invalid_body, error_msg) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", app_address))
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");
        assert_eq!(
            400,
            response.status().as_u16(),
            "The api did not fail with 400 bad request when the payload was {}.",
            error_msg
        )
    }
}