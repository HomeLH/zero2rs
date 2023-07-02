use crate::helpers::spawn_app;
use wiremock::{Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    let response =  app.post_subscriptions(body).await;
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber(){
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&app.connection_pool)
        .await
        .expect("Failed to execute request");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_is_invalid(){
    let app = spawn_app().await;
    let body = "name=le%20guin&email=";
    let response = app.post_subscriptions(body).await;
    assert_eq!(
        400,
        response.status().as_u16(),
        "The API did not return a 400 Bad Request when the payload was {}.",
        body
    );
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app_address = spawn_app().await;
    let test_cases = vec![
        ("name=le%20guin", "missing the mail"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("","missing both name and email"),
    ];
    for (invalid_body, error_msg) in test_cases {
        let response = app_address.post_subscriptions(invalid_body).await;
        assert_eq!(
            400,
            response.status().as_u16(),
            "The api did not fail with 400 bad request when the payload was {}.",
            error_msg
        )
    }
}
#[tokio::test]
pub async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;
    dbg!(&app.email_server.uri());
    app.post_subscriptions(body.into()).await;
}
#[tokio::test]
pub async fn subscribe_sends_a_confirmation_email_with_a_link(){
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;
    app.post_subscriptions(body.into()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(&email_request);
    assert_eq!(confirmation_links.html, confirmation_links.plain_text);
    // let confirmation_links = app.get_confirmation_links(&email_request);
    // assert_eq!(confirmation_links.html, confirmation_links.plain_text);
}

// subscribe fails when database is broken  
#[tokio::test]
pub async fn subscribe_fails_when_the_database_is_unavailable(){
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    sqlx::query!("ALTER TABLE subscriptions DROP COLUMN email").execute(&app.connection_pool).await.unwrap();
    let response = app.post_subscriptions(body.into()).await;
    assert_eq!(response.status().as_u16(), 500);
}
