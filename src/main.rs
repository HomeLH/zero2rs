use zero2rs::run; 
use std::net::TcpListener;
#[tokio::main]
async fn main() -> std::io::Result<()> {
    // something different
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    println!("http://127.0.0.1:{}", port);
    run(listener)?.await
}
