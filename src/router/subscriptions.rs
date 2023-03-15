use actix_web::{web, HttpResponse};
#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    email: String,
    name: String
}

// async fn subscribe(_req: HttpRequest) -> HttpResponse {
pub async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
    println!("{:?}", _form);
    HttpResponse::Ok().finish()
}