#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use dunce::canonicalize;
use theseus::prelude::*;
use tokio::time::{sleep, Duration};

// A simple Rust implementation of the authentication run
// 1) call the authenticate_begin_flow() function to get the URL to open (like you would in the frontend)
// 2) open the URL in a browser
// 3) call the authenticate_await_complete_flow() function to get the credentials (like you would in the frontend)
pub async fn authenticate_run() -> theseus::Result<Credentials> {
    println!("A browser window will now open, follow the login flow there.");
    let url = auth::authenticate_begin_flow().await?;

    println!("URL {}", url.as_str());
    webbrowser::open(url.as_str())?;

    let credentials = auth::authenticate_await_complete_flow().await?;
    State::sync().await?;

    println!("Logged in user {}.", credentials.username);
    Ok(credentials)
}

#[tokio::main]
async fn main() -> theseus::Result<()> {
    println!("Starting.");

    // Initialize state
    let st = State::get().await?;
    st.settings.write().await.max_concurrent_downloads = 1;

    // Clear profiles
    println!("Clearing profiles.");
    {
        let h = profile::list().await?;
        for (path, _) in h.into_iter() {
            profile::remove(&path).await?;
        }
    }

    println!("Creating/adding profile.");

    let profile_path =
        pack::install_pack_from_version_id("KxUUUFh5".to_string())
            .await
            .unwrap();

    //  async closure for testing any desired edits
    // (ie: changing the java runtime of an added profile)
    // println!("Editing.");
    profile::edit(&profile_path, |_profile| {
        // Eg: Java- this would let you change the java runtime of the profile instead of using the default
        // use theseus::prelude::jre::JAVA__KEY;
        // profile.java = Some(JavaSettings {
        // jre_key: Some(JAVA_17_KEY.to_string()),
        //     extra_arguments: None,
        // });
        async { Ok(()) }
    })
    .await?;
    State::sync().await?;

    // Attempt to get the default user, if it exists, and refresh their credentials
    let default_user_uuid = {
        let settings = st.settings.read().await;
        settings.default_user
    };
    let credentials = if let Some(uuid) = default_user_uuid {
        println!("Attempting to refresh existing authentication.");
        auth::refresh(uuid, false).await
    } else {
        println!("Freshly authenticating.");
        authenticate_run().await
    };

    // Check attempt to get Credentials
    // If successful, run the profile and store the RwLock to the process
    let proc_lock = match credentials {
        Ok(credentials) => {
            println!("Preparing to run Minecraft.");
            profile::run(&canonicalize(&profile_path)?, &credentials).await
        }
        Err(e) => {
            // If Hydra could not be accessed, for testing, attempt to load credentials from disk and do the same
            println!("Could not authenticate: {}.\nAttempting stored authentication.",e);
            let users = auth::users().await.expect(
                "Could not access any stored users- state was dropped.",
            );
            let credentials = users
                .first()
                .expect("Hydra failed, and no stored users were found.");
            println!("Preparing to run Minecraft.");
            profile::run(&canonicalize(&profile_path)?, credentials).await
        }
    }?;

    let pid = proc_lock
        .read()
        .await
        .child
        .id()
        .expect("Could not get PID from process.");
    println!("Minecraft PID: {}", pid);

    // Wait 5 seconds
    println!("Waiting 20 seconds to gather logs...");
    sleep(Duration::from_secs(20)).await;
    let stdout = process::get_stdout_by_pid(pid).await?;
    println!("Logs after 5sec <<< {stdout} >>> end stdout");

    println!(
        "All running process PIDs {:?}",
        process::get_all_running_pids().await?
    );
    println!(
        "All running process paths {:?}",
        process::get_all_running_profile_paths().await?
    );
    println!(
        "All running process profiles {:?}",
        process::get_all_running_profiles().await?
    );

    // hold the lock to the process until it ends
    println!("Waiting for process to end...");
    let mut proc = proc_lock.write().await;
    process::wait_for(&mut proc).await?;

    Ok(())
}
