use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use unicode_segmentation::UnicodeSegmentation;
use crate::domain::{SubscriberName, NewSubscriber, SubscriberEmail};
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
    // trying to access the data field by using from.0.name instead of from reference
    let name = match SubscriberName::parse(form.0.name) {
        Ok(name) => name,
        Err(_err) => {
            return HttpResponse::BadRequest().finish();
        }
    };
    let email = match SubscriberEmail::parse(form.0.email) {
        Ok(email) => email,
        Err(_err) => {
            return HttpResponse::BadRequest().finish();
        }
    };
    let new_subscriber = NewSubscriber{
        email,
        name
    };
    match  insert_subscriber(&pool, &new_subscriber).await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            tracing::info!("Failed to execute query: {:?}", e);
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
    skip(pool, new_subscripber)
)]
pub async fn insert_subscriber(pool: &PgPool, new_subscripber: &NewSubscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_subscripber.email.as_ref(),
        new_subscripber.name.as_ref(),
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