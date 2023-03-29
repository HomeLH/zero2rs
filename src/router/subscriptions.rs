use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;
use chrono::Utc;
#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    email: String,
    name: String
}

// async fn subscribe(_req: HttpRequest) -> HttpResponse {
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "subscribe",
        %request_id,
        subscriber_email = %form.email,
        subscriber_name = %form.name
    );

    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!(
        "Insert one record into database"
    );
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now(),
    )
    .execute(pool.get_ref())
    .instrument(query_span)
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            tracing::info!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}