
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use zero2rs::telemetry::{get_subscriber, init_subscriber};
use uuid::Uuid;
use zero2rs::email_client::EmailClient;
use zero2rs::startup::run;
use zero2rs::configuration::{self, DatabaseSettings};
use secrecy::ExposeSecret;
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
pub async fn spawn_app() -> TestApp{

    Lazy::force(&TRACING);

    let mut config = configuration::get_configuration().expect("failed to get configuration");
    config.database.database_name = Uuid::new_v4().to_string();

    let connection_pool = configure_database(&config.database).await;
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let sender_email = config.email_client.sender().expect("invalid email address");
    // copy timeout 
    // value borrowed here after partial move
    let timeout = config.email_client.timeout();
    let email_client = EmailClient::new(config.email_client.base_url, sender_email, config.email_client.authorization_token, timeout);
    let server =  run(listener, connection_pool.clone(), email_client).expect("Failed to bind server");
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