#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::path::PathBuf;

use theseus::pack::import::ImportLauncherType;
use theseus::pack::install_from::CreatePackProfile;
use theseus::prelude::*;

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
    let state = crate::State::get().await?;

    // Set java globals to auto detected ones
    {
        let jres = crate::jre::get_all_jre().await?;
        let java_8 =
            crate::jre::find_filtered_jres("1.8", jres.clone(), false).await?;
        let java_17 =
            crate::jre::find_filtered_jres("1.17", jres.clone(), false).await?;
        let java_18plus =
            crate::jre::find_filtered_jres("1.18", jres.clone(), true).await?;
        let java_globals =
            crate::jre::autodetect_java_globals(java_8, java_17, java_18plus)
                .await?;
        state.settings.write().await.java_globals = java_globals;
    }
    const ATLAUNCHER_FOLDER: &str = r"/home/thesuzerain/ATLauncher";
    const PRISM_FOLDER: &str = r"/home/thesuzerain/.local/share/PrismLauncher";
    const MMC_FOLDER: &str = r"/home/thesuzerain/MultiMC";
    const CURSEFORGE_FOLDER: &str = r"/home/thesuzerain/curseforge/minecraft";
    const GD_LAUNCHER_FOLDER: &str = r"/home/thesuzerain/gdlauncher_next";

    test_batch_import(ATLAUNCHER_FOLDER, ImportLauncherType::ATLauncher)
        .await?;
    test_batch_import(PRISM_FOLDER, ImportLauncherType::PrismLauncher).await?;
    test_batch_import(MMC_FOLDER, ImportLauncherType::MultiMC).await?;
    test_batch_import(CURSEFORGE_FOLDER, ImportLauncherType::Curseforge)
        .await?;
    test_batch_import(GD_LAUNCHER_FOLDER, ImportLauncherType::GDLauncher)
        .await?;

    println!("Done all!");
    Ok(())
}

async fn test_batch_import(
    folder: &str,
    r#type: ImportLauncherType,
) -> theseus::Result<()> {
    let instances =
        pack::import::get_importable_instances(r#type, folder.into()).await?;
    for instance in instances {
        println!("\n\n\nImporting {} for {:?}", instance, r#type);
        let profile_path = profile_create::profile_create_from_creator(
            CreatePackProfile::default(),
        )
        .await
        .unwrap();

        pack::import::import_instance(
            profile_path,
            r#type,
            PathBuf::from(folder),
            instance,
            None,
        )
        .await?;
        println!("Completoooo");
    }
    println!("Done batch import.");

    Ok(())
}
