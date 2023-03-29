use std::{net::TcpListener};
use sqlx::PgPool;
use env_logger::Env;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new("zero2rs".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    set_global_default(subscriber).expect("Failed to set subscriber");
   
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let config = zero2rs::configuration::get_configuration().expect("Fail to read configuration file.");
    let connection_pool = PgPool::connect(&config.database.connection_database())
        .await
        .expect("Fail to connect to database");
    let address = format!("127.0.0.1:{}", config.application_port); 
    // something different
    println!("{}", address);
    let listener = TcpListener::bind(address)?;
    zero2rs::startup::run(listener, connection_pool)?.await
}
