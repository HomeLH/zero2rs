pub mod configurations;

use actix_web::{web, App, HttpServer, HttpRequest, Responder, HttpResponse, dev::Server};
use std::net::TcpListener;

#[derive(serde::Deserialize, Debug)]
struct FormData {
    email: String,
    name: String
}

async fn greet(req: HttpRequest) -> impl Responder{
    let name = req.match_info().get("name").unwrap_or("");
    format!("Hello {}!", name)
}

async fn health_check(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().finish()
}

// async fn subscribe(_req: HttpRequest) -> HttpResponse {
async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
    println!("{:?}", _form);
    HttpResponse::Ok().finish()
}

// Note: think deeply
pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
        .route("/", web::get().to(greet))
        // .route("/{name}", web::get().to(greet))
        .route("/healthcheck", web::get().to(health_check))
        .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();
    Ok(server)
}