#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use theseus::prelude::*;
use theseus::profile::create::profile_create;
use anyhow::Context; // Better error context

// A simple Rust implementation of the authentication run
// 1) Call authenticate_begin_flow() function to get the URL to open (like you would in the frontend)
// 2) Open the URL in a browser
// 3) Call authenticate_await_complete_flow() function to get the credentials (like you would in the frontend)
pub async fn authenticate_run() -> theseus::Result<Credentials> {
    println!("Starting authentication process. A browser window will now open. Follow the login flow.");

    let login = minecraft_auth::begin_login().await.context("Failed to begin login")?;
    println!("Authentication URL: {}", login.redirect_uri.as_str());

    webbrowser::open(login.redirect_uri.as_str()).context("Failed to open browser")?;

    // Improved input handling
    println!("Please enter the URL code: ");
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .context("Error reading user input")?;

    let input_trimmed = input.trim();
    println!("You entered: {}", input_trimmed);

    // Finish login process
    let credentials = minecraft_auth::finish_login(input_trimmed, login)
        .await
        .context("Failed to complete login")?;

    println!("Successfully logged in as user: {}.", credentials.username);
    Ok(credentials)
}

// Refactored profile clearing into a separate function for modularity
async fn clear_profiles() -> theseus::Result<()> {
    println!("Clearing profiles.");
    let profiles = profile::list().await.context("Failed to list profiles")?;

    // Use async parallel execution for efficiency
    let mut tasks = Vec::with_capacity(profiles.len());
    for profile in profiles {
        tasks.push(tokio::spawn(async move {
            profile::remove(&profile.path).await.context("Failed to remove profile")
        }));
    }
    for task in tasks {
        task.await??;
    }

    println!("Profiles cleared.");
    Ok(())
}

pub async fn create_profile() -> theseus::Result<String> {
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
    .await
    .context("Failed to create profile")?;

    println!("Profile created at: {}", profile_path);
    Ok(profile_path)
}

pub async fn run_profile(profile_path: String) -> theseus::Result<()> {
    println!("Running the profile...");

    // Run the profile, launching Minecraft and store the RwLock to the process
    let process = profile::run(&profile_path)
        .await
        .context("Failed to run profile")?;

    println!("Minecraft UUID: {}", process.uuid);

    let all_processes = process::get_all().await?;
    println!("All running process UUIDs: {:?}", all_processes);

    // Wait for the Minecraft process to end
    println!("Waiting for Minecraft process to end...");
    process::wait_for(process.uuid).await.context("Error while waiting for process to end")?;

    Ok(())
}

#[tokio::main]
async fn main() -> theseus::Result<()> {
    println!("Starting application.");

    let _log_guard = theseus::start_logger();

    // Initialize global state
    State::init().await.context("Failed to initialize state")?;

    if minecraft_auth::users().await?.is_empty() {
        println!("No users found, initiating authentication.");
        authenticate_run().await.context("Authentication failed")?;
    }

    // Clear profiles
    clear_profiles().await?;

    // Create a new profile
    let profile_path = create_profile().await?;

    // Run the newly created profile
    run_profile(profile_path).await?;

    println!("Application finished.");
    Ok(())
}
