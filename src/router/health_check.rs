use actix_web::{HttpResponse, HttpRequest};
pub async fn health_check(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().finish()
}