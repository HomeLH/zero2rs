use actix_web::{web, App, HttpServer, HttpRequest, Responder, HttpResponse, dev::Server};
use std::net::TcpListener;

async fn greet(req: HttpRequest) -> impl Responder{
    let name = req.match_info().get("name").unwrap_or("");
    format!("Hello {}!", name)
}

async fn health_check(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().finish()
}

// Note: think deeply
pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
        .route("/", web::get().to(greet))
        // .route("/{name}", web::get().to(greet))
        .route("/healthcheck", web::get().to(health_check))
    })
    .listen(listener)?
    .run();
    Ok(server)
}