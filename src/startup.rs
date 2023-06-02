use std::net::TcpListener;
use actix_web::{HttpServer, dev::Server, App, web};
use sqlx::PgPool;
use crate::router::health_check;
use crate::router::subscribe;
use crate::email_client::EmailClient;
use tracing_actix_web::TracingLogger;
// Note: think deeply
pub fn run(listener: TcpListener, pool: PgPool, email_client: EmailClient) -> Result<Server, std::io::Error> {
    let pool = web::Data::new(pool);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
        .wrap(TracingLogger::default())
        // .route("/{name}", web::get().to(greet))
        .route("/healthcheck", web::get().to(health_check))
        .route("/subscriptions", web::post().to(subscribe))
        .app_data(pool.clone())
        .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}