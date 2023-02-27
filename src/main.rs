use actix_web::{web, App, HttpServer, HttpRequest, Responder, HttpResponse};

async fn greet(req: HttpRequest) -> impl Responder{
    let name = req.match_info().get("name").unwrap_or("");
    format!("Hello {}!", name)
}

async fn health_check(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().finish()
}
#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
        .route("/", web::get().to(greet))
        // .route("/{name}", web::get().to(greet))
        .route("/healthcheck", web::get().to(health_check))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
