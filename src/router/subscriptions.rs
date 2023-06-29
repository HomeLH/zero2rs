use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use unicode_segmentation::UnicodeSegmentation;
use crate::{domain::{SubscriberName, NewSubscriber, SubscriberEmail}, email_client::{EmailClient}};
#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    email: String,
    name: String
}

impl TryFrom<web::Form<FormData>> for NewSubscriber {
    type Error = String;
    /// Performs the conversion.
    fn try_from(form: web::Form<FormData>) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(form.0.name)?;
        let email = SubscriberEmail::parse(form.0.email)?;
        Ok(NewSubscriber { email, name })
    }

}

pub fn parse_subscriber(form: web::Form<FormData>) -> Result<NewSubscriber, String> {
    let name = SubscriberName::parse(form.0.name)?;
    let email = SubscriberEmail::parse(form.0.email)?;
    Ok(NewSubscriber { email, name })
}
// async fn subscribe(_req: HttpRequest) -> HttpResponse {
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool, email_client),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>, email_client: web::Data<EmailClient>) -> HttpResponse {
    let new_subscriber = match form.try_into() {
        Ok(s) => s,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    // trying to access the data field by using from.0.name instead of from reference
     match  insert_subscriber(&pool, &new_subscriber).await
    {
        Ok(_) => {            
            if send_confirmation_email(email_client, new_subscriber)
            .await
            .is_err() {
                return HttpResponse::InternalServerError().finish();
            }
            return HttpResponse::Ok().finish();
        }
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
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
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

pub async fn send_confirmation_email(
    email_client: web::Data<EmailClient>,
    new_subscripber: NewSubscriber,
)-> Result<(), reqwest::Error>{
    let confirmation_link = format!("{}/subscriptions/confirm?subscription_token={}", "https://my-api.com", Uuid::new_v4());
    // todo uuid for confirmed link
    email_client.send_email(
        new_subscripber.email,
        "Welcome", 
        &format!("welcome to our newsletter! <br /> Click <a href=\"{}\">here</a> to confirm your subscription.", confirmation_link),
        &format!("welcome to our newsletter! \n Visit {} to confirm your subscription.", confirmation_link)
    )
    .await
}