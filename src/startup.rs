use crate::configuration::DatabaseSettings;
use crate::configuration::Settings;
use crate::email_client::EmailClient;
use crate::router::{health_check, publish_newsletter, subscribe, confirm};
use actix_web::{dev::Server, web, App, HttpServer};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
pub struct Application {
    port: u16,
    server: Server,
}
impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database);
        let address = format!("127.0.0.1:{}", configuration.application.port);
        // something different
        tracing::info!("server is runing on {}", address);
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let sender_email = configuration
            .email_client
            .sender()
            .expect("invalid email address");
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout,
        );
        let server = run(listener, connection_pool, email_client, configuration.application.base_url)?;
        Ok(Self {
            port,
            server,
        })
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
// Note: think deeply

pub struct ApplicationBaseUrl(pub String);

pub fn run(
    listener: TcpListener,
    pool: PgPool,
    email_client: EmailClient,
    base_url: String,
) -> Result<Server, std::io::Error> {
    let pool = web::Data::new(pool);
    let email_client = web::Data::new(email_client);
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            // .route("/{name}", web::get().to(greet))
            .route("/healthcheck", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .route("/newsletter", web::post().to(publish_newsletter))
            .app_data(pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
    .connect_timeout(std::time::Duration::from_secs(2)) 
    .connect_lazy_with(configuration.connection_database().expose_secret().parse().unwrap())
}
