use crate::event::emit::{
    emit_loading, init_or_edit_loading, loading_try_for_each_concurrent,
};
use crate::event::LoadingBarType;
use crate::pack::install_from::{
    set_profile_information, EnvType, PackFile, PackFileHash,
};
use crate::prelude::ProfilePathId;
use crate::state::SideType;
use crate::util::fetch::{fetch_mirrors, write};
use crate::State;
use async_zip::tokio::read::seek::ZipFileReader;

use std::io::Cursor;
use std::path::{Component, PathBuf};

use super::install_from::{
    generate_pack_from_file, generate_pack_from_version_id, CreatePack,
    CreatePackLocation, PackFormat,
};

/// Install a modpack from a mrpack file (a modrinth .zip format)
#[theseus_macros::debug_pin]
pub async fn install_zipped_mrpack(
    location: CreatePackLocation,
    profile: ProfilePathId,
) -> crate::Result<ProfilePathId> {
    // Get file from description
    let create_pack: CreatePack = match location {
        CreatePackLocation::FromVersionId {
            project_id,
            version_id,
            title,
            icon_url,
        } => {
            generate_pack_from_version_id(
                project_id, version_id, title, icon_url, profile,
            )
            .await?
        }
        CreatePackLocation::FromFile { path } => {
            generate_pack_from_file(path, profile).await?
        }
    };

    let file = create_pack.file;
    let description = create_pack.description.clone(); // make a copy for profile edit function
    let icon = create_pack.description.icon;
    let project_id = create_pack.description.project_id;
    let version_id = create_pack.description.version_id;
    let existing_loading_bar = create_pack.description.existing_loading_bar;
    let profile = create_pack.description.profile_path;

    let state = &State::get().await?;

    let result = async {
        let reader: Cursor<&bytes::Bytes> = Cursor::new(&file);

        // Create zip reader around file
        let mut zip_reader =
            ZipFileReader::new(reader).await.map_err(|_| {
                crate::Error::from(crate::ErrorKind::InputError(
                    "Failed to read input modpack zip".to_string(),
                ))
            })?;

        // Extract index of modrinth.index.json
        let zip_index_option = zip_reader
            .file()
            .entries()
            .iter()
            .position(|f| f.entry().filename() == "modrinth.index.json");
        if let Some(zip_index) = zip_index_option {
            let mut manifest = String::new();
            let entry = zip_reader
                .file()
                .entries()
                .get(zip_index)
                .unwrap()
                .entry()
                .clone();
            let mut reader = zip_reader.entry(zip_index).await?;
            reader.read_to_string_checked(&mut manifest, &entry).await?;

            let pack: PackFormat = serde_json::from_str(&manifest)?;

            if &*pack.game != "minecraft" {
                return Err(crate::ErrorKind::InputError(
                    "Pack does not support Minecraft".to_string(),
                )
                .into());
            }

            // Sets generated profile attributes to the pack ones (using profile::edit)
            set_profile_information(
                profile.clone(),
                &description,
                &pack.name,
                &pack.dependencies,
            )
            .await?;

            let profile_full_path = profile.get_full_path().await?;
            let profile = profile.clone();
            let result = async {
                let loading_bar = init_or_edit_loading(
                    existing_loading_bar,
                    LoadingBarType::PackDownload {
                        profile_path: profile_full_path.clone(),
                        pack_name: pack.name.clone(),
                        icon,
                        pack_id: project_id,
                        pack_version: version_id,
                    },
                    100.0,
                    "Downloading modpack",
                )
                .await?;

                let num_files = pack.files.len();
                use futures::StreamExt;
                loading_try_for_each_concurrent(
                    futures::stream::iter(pack.files.into_iter())
                        .map(Ok::<PackFile, crate::Error>),
                    None,
                    Some(&loading_bar),
                    70.0,
                    num_files,
                    None,
                    |project| {
                        let profile_full_path = profile_full_path.clone();
                        async move {
                            //TODO: Future update: prompt user for optional files in a modpack
                            if let Some(env) = project.env {
                                if env
                                    .get(&EnvType::Client)
                                    .map(|x| x == &SideType::Unsupported)
                                    .unwrap_or(false)
                                {
                                    return Ok(());
                                }
                            }

                            let file = fetch_mirrors(
                                &project
                                    .downloads
                                    .iter()
                                    .map(|x| &**x)
                                    .collect::<Vec<&str>>(),
                                project
                                    .hashes
                                    .get(&PackFileHash::Sha1)
                                    .map(|x| &**x),
                                &state.fetch_semaphore,
                            )
                            .await?;

                            let path = std::path::Path::new(&project.path)
                                .components()
                                .next();
                            if let Some(path) = path {
                                match path {
                                    Component::CurDir
                                    | Component::Normal(_) => {
                                        let path = profile_full_path
                                            .join(project.path);
                                        write(
                                            &path,
                                            &file,
                                            &state.io_semaphore,
                                        )
                                        .await?;
                                    }
                                    _ => {}
                                };
                            }
                            Ok(())
                        }
                    },
                )
                .await?;

                emit_loading(&loading_bar, 0.0, Some("Extracting overrides"))
                    .await?;

                let mut total_len = 0;

                for index in 0..zip_reader.file().entries().len() {
                    let file =
                        zip_reader.file().entries().get(index).unwrap().entry();

                    if (file.filename().starts_with("overrides")
                        || file.filename().starts_with("client_overrides"))
                        && !file.filename().ends_with('/')
                    {
                        total_len += 1;
                    }
                }

                for index in 0..zip_reader.file().entries().len() {
                    let file = zip_reader
                        .file()
                        .entries()
                        .get(index)
                        .unwrap()
                        .entry()
                        .clone();

                    let file_path = PathBuf::from(file.filename());
                    if (file.filename().starts_with("overrides")
                        || file.filename().starts_with("client_overrides"))
                        && !file.filename().ends_with('/')
                    {
                        // Reads the file into the 'content' variable
                        let mut content = Vec::new();
                        let mut reader = zip_reader.entry(index).await?;
                        reader.read_to_end_checked(&mut content, &file).await?;

                        let mut new_path = PathBuf::new();
                        let components = file_path.components().skip(1);

                        for component in components {
                            new_path.push(component);
                        }

                        if new_path.file_name().is_some() {
                            write(
                                &profile_full_path.join(new_path),
                                &content,
                                &state.io_semaphore,
                            )
                            .await?;
                        }

                        emit_loading(
                            &loading_bar,
                            30.0 / total_len as f64,
                            Some(&format!(
                                "Extracting override {}/{}",
                                index, total_len
                            )),
                        )
                        .await?;
                    }
                }

                if let Some(profile_val) =
                    crate::api::profile::get(&profile, None).await?
                {
                    crate::launcher::install_minecraft(
                        &profile_val,
                        Some(loading_bar),
                    )
                    .await?;

                    State::sync().await?;
                }

                Ok::<ProfilePathId, crate::Error>(profile.clone())
            }
            .await;

            match result {
                Ok(profile) => Ok(profile),
                Err(err) => {
                    let _ = crate::api::profile::remove(&profile).await;

                    Err(err)
                }
            }
        } else {
            Err(crate::Error::from(crate::ErrorKind::InputError(
                "No pack manifest found in mrpack".to_string(),
            )))
        }
    }
    .await;

    match result {
        Ok(profile) => Ok(profile),
        Err(err) => {
            let _ = crate::api::profile::remove(&profile).await;

            Err(err)
        }
    }
}
