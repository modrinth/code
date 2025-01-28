#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::env::args;
use std::net::SocketAddr;
use theseus::prelude::*;
use tokio::net::TcpListener;
use tokio::signal::ctrl_c;
use uuid::Uuid;

// A simple Rust implementation of the authentication run
// 1) call the authenticate_begin_flow() function to get the URL to open (like you would in the frontend)
// 2) open the URL in a browser
// 3) call the authenticate_await_complete_flow() function to get the credentials (like you would in the frontend)
pub async fn authenticate_run() -> theseus::Result<Credentials> {
    println!("A browser window will now open, follow the login flow there.");
    let login = minecraft_auth::begin_login().await?;

    println!("URL {}", login.redirect_uri.as_str());
    webbrowser::open(login.redirect_uri.as_str())?;

    println!("Please enter URL code: ");
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("error: unable to read user input");

    println!("You entered: {}", input.trim());

    let credentials = minecraft_auth::finish_login(&input, login).await?;

    println!("Logged in user {}.", credentials.username);
    Ok(credentials)
}

#[tokio::main]
async fn main() -> theseus::Result<()> {
    println!("Starting.");

    let _log_guard = theseus::start_logger();

    // Initialize state
    State::init().await?;

    // if minecraft_auth::users().await?.is_empty() {
    //     println!("No users found, authenticating.");
    //     authenticate_run().await?; // could take credentials from here direct, but also deposited in state users
    // }

    match args().nth(1).as_deref() {
        Some("host") => main_host().await?,
        Some("client") => main_client().await?,
        Some(other) => tracing::error!(
            "'host' or 'client' expected as first CLI arg, but found '{other}'"
        ),
        None => tracing::error!("Expected first CLI arg 'host' or 'client'"),
    }

    Ok(())
}

async fn main_host() -> theseus::Result<()> {
    tracing::info!("Starting host");

    let socket = State::get().await?.friends_socket.open_port(25565).await?;
    tracing::info!("Running host on socket {}", socket.socket_id());

    ctrl_c().await?;
    tracing::info!("Stopping host");
    socket.shutdown().await?;

    Ok(())
}

async fn main_client() -> theseus::Result<()> {
    tracing::info!("Starting client");

    let socket_id = args()
        .nth(2)
        .expect("Expected second CLI arg to be socket ID")
        .parse::<Uuid>()?;

    tracing::info!("Listening on port 25585 to connect to {socket_id}");
    let tcp_stream =
        TcpListener::bind(SocketAddr::new("127.0.0.1".parse().unwrap(), 25585))
            .await?
            .accept()
            .await?
            .0;
    tracing::info!("Connecting to {socket_id}");
    let socket = State::get()
        .await?
        .friends_socket
        .connect_to_socket(socket_id, tcp_stream)
        .await?;

    ctrl_c().await?;
    tracing::info!("Stopping client");
    socket.shutdown().await?;

    Ok(())
}
