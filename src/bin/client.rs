extern crate ex_tokio_tracing_sqlx_http3;
use ex_tokio_tracing_sqlx_http3::types::*;
use tokio::net::UdpSocket;
// use tracing::{info, instrument};
use core::net::SocketAddr;
use tracing::debug;
use tracing::info;
// Send udp message to the server at "0.0.0.0:3000" to get the phone number of a name.
#[tracing::instrument]
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    // Set udp address for the server and bind
    let addr = SocketAddr::from(([0, 0, 0, 0], 3009));
    let dest_addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let socket = UdpSocket::bind(addr).await.expect("Error binding socket");

    // request the phone number of karl
    let email = Email::from("charlie@domain.com".to_string());

    let len = socket.send_to(email.as_bytes(), dest_addr).await.unwrap();
    debug!("{:?} bytes sent", len);

    // Wait for the response and print it, which is a String type number.
    let mut buf = [0; 1024];
    let (len, or_addr) = socket.recv_from(&mut buf).await.unwrap();
    debug!("{:?} bytes received from {:?}", len, or_addr);
    let num = String::from_utf8(buf[..len].to_vec()).expect("Invalid UTF-8");
    info!("Phone number for Charlie: {}", num);
}
