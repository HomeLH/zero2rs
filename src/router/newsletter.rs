use crate::email_client::EmailClient;
use crate::{domain::SubscriberEmail, router::error_chain_fmt};
use actix_web::ResponseError;
use actix_web::http::header;
use actix_web::{web, HttpResponse, HttpRequest};
use anyhow::Context;
use reqwest::StatusCode;
use sqlx::PgPool;
use secrecy::Secret;
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
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn error_response(&self) -> HttpResponse {
        match self {
            PublishError::UnexpectedError(_) => {
                HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
            }
            PublishError::AuthError(_) => {
                let mut response = HttpResponse::new(StatusCode::UNAUTHORIZED);
                let header_value = header::HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();
                response.headers_mut().insert(
                    header::WWW_AUTHENTICATE,
                    header_value,
                );
                response
            }
        }
    }
}

// publish_newsletter function
// return 200 OK if newsletter is sent successfully.
#[tracing::instrument(name = "Publish a newsletter", skip(_body, pool, email_client))]
pub async fn publish_newsletter(
    _body: web::Json<BodyData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    request: HttpRequest,
) -> Result<HttpResponse, PublishError> {
    let _credentials =  basic_authentication(request.headers())
        .map_err(|e| PublishError::AuthError(e))?;
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

struct Credentials {
    username: String,
    password: Secret<String>,
}

// basic authentication 
fn basic_authentication(
    headers: &header::HeaderMap,
) -> Result<Credentials, anyhow::Error> {
    let header_value = headers
        .get(header::AUTHORIZATION)
        .context("The authorization header is missing")?
        .to_str()
        .context("The authorization header is invalid utf-8")?;
    let base64encoded_segments = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme was not 'Basic '.")?;
    let decoded_bytes = base64::decode_config(base64encoded_segments, base64::STANDARD)
        .context("The authorization header is not base64-encoded.")?;
    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The authorization header contains invalid utf-8")?;

    let mut credentials = decoded_credentials.splitn(2, ':');
    let username = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A username must be provided in 'Basic' auth"))?
        .to_string();
    let password = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A password must be provided in 'Basic' auth"))?
        .to_string();
    Ok(Credentials { username, password:Secret::new(password)})

}