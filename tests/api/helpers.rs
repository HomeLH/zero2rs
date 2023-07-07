use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;
use zero2rs::configuration::{self, DatabaseSettings};
use zero2rs::startup::{get_connection_pool, Application};
use zero2rs::telemetry::{get_subscriber, init_subscriber};

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}
pub struct TestApp {
    pub address: String,
    pub connection_pool: PgPool,
    pub port: u16,
    pub email_server: MockServer,
}
impl TestApp {
    pub async fn post_subscriptions(&self, body: &str) -> reqwest::Response {
        let client = reqwest::Client::new();
        let url = format!("{}/subscriptions", self.address);
        client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body.to_owned())
            .send()
            .await
            .expect("Failed to post")
    }
    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body = serde_json::from_slice::<serde_json::Value>(&email_request.body).unwrap();

        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
            confirmation_link
                .set_port(Some(self.port))
                .expect("Failed to set port");
            confirmation_link
        };
        let html_link = get_link(&body["HtmlContent"].as_str().unwrap());
        let plain_link = get_link(&body["TextContent"].as_str().unwrap());
        ConfirmationLinks {
            html: html_link,
            plain_text: plain_link,
        }
    }
    // post_newsletters, para: &self, body: json
    pub async fn post_newsletters(&self, body: &serde_json::Value) -> reqwest::Response {
        let client = reqwest::Client::new();
        let url = format!("{}/newsletters", self.address);
        client.post(&url).json(&body).send().await.unwrap()
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
pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = configuration::get_configuration().expect("failed to get configuration");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c.email_client.base_url = email_server.uri();
        c
    };

    configure_database(&configuration.database).await;
    let app = Application::build(configuration.clone())
        .await
        .expect("Failed to build app");
    let address = format!("http://127.0.0.1:{}", app.port());
    let application_port = app.port();
    let _ = tokio::spawn(app.run_until_stopped());
    TestApp {
        address,
        connection_pool: get_connection_pool(&configuration.database),
        port: application_port,
        email_server,
    }
}
async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection =
        PgConnection::connect(&config.connection_database_without_db().expose_secret())
            .await
            .expect("Failed to connect to database");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
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
