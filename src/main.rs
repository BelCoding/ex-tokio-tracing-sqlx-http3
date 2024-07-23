mod db;
use db::Db;
use tracing::debug;
// use tracing::{debug, error, info};

#[tokio::main]
#[tracing::instrument]
async fn main() {
    console_subscriber::init();
    let db = Db::new().await.unwrap();
    debug!(?db.pool);
}
