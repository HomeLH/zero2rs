use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use zero2rs::telemetry::{get_subscriber, init_subscriber};
use uuid::Uuid;
use zero2rs::configuration::{self, DatabaseSettings};
use secrecy::ExposeSecret;
use zero2rs::startup::{Application, get_connection_pool};
pub struct TestApp {
    pub address: String,
    pub connection_pool: PgPool,
}
impl TestApp {
    pub async fn post_subscriptions(&self, body: &str) -> reqwest::Response {
        let client = reqwest::Client::new();
        let url = format!("{}/subscriptions", self.address);
        client.post(&url).header("Content-Type", "application/x-www-form-urlencoded").body(body.to_owned()).send().await.expect("Failed to post")
    }
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
pub async fn spawn_app() -> TestApp{

    Lazy::force(&TRACING);


    let configuration = {
        let mut c = configuration::get_configuration().expect("failed to get configuration");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application_port = 0;
        c
    };

    configure_database(&configuration.database).await;
    let app = Application::build(configuration.clone()).await.expect("Failed to build app");
    let address = format!("http://127.0.0.1:{}", app.port());
    let _ = tokio::spawn(app.run_until_stopped());
    TestApp {
        address,
        connection_pool: get_connection_pool(&configuration.database),
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
