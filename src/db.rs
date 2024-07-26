use crate::types::*;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;
use tracing::{debug, error, info};

#[derive(Debug)]
pub struct Db {
    pool: Pool<Postgres>,
}

impl Db {
    /// Create a new database pool
    #[tracing::instrument]
    pub async fn new() -> Result<Db, sqlx::Error> {
        // Connect to the database
        dotenvy::dotenv().expect("A .env file with DATABASE_URL is required.");
        let conn_str = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
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
    pub async fn request_phone(&self, email: &Email) -> Option<String> {
        // Get the connection and query the database for the phone number
        let mut conn: sqlx::pool::PoolConnection<Postgres> = match self.pool.acquire().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Error: {:?}", e);
                return None;
            }
        };

        #[cfg(not(test))]
        let res = sqlx::query!("SELECT * FROM contacts WHERE email = $1", email.as_str())
            .fetch_one(&mut *conn)
            .await;

        #[cfg(test)]
        let res = sqlx::query!("SELECT * FROM contacts_t WHERE email = $1", email.as_str())
            .fetch_one(&mut *conn)
            .await;

        match res {
            Ok(record) => return Some(record.number),
            Err(e) => {
                error!("Error: {:?}", e);
                return None;
            }
        };
    }

    /// Get email accounts
    /// In case of failure returns None to the caller after printing the error to the trace.
    #[tracing::instrument]
    pub async fn request_all_email_accounts(&self) -> Option<EmailList> {
        // Get the connection and query the database for the phone number
        let mut conn: sqlx::pool::PoolConnection<Postgres> = match self.pool.acquire().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Error: {:?}", e);
                return None;
            }
        };

        #[cfg(not(test))]
        let res: Result<Vec<Email>, _> = sqlx::query_as!(Email, "SELECT email FROM contacts")
            .fetch_all(&mut *conn)
            .await;

        #[cfg(test)]
        let res: Result<Vec<Email>, _> = sqlx::query_as!(Email, "SELECT email FROM contacts_t")
            .fetch_all(&mut *conn)
            .await;

        let emails: EmailList = match res {
            Ok(records) => records.into_iter().collect(),
            Err(e) => {
                error!("Error: {:?}", e);
                return None;
            }
        };
        debug!(?emails);
        Some(emails)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    fn tests_setup(pool: sqlx::PgPool) -> Db {
        Db { pool }
    }

    #[sqlx::test(fixtures(
        path = "../test-fixtures",
        scripts("create-contacts.sql", "insert-contacts.sql")
    ))]
    async fn load_table_test(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let mut conn: sqlx::pool::PoolConnection<Postgres> = pool.acquire().await?;
        let res = sqlx::query!("SELECT * FROM contacts_t")
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
        let mail: Email = Email::from_str("bob@domain.com").unwrap();
        let res = sqlx::query_as!(
            Contact,
            "SELECT * FROM contacts_t WHERE email = $1",
            mail.as_str()
        )
        .fetch_one(&mut *conn)
        .await;

        let contact = match res {
            Ok(row) => row,
            Err(e) => {
                panic!("Error: {:?}", e);
            }
        };

        assert_eq!(contact.number, format!("234-567-8901"));
        Ok(())
    }

    #[sqlx::test(fixtures(
        path = "../test-fixtures",
        scripts("create-contacts.sql", "insert-contacts.sql")
    ))]
    async fn read_bob_number_test(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let mut conn: sqlx::pool::PoolConnection<Postgres> = pool.acquire().await?;
        let mail: Email = Email::from_str("bob@domain.com").unwrap();
        let res = sqlx::query!(
            "SELECT number FROM contacts_t WHERE email = $1",
            mail.as_str()
        )
        .fetch_one(&mut *conn)
        .await;

        let record = match res {
            Ok(row) => row,
            Err(e) => {
                panic!("Error: {:?}", e);
            }
        };

        assert_eq!(record.number, format!("234-567-8901"));
        Ok(())
    }

    #[sqlx::test(fixtures(
        path = "../test-fixtures",
        scripts("create-contacts.sql", "insert-contacts.sql")
    ))]
    async fn request_phone_charlie_test(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let db = tests_setup(pool);
        let mail: Email = Email::from_str("charlie@domain.com").unwrap();
        let res = db.request_phone(&mail).await;
        assert!(res.is_some());
        assert_eq!(res.unwrap(), format!("345-678-9012"));
        Ok(())
    }

    #[sqlx::test(fixtures(
        path = "../test-fixtures",
        scripts("create-contacts.sql", "insert-contacts.sql")
    ))]
    async fn request_all_email_accounts_test(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let db = tests_setup(pool);
        let res = db.request_all_email_accounts().await;
        assert!(res.is_some());
        let emails: EmailList = res.unwrap();
        assert_eq!(emails.len(), 5);
        Ok(())
    }
}
