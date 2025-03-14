#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::time::Duration;
use theseus::prelude::*;
use theseus::worlds::{get_profile_worlds, get_server_status};
use tokio::signal::ctrl_c;

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

    // let state = State::get().await?;
    // let instance_path = state.directories
    //     .profiles_dir()
    //     .join("Logging Test")
    //     .display()
    //     .to_string();
    // for world in get_profile_worlds(&instance_path).await? {
    //     tracing::info!("{}", serde_json::to_string_pretty(&world)?);
    // }

    let ping_result = get_server_status("hypixel.net").await?;
    tracing::info!("{}", serde_json::to_string_pretty(&ping_result)?);

    Ok(())
}
