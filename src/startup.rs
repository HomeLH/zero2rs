use std::net::TcpListener;
use actix_web::{HttpServer, dev::Server, App, web};
use crate::router::health_check;
use crate::router::subscribe;
// Note: think deeply
pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
        // .route("/{name}", web::get().to(greet))
        .route("/healthcheck", web::get().to(health_check))
        .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();
    Ok(server)
}