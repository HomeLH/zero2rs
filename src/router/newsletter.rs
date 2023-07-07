use crate::email_client::EmailClient;
use crate::{domain::SubscriberEmail, router::error_chain_fmt};
use actix_web::ResponseError;
use actix_web::{web, HttpResponse};
use anyhow::Context;
use sqlx::PgPool;
// define a struct called BodyData that contains both title and content fields.
#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}
// define Content struct that contains plain text and html text.
#[derive(serde::Deserialize)]
pub struct Content {
    plain: String,
    html: String,
}
struct ConfirmedSubscriber {
    email: SubscriberEmail,
}
// define some error types
#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            PublishError::UnexpectedError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::new(self.status_code())
    }
}

// publish_newsletter function
// return 200 OK if newsletter is sent successfully.
#[tracing::instrument(name = "Publish a newsletter", skip(_body, pool, email_client))]
pub async fn publish_newsletter(
    _body: web::Json<BodyData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, PublishError> {
    let subscribers = get_confirmed_subscribers(&pool).await?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                let _result = email_client
                    .send_email(
                        &subscriber.email,
                        &_body.title,
                        &_body.content.plain,
                        &_body.content.html,
                    )
                    .await
                    .with_context(|| {
                        format!("Failed to send email to {}", subscriber.email.as_ref())
                    })?;
            }
            Err(e) => {
                // comment error.cause_chain = ?e
                tracing::warn!(error.cause_chain = ?e, "skipping invalid subscriber email");
            }
        }
    }
    Ok(HttpResponse::Ok().finish())
}
// get confirmed subscriber from database
// return a vector of ConfirmedSubscriber
pub async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let confirmed_subscribers = sqlx::query!(
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pool)
    .await?
    // comment: what is the difference between query and query_as?
    // query_as! is a macro that allows you to specify the type of the result set
    // A variant of query which takes a path to an explicitly defined struct as the output type.
    // Note: use query_as! to specify the type of the result set, when add some fields to the struct
    .into_iter()
    .map(|row| match SubscriberEmail::parse(row.email) {
        Ok(email) => Ok(ConfirmedSubscriber { email }),
        Err(e) => Err(anyhow::anyhow!(e)),
    })
    .collect();
    Ok(confirmed_subscribers)
}
