mod db;
use db::Db;
use tracing::info;
// use tracing::{debug, error, info};

#[tokio::main]
#[tracing::instrument]
async fn main() {
    console_subscriber::init();
    let mut db = Db::new().await.unwrap();
    match db.request_phone("eve@domain.com").await {
        Some(phone) => info!("Phone number found: {}", phone),
        None => {
            info!("No phone number found");
        }
    };
}
