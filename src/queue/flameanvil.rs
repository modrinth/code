use crate::database::models::categories::GameVersion;
use crate::file_hosting::FileHostingError;
use crate::routes::project_creation::CreateError;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone)]
struct FlameGameVersionType {
    id: i32,
    slug: String,
}

#[derive(Deserialize, Debug, Clone)]
struct FlameGameVersion {
    id: i32,
    #[serde(rename = "gameVersionTypeID")]
    game_version_type_id: i32,
    name: String,
    slug: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct FlameUploadFile {
    changelog: String,
    // always "markdown"
    changelog_type: String,
    display_name: String,
    game_versions: Vec<i32>,
    release_type: String,
    // TODO: relations?
}

pub struct FlameAnvilQueue {
    mod_loaders: Vec<FlameGameVersion>,
    minecraft_versions: Vec<FlameGameVersion>,
    last_updated: DateTime<Utc>,
}

pub struct UploadFile {
    pub loaders: Vec<String>,
    pub game_versions: Vec<String>,
    pub display_name: String,
    pub changelog: String,
    pub version_type: String,
}

// Batches download transactions every thirty seconds
impl FlameAnvilQueue {
    pub fn new() -> Self {
        FlameAnvilQueue {
            mod_loaders: vec![],
            minecraft_versions: vec![],
            last_updated: Utc::now() - Duration::days(365),
        }
    }

    pub fn convert_game_versions_to_flame(
        &self,
        original: Vec<String>,
        game_versions: &[GameVersion],
    ) -> Vec<i32> {
        let mut og_to_flame = HashMap::new();
        let mut last_visited = if self
            .minecraft_versions
            .last()
            .map(|x| x.name.ends_with("-Snapshot"))
            .unwrap_or_default()
        {
            None
        } else {
            self.minecraft_versions
                .iter()
                .rfind(|x| !x.name.ends_with("-Snapshot"))
                .cloned()
        };

        for game_version in game_versions {
            if let Some(flame_game_version) =
                self.minecraft_versions.iter().find(|x| {
                    x.name
                        == if game_version.version.starts_with('b') {
                            game_version.version.replace('b', "Beta ")
                        } else {
                            game_version.version.clone()
                        }
                })
            {
                last_visited = Some(flame_game_version.clone());
                og_to_flame
                    .insert(&game_version.version, flame_game_version.id);
            } else if let Some(last_visited) = &last_visited {
                if game_version.major {
                    og_to_flame.insert(&game_version.version, last_visited.id);
                } else {
                    let mut splits = last_visited.name.split('.');
                    let new_str = format!(
                        "{}.{}-Snapshot",
                        splits.next().unwrap_or_default(),
                        splits.next().unwrap_or_default()
                    );

                    if let Some(flame_game_version) = self
                        .minecraft_versions
                        .iter()
                        .find(|x| x.name == new_str)
                    {
                        og_to_flame.insert(
                            &game_version.version,
                            flame_game_version.id,
                        );
                    } else {
                        og_to_flame
                            .insert(&game_version.version, last_visited.id);
                    }
                }
            } else if let Some(first) = self.minecraft_versions.last() {
                og_to_flame.insert(&game_version.version, first.id);
            }
        }

        let mut new = Vec::new();

        for x in original {
            if let Some(value) = og_to_flame.get(&&x) {
                new.push(*value);
            }
        }

        new
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn upload_file(
        &mut self,
        api_token: &str,
        project_id: i32,
        upload_file: UploadFile,
        game_versions: &[GameVersion],
        file: Vec<u8>,
        file_name: String,
        mime_type: String,
    ) -> Result<i32, CreateError> {
        if self.last_updated < (Utc::now() - Duration::minutes(30)) {
            self.index(api_token).await.map_err(|_| {
                CreateError::InvalidInput(
                    "Indexing metadata from FlameAnvil failed!".to_string(),
                )
            })?;
        }

        let mut loaders_converted = upload_file
            .loaders
            .into_iter()
            .filter_map(|x| self.mod_loaders.iter().find(|y| y.slug == x))
            .map(|x| x.id)
            .collect::<Vec<i32>>();

        let mut game_versions_converted = self.convert_game_versions_to_flame(
            upload_file.game_versions,
            game_versions,
        );

        loaders_converted.append(&mut game_versions_converted);

        let file = reqwest::multipart::Part::bytes(file)
            .file_name(file_name)
            .mime_str(&mime_type)
            .map_err(|_| {
                CreateError::InvalidInput(
                    "Error while converting inputted file to multipart payload"
                        .to_string(),
                )
            })?;

        let form = reqwest::multipart::Form::new()
            .text(
                "metadata",
                serde_json::to_string(&FlameUploadFile {
                    changelog: upload_file.changelog,
                    changelog_type: "markdown".to_string(),
                    display_name: upload_file.display_name,
                    game_versions: loaders_converted,
                    release_type: upload_file.version_type,
                })
                .unwrap(),
            )
            .part("file", file);

        #[derive(Deserialize)]
        struct FileResponse {
            id: i32,
        }

        let client = reqwest::Client::new();

        let id = client.post(&*format!("https://minecraft.curseforge.com/api/projects/{project_id}/upload-file?token={api_token}"))
            .multipart(form)
            .send()
            .await.map_err(|_| CreateError::FileHostingError(FileHostingError::S3Error("Error uploading file to FlameAnvil!".to_string())))?
            .json::<FileResponse>()
            .await.map_err(|_| CreateError::FileHostingError(FileHostingError::S3Error("Error deserializing uploaded file response from FlameAnvil!".to_string())))?;

        Ok(id.id)
    }

    pub async fn index(
        &mut self,
        api_token: &str,
    ) -> Result<(), reqwest::Error> {
        let (game_versions, game_version_types) = futures::future::try_join(
            reqwest::get(format!("https://minecraft.curseforge.com/api/game/versions?token={api_token}")),
            reqwest::get(format!("https://minecraft.curseforge.com/api/game/version-types?token={api_token}"))
        ).await?;

        let (game_versions, game_version_types) = futures::future::try_join(
            game_versions.json::<Vec<FlameGameVersion>>(),
            game_version_types.json::<Vec<FlameGameVersionType>>(),
        )
        .await?;

        let mod_loader_types = game_version_types
            .iter()
            .filter(|x| x.slug == *"modloader")
            .map(|x| x.id)
            .collect::<Vec<_>>();
        let minecraft_types = game_version_types
            .iter()
            .filter(|x| x.slug.starts_with("minecraft"))
            .map(|x| x.id)
            .collect::<Vec<_>>();

        let mod_loaders = game_versions
            .iter()
            .filter(|x| mod_loader_types.contains(&x.game_version_type_id))
            .cloned()
            .collect::<Vec<_>>();
        let minecraft_versions = game_versions
            .iter()
            .filter(|x| minecraft_types.contains(&x.game_version_type_id))
            .cloned()
            .collect::<Vec<_>>();

        self.mod_loaders = mod_loaders;
        self.minecraft_versions = minecraft_versions;

        Ok(())
    }
}
