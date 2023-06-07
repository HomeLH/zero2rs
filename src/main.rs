use zero2rs::telemetry::{get_subscriber, init_subscriber};
use zero2rs::configuration;
use zero2rs::startup::Application;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2rs".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    let config = configuration::get_configuration().expect("Fail to read configuration file.");
    Application::build(config).await?.run_until_stopped().await?;
    Ok(())
}
