use wiremock::{Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use crate::helpers::spawn_app;
use reqwest::Url;

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    let app = spawn_app().await;
    let response = reqwest::get(&format!("{}/subscriptions/confirm", app.address))
        .await
        .expect("Failed to execute request.");
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;
    let _response = app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let body = serde_json::from_slice::<serde_json::Value>(&email_request.body).unwrap();

    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };
    let raw_confirmation_link = get_link(&body["HtmlContent"].as_str().unwrap());
    let mut confirmation_link = Url::parse(&raw_confirmation_link).unwrap();
    confirmation_link.set_port(Some(app.port)).unwrap();
    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");

    let response = reqwest::get(confirmation_link).await.unwrap();

    assert_eq!(response.status().as_u16(), 200);

}