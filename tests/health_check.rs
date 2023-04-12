use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{PgConnection, Connection, PgPool, Executor};
use zero2rs::startup::run;
use zero2rs::configuration::{self, DatabaseSettings};
use zero2rs::telemetry::{get_subscriber, init_subscriber};
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub connection_pool: PgPool,
}
static TRACING: Lazy<()> = Lazy::new(|| {

    let default_filter_level = "debug".to_string();
    let test_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(test_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(test_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

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

async fn spawn_app() -> TestApp{

    Lazy::force(&TRACING);

    let mut config = configuration::get_configuration().expect("failed to get configuration");
    config.database.database_name = Uuid::new_v4().to_string();

    let connection_pool = configure_database(&config.database).await;
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server =  run(listener, connection_pool.clone()).expect("Failed to bind server");
    let _ = tokio::spawn(server);
    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        connection_pool
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_database_without_db().expose_secret())
        .await
        .expect("Failed to connect to database");
    connection.execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");
    
    let connection_pool = PgPool::connect(&config.connection_database().expose_secret())
       .await
       .expect("Failed to connect to database");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate");
    connection_pool
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(&format!("{}/subscriptions", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(200, response.status().as_u16());
    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.connection_pool)
        .await
        .expect("Failed to execute request");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");

}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_is_invalid(){
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(&format!("{}/subscriptions", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(
        400,
        response.status().as_u16(),
        "The API did not return a 400 Bad Request when the payload was {}.",
        body
    );
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the mail"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("","missing both name and email"),
    ];
    for (invalid_body, error_msg) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", app_address.address))
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