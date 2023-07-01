use actix_web::{HttpResponse, web::Query};

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(
    name = "Confirm a pending subscriber",
    skip(_parameters),
)]
pub async fn confirm(_parameters: Query<Parameters>) -> HttpResponse {
    HttpResponse::Ok().finish()
}