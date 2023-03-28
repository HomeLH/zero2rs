use std::net::TcpListener;
use actix_web::{HttpServer, dev::Server, App, web, middleware::Logger};
use sqlx::PgPool;
use crate::router::health_check;
use crate::router::subscribe;
// Note: think deeply
pub fn run(listener: TcpListener, pool: PgPool) -> Result<Server, std::io::Error> {
    let pool = web::Data::new(pool);
    let server = HttpServer::new(move || {
        App::new()
        .wrap(Logger::default())
        // .route("/{name}", web::get().to(greet))
        .route("/healthcheck", web::get().to(health_check))
        .route("/subscriptions", web::post().to(subscribe))
        .app_data(pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}