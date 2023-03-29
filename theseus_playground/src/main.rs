#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{path::{Path, PathBuf}, collections::HashSet};
use theseus::prelude::*;
use tokio::sync::oneshot;

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

    webbrowser::open(url.as_str())?;
    let credentials = flow.await.unwrap()?;
    State::sync().await?;
    println!("Logged in user {}.", credentials.username);
    Ok(credentials)
}

#[tokio::main]
async fn main() -> theseus::Result<()> {
    // Initialize state
    let _ = State::get().await?;

    // Clear profiles
    let h = profile::list().await?;
    for (path, _) in h.into_iter() {
        profile::remove(&path).await?;
    }

    // Create vanilla minecraft instance
    let path = Path::new("../minecraft");

    let profile = Profile {
        path: path.to_path_buf(),
        metadata: ProfileMetadata {
            name: String::from("Example Pack"),
            icon: None,
            game_version: String::from("1.18.2"),
            loader: ModLoader::Vanilla,
            loader_version: None,
            format_version: 1,
        },
        java: Some(JavaSettings {
            install: Some(PathBuf::from("/usr/bin/java")),
            extra_arguments: Some(Vec::new()),
        }),
        memory: Some(MemorySettings {
            minimum: None,
            maximum: 8192,
        }),
        resolution: Some(WindowSize(1920, 1080)),
        hooks: Some(Hooks {
            pre_launch: HashSet::new(),
            wrapper: None,
            post_exit: HashSet::new(),
        }),
    };
    profile::add(profile).await?;


    // Attempt to create credentials.
    let credentials = authenticate_run().await?;
    // Attempt to load credentials. Use if ^ is giving rate limit.
    // let users =  users().await.unwrap();
    // let credentials =  users.first().unwrap();

    // Run MC
    profile::run(&path.canonicalize()?, &credentials).await?;
    Ok(())
}
