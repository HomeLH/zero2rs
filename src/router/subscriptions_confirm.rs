use actix_web::{web, HttpResponse, web::Query};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(
    name = "Confirm a pending subscriber",
    skip(parameters, pool),
)]
pub async fn confirm(parameters: Query<Parameters>, pool: web::Data<PgPool>) -> HttpResponse {
    let id = match get_subscriber_id(&pool, &parameters.subscription_token).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Failed to query database: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    match id {
        Some(id) => {
            if confirm_subscriber(&pool, id).await.is_err() {
                return HttpResponse::InternalServerError().finish();
            }
            HttpResponse::Ok().finish()
        }
        None => HttpResponse::BadRequest().finish(),
    }
    
}

// get subscriber id from token 
pub async fn get_subscriber_id(pool: &PgPool, subscription_token: &str) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscription_id FROM subscriptions_token WHERE subscriptions_token = $1"#,
        subscription_token
    )
    .fetch_optional(pool)
    .await?
    .map(|s| s.subscription_id);
    Ok(result)
}

// confirm subscriber, update status form pending to confirmed in subscriptions table
async fn confirm_subscriber(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        id
    )
    .execute(pool)
    .await?;
    Ok(())
}