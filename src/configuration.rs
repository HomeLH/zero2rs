use config::ConfigError;
use secrecy::{Secret, ExposeSecret};

use crate::domain::SubscriberEmail;
#[derive(serde::Deserialize)]
pub struct Settings{
    pub database: DatabaseSettings,
    pub application_port: u16,
    pub email_client: EmailClientSettings,
}
#[derive(serde::Deserialize)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
}
impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }
}
#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}
impl DatabaseSettings {
    pub fn connection_database(&self) -> Secret<String> {
        Secret::new(format!("postgres://{}:{}@{}:{}/{}",
                self.username, self.password.expose_secret(), self.host, self.port, self.database_name
        ))
    }
    pub fn connection_database_without_db(&self) -> Secret<String> {
        Secret::new(format!("postgres://{}:{}@{}:{}",
                self.username, self.password.expose_secret(), self.host, self.port
        ))
    }
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let mut settings = config::Config::default();
    settings.merge(config::File::with_name("configuration"))?;
    settings.try_into()
}