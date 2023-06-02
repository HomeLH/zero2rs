use std::{net::TcpListener};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use zero2rs::telemetry::{get_subscriber, init_subscriber};
use zero2rs::email_client::EmailClient;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2rs".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
   
    let config = zero2rs::configuration::get_configuration().expect("Fail to read configuration file.");
    let connection_pool = PgPool::connect(&config.database.connection_database().expose_secret())
        .await
        .expect("Fail to connect to database");
    let address = format!("127.0.0.1:{}", config.application_port); 
    // something different
    tracing::info!("server is runing on {}", address);
    let listener = TcpListener::bind(address)?;
    let sender_email = config.email_client.sender().expect("invalid email address");
    let email_client = EmailClient::new(config.email_client.base_url, sender_email);
    zero2rs::startup::run(listener, connection_pool, email_client)?.await
}
