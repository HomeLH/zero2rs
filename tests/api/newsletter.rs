use std::path;

use wiremock::matchers::any;
use wiremock::{Mock, ResponseTemplate};
use crate::helpers::{spawn_app, TestApp, ConfirmationLinks};

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter content",
            "html": "<p>Newsletter content</p>",
        }
    });

    let response = app.post_newsletters(&newsletter_request_body).await;

    assert_eq!(response.status().as_u16(), 200);

}
async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks{
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    // we need to send email in subscription api successfully.
    // so we need a mock server to response.
    // response for public sending email api(as usual exteranl api)
    let _mock_guard = Mock::given(path::Path::new("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;
    app.post_subscriptions(body).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    app.get_confirmation_links(&email_request)
}
async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_link = create_unconfirmed_subscriber(app).await.html;
    reqwest::get(confirmation_link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[tokio::test]
pub async fn newsletters_are_delivered_to_confirmed_subscribers() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;

    // response for newsletter api
    Mock::given(path("/email"))
        .and(matcher::method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter content",
            "html": "<p>Newsletter content</p>",
        }
    });

    let response = app.post_newsletters(&newsletter_request_body).await;

    assert_eq!(response.status().as_u16(), 200);
}

// test case for invalid input for newsletter api
#[tokio::test]
pub async fn newsletter_return_400_for_invalid_data() {
    let app = spawn_app().await;
    //create_confirmed_subscriber(&app).await;

    let test_cases = vec![
        (
            serde_json::json!({
                "content": {
                    "text": "Newsletter content",
                    "html": "<p>Newsletter content</p>",
                }
            }),
            "missing the `title` field",
        ),
        (
            serde_json::json!({
                "title": "Newsletter title",
            }),
            "missing the `content` field",
        ),
        (
            serde_json::json!({
                "title": "",
                "content": {
                    "text": "Newsletter content",
                    "html": "<p>Newsletter content</p>",
                }
            }),
            "empty title",
        ),
    ];
    for (invalid_body, error_msg) in test_cases {
        let response = app.post_newsletters(&invalid_body).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_msg
        );
    }
}
