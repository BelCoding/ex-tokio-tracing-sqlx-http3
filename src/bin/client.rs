extern crate ex_tokio_tracing_sqlx_http3;
// use ex_tokio_tracing_sqlx_http3::types::*;
use tokio::net::UdpSocket;
use tokio::time::{timeout, Duration};
// use tracing::{info, instrument};
use clap::{command, Parser};
use core::net::SocketAddr;
use std::thread::sleep;
use tracing::{debug, error, info};

#[derive(Debug, Parser)]
#[command(about = "Test client to send requests.
Either -e or -m has to be provided.
Port can be specified, otherwise 3001 will be taken by default.")]
struct Arguments {
    #[arg(
        short = 'e',
        long = "email",
        required = false,
        help = "Request the phone for this email."
    )]
    email: Option<String>,
    #[arg(
        short = 'm',
        long = "many",
        required = false,
        num_args = 0,
        help = "Perform a test with a series of many requests."
    )]
    many: bool,
    #[arg(short = 'p', long = "dest-port", required = false)]
    port: Option<u16>,
}

// Send udp message to the server at "0.0.0.0:3000" to get the phone number of a name.
#[tracing::instrument]
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = Arguments::parse();
    debug!(?args.email, ?args.many, ?args.port, "Arguments parsed:");

    let dest_port = args.port.unwrap_or(3001);
    match (args.email, args.many) {
        (Some(email), false) => send_request_to_server(email, dest_port).await,
        // (Some(email), None) => send_request_to_server(email, dest_port).await,
        (None, true) => {
            // (None, Some(_)) => {
            spawn_many_requests(dest_port).await;
        }
        _ => {
            error!("Invalid arguments");
        }
    }
}

async fn spawn_many_requests(dest_port: u16) {
    let emails: Vec<String> = get_emails_vec();
    let emails_copy: Vec<String> = emails.iter().rev().cloned().collect();
    let tot_requests = emails.len() + emails_copy.len();

    let jh: tokio::task::JoinHandle<()> = tokio::spawn(async move {
        for e in emails {
            send_request_to_server(e, dest_port).await;
            sleep(Duration::from_millis(100));
        }
    });

    let jh2: tokio::task::JoinHandle<()> = tokio::spawn(async move {
        for e in emails_copy {
            send_request_to_server(e, dest_port).await;
            sleep(Duration::from_millis(100));
        }
    });

    jh.await.expect("Error in join handle");
    jh2.await.expect("Error in join handle");
    debug!("Tasks joined.");
    debug!(tot_requests, "Spawned tasks.");
}

async fn send_request_to_server(email: String, dest_port: u16) {
    // Set udp dest_addr for the server and bind
    let addr = SocketAddr::from(([0, 0, 0, 0], 0000)); // Will bind a random port
    let dest_addr = SocketAddr::from(([0, 0, 0, 0], dest_port));
    let socket = UdpSocket::bind(addr).await.expect("Error binding socket");

    let len = socket.send_to(email.as_bytes(), dest_addr).await.unwrap();
    debug!("{:?} bytes sent", len);

    // Wait for the response and print it, which is a String type phone number.
    let mut buf = [0; 1024];
    let timeout_duration = Duration::from_secs(5); // Set the timeout duration to 5 seconds
    match timeout(timeout_duration, socket.recv_from(&mut buf)).await {
        Ok(Ok((len, or_addr))) => {
            // Successfully received data within the timeout
            println!("Received {} bytes from {:?}", len, or_addr);
            let num = String::from_utf8(buf[..len].to_vec()).expect("Invalid UTF-8");
            info!("Phone number for {}: {}", email, num);
        }
        Ok(Err(e)) => {
            // An error occurred during recv_from
            eprintln!("recv_from error: {:?}", e);
        }
        Err(_) => {
            // The operation timed out
            eprintln!("recv_from timed out");
        }
    }
}

fn get_emails_vec() -> Vec<String> {
    vec![
        "alice@domain.com".to_string(),
        "bob@domain.com".to_string(),
        "charlie@domain.com".to_string(),
        "david@domain.com".to_string(),
        "eve@domain.com".to_string(),
        "frank@domain.com".to_string(),
        "grace@domain.com".to_string(),
        "heidi@domain.com".to_string(),
        "ivan@domain.com".to_string(),
        "judy@domain.com".to_string(),
        "karl@domain.com".to_string(),
        "laura@domain.com".to_string(),
        "mallory@domain.com".to_string(),
        "nathan@domain.com".to_string(),
        "olivia@domain.com".to_string(),
        "peggy@domain.com".to_string(),
        "quinn@domain.com".to_string(),
        "rachel@domain.com".to_string(),
        "steve@domain.com".to_string(),
        "trudy@domain.com".to_string(),
        "ursula@domain.com".to_string(),
        "victor@domain.com".to_string(),
        "wendy@domain.com".to_string(),
        "xander@domain.com".to_string(),
        "yvonne@domain.com".to_string(),
        "zach@domain.com".to_string(),
        "amy@domain.com".to_string(),
        "brian@domain.com".to_string(),
        "carol@domain.com".to_string(),
        "dan@domain.com".to_string(),
        "ellen@domain.com".to_string(),
        "fred@domain.com".to_string(),
        "gina@domain.com".to_string(),
        "harry@domain.com".to_string(),
        "irene@domain.com".to_string(),
        "jack@domain.com".to_string(),
        "karen@domain.com".to_string(),
        "leo@domain.com".to_string(),
        "mona@domain.com".to_string(),
        "nick@domain.com".to_string(),
        "olga@domain.com".to_string(),
        "paul@domain.com".to_string(),
        "quincy@domain.com".to_string(),
        "rita@domain.com".to_string(),
        "sam@domain.com".to_string(),
        "tina@domain.com".to_string(),
        "uma@domain.com".to_string(),
        "vince@domain.com".to_string(),
        "wanda@domain.com".to_string(),
    ]
}
