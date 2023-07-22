#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use theseus::jre::autodetect_java_globals;
use theseus::prelude::*;

use theseus::profile_create::profile_create;
use tokio::time::{sleep, Duration};

// A simple Rust implementation of the authentication run
// 1) call the authenticate_begin_flow() function to get the URL to open (like you would in the frontend)
// 2) open the URL in a browser
// 3) call the authenticate_await_complete_flow() function to get the credentials (like you would in the frontend)
pub async fn authenticate_run() -> theseus::Result<Credentials> {
    println!("A browser window will now open, follow the login flow there.");
    let url = auth::authenticate_begin_flow().await?;

    println!("URL {}", url.as_str());
    webbrowser::open(url.as_str())
        .map_err(|e| IOError::with_path(e, url.as_str()))?;

    let credentials = auth::authenticate_await_complete_flow().await?;
    State::sync().await?;

    println!("Logged in user {}.", credentials.username);
    Ok(credentials)
}

#[tokio::main]
async fn main() -> theseus::Result<()> {
    println!("Starting.");

    let _log_guard = theseus::start_logger();

    // Initialize state
    let st = State::get().await?;
    //State::update();

    // Autodetect java globals
    let jres = jre::get_all_jre().await?;
    let java_8 = jre::find_filtered_jres("1.8", jres.clone(), false).await?;
    let java_17 = jre::find_filtered_jres("1.78", jres.clone(), false).await?;
    let java_18plus =
        jre::find_filtered_jres("1.18", jres.clone(), true).await?;
    let java_globals =
        autodetect_java_globals(java_8, java_17, java_18plus).await?;
    st.settings.write().await.java_globals = java_globals;

    st.settings.write().await.max_concurrent_downloads = 50;
    st.settings.write().await.hooks.post_exit =
        Some("echo This is after Minecraft runs- global setting!".to_string());
    // Changed the settings, so need to reset the semaphore
    st.reset_fetch_semaphore().await;

    //
    // st.settings
    //     .write()
    //     .await
    //     .java_globals
    //     .insert(JAVA_8_KEY.to_string(), check_jre(path).await?.unwrap());
    // Clear profiles
    println!("Clearing profiles.");
    {
        let h = profile::list(None).await?;
        for (path, _) in h.into_iter() {
            profile::remove(&path).await?;
        }
    }

    println!("Creating/adding profile.");

    let name = "Example".to_string();
    let game_version = "1.19.2".to_string();
    let modloader = ModLoader::Vanilla;
    let loader_version = "stable".to_string();

    let profile_path = profile_create(
        name.clone(),
        game_version,
        modloader,
        Some(loader_version),
        None,
        None,
        None,
        None,
    )
    .await?;

    State::sync().await?;

    // Attempt to run game
    if auth::users().await?.is_empty() {
        println!("No users found, authenticating.");
        authenticate_run().await?; // could take credentials from here direct, but also deposited in state users
    }

    println!("running");
    // Run a profile, running minecraft and store the RwLock to the process
    let proc_lock = profile::run(&profile_path).await?;
    let uuid = proc_lock.read().await.uuid;
    let pid = proc_lock.read().await.current_child.read().await.id();

    println!("Minecraft UUID: {}", uuid);
    println!("Minecraft PID: {:?}", pid);

    // Wait 5 seconds
    println!("Waiting 5 seconds to gather logs...");
    sleep(Duration::from_secs(5)).await;
    let stdout = process::get_output_by_uuid(&uuid).await?;
    println!("Logs after 5sec <<< {stdout} >>> end stdout");

    println!(
        "All running process UUID {:?}",
        process::get_all_running_uuids().await?
    );
    println!(
        "All running process paths {:?}",
        process::get_all_running_profile_paths().await?
    );

    // hold the lock to the process until it ends
    println!("Waiting for process to end...");
    let mut proc = proc_lock.write().await;
    process::wait_for(&mut proc).await?;

    Ok(())
}
