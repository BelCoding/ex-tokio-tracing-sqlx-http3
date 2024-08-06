extern crate ex_tokio_tracing_sqlx_http3;
use core::net::SocketAddr;
use db::Db;
use ex_tokio_tracing_sqlx_http3::*;
use phonehandler::PhoneHandler;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, instrument, warn};
use types::*;

#[tokio::main]
async fn main() {
    console_subscriber::init();
    let span = tracing::info_span!("main");
    let _guard = span.enter();
    let db = match Db::new().await {
        Ok(db) => db,
        Err(e) => {
            error!("Error creating db object: {:?}", e);
            return;
        }
    };

    let Some(phandler) = PhoneHandler::new(&db).await else {
        error!("Error creating phonebook");
        return;
    };
    info!("PhoneHandler created!");
    debug!(accounts = ?&phandler.accounts, "Accounts:");

    // tx and rx for incoming messages into the server
    let (inc_tx, inc_rx) = mpsc::channel(32);
    // tx and rx for outgoing messages
    let (out_tx, out_rx) = mpsc::channel(32);

    let Some(join_db_handler) = phandler.spawn_db_handler(db, inc_rx, out_tx).await else {
        error!("Error spawning db handler");
        return;
    };

    // Set udp address for the server and bind
    let socket = UdpSocket::bind("0.0.0.0:3000")
        .await
        .expect("Error binding socket");
    let arc_socket: Arc<UdpSocket> = Arc::new(socket);
    let Some((listener_handle, sender_handle)) =
        udp_server(arc_socket.clone(), inc_tx, out_rx).await
    else {
        error!("Error starting udp server");
        drop(arc_socket);
        return;
    };

    debug!("Waiting for the join handlers...");
    // Check the join handlers and close the socket if any of them is interrupted.
    tokio::select! {
        _ = join_db_handler => {
            error!("db_handler aborted!");
        }
        _ = listener_handle => {
            error!("udp_listener aborted!");
        }
        _ = sender_handle => {
            error!("udp_sender aborted!");
        }
    }
    // Make sure to close the socket before existing the program, so that the program does not close it too late.
    drop(arc_socket);
    std::process::exit(0);
}

/// Takes the channel's msgs from out_rx and sends them over udp.
///
/// Receives messages from the udp socket and sends them to inc_tx channel
#[instrument(skip(r_socket, inc_tx, out_rx))]
pub async fn udp_server(
    r_socket: Arc<UdpSocket>,
    inc_tx: mpsc::Sender<Menu>,
    out_rx: mpsc::Receiver<(String, SocketAddr)>,
) -> Option<(JoinHandle<()>, JoinHandle<()>)> {
    info!("Server starting at {:?}", r_socket.local_addr().unwrap());
    debug!(socket = ?r_socket, "Server socket:");
    debug!(inc_tx = ?inc_tx, "Incoming channel:");
    debug!(out_rx = ?out_rx, "Outgoing channel:");

    let s_socket = r_socket.clone();
    let res = tokio::task::Builder::new()
        .name("udp_listener")
        .spawn(async move {
            udp_listener(r_socket, inc_tx).await;
        });

    let listener_handle = match res {
        Ok(f) => {
            debug!("udp_listener spawned!");
            f
        }
        Err(e) => {
            error!("Error spawning udp_listener: {:?}", e);
            return None;
        }
    };

    let res = tokio::task::Builder::new()
        .name("udp_sender")
        .spawn(async move {
            udp_sender(s_socket, out_rx).await;
        });

    let sender_handle = match res {
        Ok(f) => {
            debug!("udp_sender spawned!");
            f
        }
        Err(e) => {
            error!("Error spawning udp_sender: {:?}", e);
            return None;
        }
    };

    Some((listener_handle, sender_handle))
}

#[instrument(skip(s_socket, out_rx))]
async fn udp_sender(s_socket: Arc<UdpSocket>, mut out_rx: mpsc::Receiver<(String, SocketAddr)>) {
    debug!("Start receiving from out_rx");
    while let Some((phone_num, addr)) = out_rx.recv().await {
        // let (phone_num, addr) = out_rx.recv().await.unwrap();
        info!("Sending phone number via UDP {} to {:?}", phone_num, addr);
        let len = s_socket
            .send_to(&phone_num.into_bytes(), addr)
            .await
            .unwrap_or_default();

        debug!("{:?} bytes sent", len);
        if len == usize::default() {
            error!("Error sending to UDP socket");
            break;
        }
    }
    warn!("Ended receiving from internal channel out_rx");
}

#[instrument(skip(r_socket, inc_tx))]
async fn udp_listener(r_socket: Arc<UdpSocket>, inc_tx: mpsc::Sender<Menu>) {
    let mut buf: [u8; 1024] = [0; 1024];
    while let Ok((len, addr)) = r_socket.recv_from(&mut buf).await {
        debug!("{:?} bytes received via UDP from {:?}", len, addr);
        if inc_tx
            .send(Menu::Get(String::from_n_bytes(buf, len).into(), addr))
            .await
            .is_err()
        {
            error!("Error sending to internal channel inc_tx");
            break;
        }
    }
    warn!("Ended listening to the server socket");
}
