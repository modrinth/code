#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use theseus::prelude::*;

use theseus::profile::create::profile_create;

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

    if minecraft_auth::users().await?.is_empty() {
        println!("No users found, authenticating.");
        authenticate_run().await?; // could take credentials from here direct, but also deposited in state users
    }
    //
    // st.settings
    //     .write()
    //     .await
    //     .java_globals
    //     .insert(JAVA_8_KEY.to_string(), check_jre(path).await?.unwrap());
    // Clear profiles
    println!("Clearing profiles.");
    {
        let h = profile::list().await?;
        for profile in h.into_iter() {
            profile::remove(&profile.path).await?;
        }
    }

    println!("Creating/adding profile.");

    let name = "Example".to_string();
    let game_version = "1.16.1".to_string();
    let modloader = ModLoader::Forge;
    let loader_version = "stable".to_string();

    let profile_path = profile_create(
        name,
        game_version,
        modloader,
        Some(loader_version),
        None,
        None,
        None,
    )
    .await?;

    println!("running");
    // Run a profile, running minecraft and store the RwLock to the process
    let process = profile::run(&profile_path).await?;

    println!("Minecraft UUID: {}", process.uuid);

    println!("All running process UUID {:?}", process::get_all().await?);

    // hold the lock to the process until it ends
    println!("Waiting for process to end...");
    process::wait_for(process.uuid).await?;

    Ok(())
}
