use sqlx::{postgres::PgPoolOptions, Pool, Postgres, Row};
// use std::{future::Future, net::SocketAddr, thread::JoinHandle};
// use std::time::Duration;
use tracing::{error, info};
// use tracing::{debug, error, info};

#[derive(Debug)]
pub struct Db {
    pub pool: Pool<Postgres>,
}

impl Db {
    /// Create a new database pool
    #[tracing::instrument]
    pub async fn new() -> Result<Db, sqlx::Error> {
        // Connect to the database
        let conn_str = std::env::var("DATABASE_URL")
            .expect("Env var DATABASE_URL is required for this example.");

        let pool = PgPoolOptions::new()
            .min_connections(2)
            .max_connections(20)
            .connect(&conn_str)
            .await?;

        info!("Db pool created!");
        Ok(Db { pool })
    }

    /// Request the phone number for a given email address.
    /// In case of failure returns None to the caller after printing the error to the trace.
    #[tracing::instrument]
    pub async fn request_phone(&mut self, email: &str) -> Option<String> {
        // Get the connection and query the database for the phone number
        let mut conn: sqlx::pool::PoolConnection<Postgres> = match self.pool.acquire().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Error: {:?}", e);
                return None;
            }
        };
        let res = sqlx::query("SELECT * FROM contacts WHERE email = $1")
            .bind(email)
            .fetch_one(&mut *conn)
            .await;

        let row = match res {
            Ok(row) => row,
            Err(e) => {
                error!("Error: {:?}", e);
                return None;
            }
        };

        let number: String = row.get::<String, _>("number");
        Some(number)
    }
}

#[sqlx::test(fixtures(
    path = "../test-fixtures",
    scripts("create-contacts.sql", "insert-contacts.sql")
))]
async fn load_table_test(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let mut conn: sqlx::pool::PoolConnection<Postgres> = pool.acquire().await?;
    let res = sqlx::query("SELECT * FROM contacts")
        .fetch_all(&mut *conn)
        .await;

    assert!(res.is_ok());
    let rows = res.unwrap();
    // Assert the number of rows is equal to 5
    assert_eq!(rows.len(), 5);
    Ok(())
}

#[sqlx::test(fixtures(
    path = "../test-fixtures",
    scripts("create-contacts.sql", "insert-contacts.sql")
))]
async fn read_bob_test(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let mut conn: sqlx::pool::PoolConnection<Postgres> = pool.acquire().await?;
    let mail = "bob@domain.com";
    let res = sqlx::query("SELECT * FROM contacts WHERE email = $1")
        .bind(mail)
        .fetch_one(&mut *conn)
        .await;

    let row = match res {
        Ok(row) => row,
        Err(e) => {
            panic!("Error: {:?}", e);
        }
    };

    let val = row.get::<String, _>("number");
    assert_eq!(val, format!("234-567-8901"));
    Ok(())
}

#[sqlx::test(fixtures(
    path = "../test-fixtures",
    scripts("create-contacts.sql", "insert-contacts.sql")
))]
async fn read_bob_number_test(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let mut conn: sqlx::pool::PoolConnection<Postgres> = pool.acquire().await?;
    let mail = "bob@domain.com";
    let res = sqlx::query("SELECT number FROM contacts WHERE email = $1")
        .bind(mail)
        .fetch_one(&mut *conn)
        .await;

    let row = match res {
        Ok(row) => row,
        Err(e) => {
            panic!("Error: {:?}", e);
        }
    };

    let val: String = row.get(0);
    assert_eq!(val, format!("234-567-8901"));
    Ok(())
}

#[sqlx::test(fixtures(
    path = "../test-fixtures",
    scripts("create-contacts.sql", "insert-contacts.sql")
))]
async fn request_phone_charlie_test(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let mut db = Db { pool };
    let mail = "charlie@domain.com";
    let res = db.request_phone(mail).await;
    assert!(res.is_some());
    assert_eq!(res.unwrap(), format!("345-678-9012"));
    Ok(())
}
