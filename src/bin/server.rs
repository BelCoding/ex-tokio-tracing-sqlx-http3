extern crate ex_tokio_tracing_sqlx_http3;
use ex_tokio_tracing_sqlx_http3::db::Db;
use tracing::info;

// This code is just to populate the server binary: cargo run --bin server
// TODO! Implement the server side of the application
#[tokio::main]
#[tracing::instrument]
async fn main() {
    console_subscriber::init();
    let db = Db::new().await.unwrap();
    match db.request_phone(&"eve@domain.com".to_string().into()).await {
        Some(phone) => info!("Phone number found: {}", phone),
        None => {
            info!("No phone number found");
        }
    };
}
