use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use unicode_segmentation::UnicodeSegmentation;
#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    email: String,
    name: String
}

// async fn subscribe(_req: HttpRequest) -> HttpResponse {
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    if !is_valid_name(&form.name){
        return HttpResponse::InternalServerError().finish();
    }
    match  insert_subscriber(&pool, &form).await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            tracing::info!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
pub fn is_valid_name(s: &str) -> bool {
    let is_empty_or_whitespace =  s.trim().is_empty();
    let is_too_long = s.graphemes(true).count() > 256;
    let forbidden_character = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
    let contain_forbidden_character = s.chars().any(|e| forbidden_character.contains(&e));
    !(is_empty_or_whitespace || is_too_long || contain_forbidden_character)
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, pool)
)]
pub async fn insert_subscriber(pool: &PgPool, form: &web::Form<FormData>) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now(),
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}