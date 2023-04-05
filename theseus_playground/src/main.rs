#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use dunce::canonicalize;
use std::path::Path;
use theseus::prelude::*;
use tokio::process::Child;
use tokio::sync::{oneshot, RwLockWriteGuard};

// We use this function directly to call authentication procedure
// Note: "let url = match url" logic is handled differently, so that if there is a rate limit in the other set causing that one to end early,
// we can see the error message in this thread rather than a Recv error on 'rx' when the receiver is mysteriously droppped
pub async fn authenticate_run() -> theseus::Result<Credentials> {
    println!("Adding new user account to Theseus");
    println!("A browser window will now open, follow the login flow there.");

    let (tx, rx) = oneshot::channel::<url::Url>();
    let flow = tokio::spawn(auth::authenticate(tx));

    let url = rx.await;
    let url = match url {
        Ok(url) => url,
        Err(e) => {
            flow.await.unwrap()?;
            return Err(e.into());
        }
    };
    println!("URL {}", url.as_str());
    webbrowser::open(url.as_str())?;
    let credentials = flow.await.unwrap()?;
    State::sync().await?;
    println!("Logged in user {}.", credentials.username);
    Ok(credentials)
}

#[tokio::main]
async fn main() -> theseus::Result<()> {
    println!("Starting.");

    // Initialize state
    {
        let st = State::get().await?;
        st.settings.write().await.max_concurrent_downloads = 100;
    }

    // Set max concurrent downloads to 10

    // Example variables for simple project case
    let name = "Example".to_string();
    let game_version = "1.19.2".to_string();
    let modloader = ModLoader::Fabric;
    let loader_version = "stable".to_string();

    // let icon = Some(
    //     Path::new("../icon_test.png")
    //         .canonicalize()
    //         .expect("Icon could be not be found. If not using, set to None"),
    // );
    //let icon = None;

    // Clear profiles
    println!("Clearing profiles.");
    {
        let h = profile::list().await?;
        for (path, _) in h.into_iter() {
            profile::remove(&path).await?;
        }
    }

    println!("Creating/adding profile.");
    // Attempt to create a profile. If that fails, try adding one from the same path.
    // TODO: actually do a nested error check for the correct profile error.
    // let profile_path = profile_create(
    //     name.clone(),
    //     game_version,
    //     modloader,
    //     Some(loader_version),
    //     icon,
    // )
    // .await?;
    let profile_path =
        pack::install_pack_from_version_id("hZ6f9Ij9".to_string())
            .await
            .unwrap();

    // let profiles = profile::list().await?;
    // let profile_path = profiles.keys().next().unwrap();
    State::sync().await?;

    //  async closure for testing any desired edits
    // (ie: changing the java runtime of an added profile)
    println!("Editing.");
    profile::edit(&profile_path, |profile| {
        // Eg: Java. TODO: hook up to jre.rs class to pick optimal java
        profile.java = Some(JavaSettings {
            install: Some(Path::new("/usr/bin/java").to_path_buf()),
            extra_arguments: None,
        });
        async { Ok(()) }
    })
    .await?;
    State::sync().await?;

    println!("Authenticating.");
    // Attempt to create credentials and run

    let users = auth::users().await.unwrap();
    let proc_lock = if let Some(credentials) = users.first() {
        println!("Running.");
        profile::run(&canonicalize(&profile_path)?, credentials).await
    } else {
        println!("No stored profile. Authenticating normally.",);
        let credentials = authenticate_run().await.unwrap();
        println!("Running.");
        profile::run(&canonicalize(&profile_path)?, &credentials).await
    }
    .unwrap();

    println!("Started. Waiting...");
    let mut proc: RwLockWriteGuard<Child> = proc_lock.write().await;
    profile::wait_for(&mut proc).await?;

    // Run MC
    Ok(())
}
