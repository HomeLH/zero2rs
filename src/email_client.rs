use std::time::Duration;
use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{Secret, ExposeSecret};

// define email client structure
pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    authorization_token: Secret<String>
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail, authorization_token: Secret<String>, timeout:Duration) -> Self {
        Self { http_client: Client::builder().timeout(timeout).build().unwrap(), base_url, sender, authorization_token }

    }
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        self.http_client
            .post(&format!("{}/email", self.base_url))
            .header("X-Postmark-Server-Token", self.authorization_token.expose_secret() )
            .json(&SendEmailRequest {
                from: self.sender.as_ref(),
                to: recipient.as_ref(),
                subject,
                html_content,
                text_content,
            })
            .send()
            .await?
            .error_for_status()?;
            // .map_error(|error| {"Fail to send email".to_string()}"})?;
        Ok(())
    }
}
// implement SendEmailRequest structure
#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_content: &'a str,
    text_content: &'a str,
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use secrecy::Secret;
    use fake::{Fake, Faker};
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate, Request};
    use claim::{assert_ok, assert_err};

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                let has_from = body.get("From").is_some();
                let has_to = body.get("To").is_some();
                let has_subject = body.get("Subject").is_some();
                let has_html_content = body.get("HtmlContent").is_some();
                let has_text_content = body.get("TextContent").is_some();
                has_from && has_to && has_subject && has_html_content && has_text_content
            } else {
                false
            }
        }
    }        

    fn subject() -> String {
        Sentence(1..2).fake()
    }
    fn content() -> String {
        Paragraph(1..10).fake()
    }

    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(base_url, email(), Secret::new(Faker.fake()), std::time::Duration::from_millis(200))
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // Arrange
        let mock_server = MockServer::start().await;
        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        //act 
        let _ = email_client(mock_server.uri()).send_email(email(), &subject(), &content(), &content()).await;
    }
    //  server returns 200 if sending email is successful
    #[tokio::test]
    async fn send_email_returns_ok_if_the_server_returns_200() {
        // Arrange
        let mock_server = MockServer::start().await;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        //act 
        let result = email_client(mock_server.uri()).send_email(email(), &subject(), &content(), &content()).await;
        //assert
        assert_ok!(result);
    }
    // server returns 500 if sending email returns error
    #[tokio::test]
    async fn send_email_returns_err_if_the_server_returns_500() {
        // Arrange
        let mock_server = MockServer::start().await;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        //act 
        let result = email_client(mock_server.uri()).send_email(email(), &subject(), &content(), &content()).await;
        //assert
        assert_err!(result);
    }
    // timeout case when sending email takes too long
    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        // Arrange
        let mock_server = MockServer::start().await;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(10)))
            .expect(1)
            .mount(&mock_server)
            .await;

        //act 
        let result = email_client(mock_server.uri()).send_email(email(), &subject(), &content(), &content()).await;
        //assert
        assert_err!(result);
    }
}
