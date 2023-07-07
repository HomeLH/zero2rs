use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use sqlx::{PgPool, Transaction, Postgres};
use uuid::Uuid;
use chrono::Utc;
use rand::{thread_rng, Rng};
use unicode_segmentation::UnicodeSegmentation;
use crate::{domain::{SubscriberName, NewSubscriber, SubscriberEmail}, email_client::{EmailClient}, startup::ApplicationBaseUrl};

#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("{0}")]
    ValidationError(String),
}
impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

pub fn error_chain_fmt(e: &impl std::error::Error, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //iterate over the cause chain
    writeln!(f, "{}", e)?;
    let mut e = e.source();
    while let Some(cause) = e {
        writeln!(f, "caused by: {}", cause)?;
        e = cause.source();
    }
    Ok(())
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            SubscribeError::ValidationError(_) => actix_web::http::StatusCode::BAD_REQUEST,
            SubscribeError::UnexpectedError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::new(self.status_code())
    }
} 

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
    skip(form, pool, email_client, base_url),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>, email_client: web::Data<EmailClient>
    , base_url: web::Data<ApplicationBaseUrl>) -> Result<HttpResponse, SubscribeError> {
    let new_subscriber = form.try_into().map_err(|e| SubscribeError::ValidationError(e))?; 
    let mut transaction =  pool.begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;

    // trying to access the data field by using from.0.name instead of from reference
    let subscriber_id =  insert_subscriber(&mut transaction, &new_subscriber)
    .await
    .context("Failed to insert new subscriber in the database")?;
    let subscription_token = generate_subscription_token();

    store_token(&mut transaction, subscriber_id, &subscription_token)
    .await
    .context("Failed to store subscription token in the database")?;

    send_confirmation_email(email_client, new_subscriber, &base_url.0, &subscription_token)
    .await
    .context("Failed to send confirmation email")?;
    

    transaction.commit()
    .await
    .context("Failed to commit SQL transaction")?;

    Ok(HttpResponse::Ok().finish())
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
    skip(transaction, new_subscripber)
)]
pub async fn insert_subscriber(transaction: &mut Transaction<'_, Postgres>, new_subscripber: &NewSubscriber) -> Result<Uuid, sqlx::Error> {
    let uid = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        uid,
        new_subscripber.email.as_ref(),
        new_subscripber.name.as_ref(),
        Utc::now(),
    )
    .execute(transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(uid)
}
#[tracing::instrument(
    name = "Sending confirmation email",
    skip(email_client, new_subscripber, base_url, subscription_token),
)]
pub async fn send_confirmation_email(
    email_client: web::Data<EmailClient>,
    new_subscripber: NewSubscriber,
    base_url: &str,
    subscription_token: &str,
)-> Result<(), reqwest::Error>{
    let confirmation_link = format!("{}/subscriptions/confirm?subscription_token={}", base_url, subscription_token);
    // todo uuid for confirmed link
    email_client.send_email(
        &new_subscripber.email,
        "Welcome", 
        &format!("welcome to our newsletter! <br /> Click <a href=\"{}\">here</a> to confirm your subscription.", confirmation_link),
        &format!("welcome to our newsletter! \n Visit {} to confirm your subscription.", confirmation_link)
    )
    .await
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    let token: String = std::iter::repeat(())
        .map(|()| rng.sample(rand::distributions::Alphanumeric))
        .map(char::from)
        .take(25)
        .collect();
    token
}

async fn store_token(transaction: &mut Transaction<'_, Postgres>, subscriber_id: Uuid, subscription_token: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions_token (subscriptions_token, subscription_id)
        VALUES ($1, $2)
        "#,
        subscription_token,
        subscriber_id,
    )
    .execute(transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}