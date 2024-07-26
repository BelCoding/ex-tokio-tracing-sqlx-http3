use crate::db::*;
use crate::types::*;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// An async PhoneHandler that can add and get phone numbers.
/// The PhoneHandler is backed by a PostgreSQL database.
#[derive(Debug)]
pub struct PhoneHandler {
    /// List of accounts, with just the emails.
    /// Instead of wrapping into Arc<Mutex<Vec<Email>>>, we use the channels so serial-multiplex the calls.
    ///
    /// If this list is kept updated, it can be useful to reject the emails's requests
    /// that are not in the list (they should have been added first).
    pub accounts: EmailList, // TODO! Update it systematically
}

impl PhoneHandler {
    /// Async constructor.
    /// Gets the list of email accounts from the database.
    #[tracing::instrument]
    pub async fn new(db: &Db) -> Option<PhoneHandler> {
        match db.request_all_email_accounts().await {
            Some(accounts) => Some(PhoneHandler { accounts }),
            None => {
                info!("No email accounts found!");
                None
            }
        }
    }

    /// Spawns a separate thread to receive messages from inc_rx,
    /// handle the operations in database and send the responses via out_tx.
    ///
    /// TODO! Add an usage code example
    #[tracing::instrument]
    pub async fn spawn_db_handler(
        mut self,
        mut db: Db,
        mut inc_rx: mpsc::Receiver<Menu>,
        out_tx: mpsc::Sender<(String, SocketAddr)>,
    ) -> Option<tokio::task::JoinHandle<()>> {
        // tokio spawn the receiver on a separate thread that can block
        let j = tokio::task::Builder::new()
            .name("db_handler")
            .spawn(async move {
                // Receive names via the channel and manage the errors if any.
                while let Some(op) = inc_rx.recv().await {
                    self.handle_operation(op, &mut db, &out_tx).await;
                }
                info!("Channel disconnected!");
            });

        match j {
            Ok(f) => {
                debug!("db_handler spawned!");
                Some(f)
            }
            Err(e) => {
                error!("Error spawning db handler: {:?}", e);
                return None;
            }
        }
    }

    /// Handle the operation requested by the client.
    async fn handle_operation(
        &mut self,
        op: Menu,
        db: &mut Db,
        out_tx: &mpsc::Sender<(String, SocketAddr)>,
    ) {
        match op {
            Menu::Get(email, addr) => match db.request_phone(&email).await {
                Some(num) => {
                    info!("Phone number {} found for {}", num, email);
                    if let Err(e) = out_tx.send((num, addr)).await {
                        error!("Error: Rcv dropped the channel?: {}", e);
                    } else {
                        debug!("Sent the phone number to the channel.");
                    }
                }
                None => info!("No phone number found for {}", email),
            },
            Menu::Add(entry, _addr) => {
                info!("Adding {} with number {}", entry.email, entry.number);
                todo!();
            }
        }
    }
}
