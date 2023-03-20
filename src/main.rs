use std::{net::TcpListener};
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = zero2rs::configuration::get_configuration().expect("Fail to read configuration file.");
    let address = format!("127.0.0.1:{}", config.application_port); 
    // something different
    println!("{}", address);
    let listener = TcpListener::bind(address)?;
    zero2rs::startup::run(listener)?.await
}
