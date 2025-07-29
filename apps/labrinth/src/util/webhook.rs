use crate::database::models::legacy_loader_fields::MinecraftGameVersion;
use crate::database::redis::RedisPool;
use crate::models::ids::ProjectId;
use crate::routes::ApiError;
use ariadne::ids::base62_impl::to_base62;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

const PLUGIN_LOADERS: &[&str] = &[
    "bukkit",
    "spigot",
    "paper",
    "purpur",
    "bungeecord",
    "waterfall",
    "velocity",
    "sponge",
];

struct WebhookMetadata {
    pub project_url: String,
    pub project_title: String,
    pub project_summary: String,
    pub display_project_type: String,
    pub project_icon_url: Option<String>,
    pub color: Option<u32>,

    pub author: Option<WebhookAuthor>,

    pub categories_formatted: Vec<String>,
    pub loaders_formatted: Vec<String>,
    pub versions_formatted: Vec<String>,

    pub gallery_image: Option<String>,
}

struct WebhookAuthor {
    pub name: String,
    pub url: String,
    pub icon_url: Option<String>,
}

async fn get_webhook_metadata(
    project_id: ProjectId,
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<Option<WebhookMetadata>, ApiError> {
    let project = crate::database::models::project_item::DBProject::get_id(
        project_id.into(),
        pool,
        redis,
    )
    .await?;

    if let Some(mut project) = project {
        let mut owner = None;

        if let Some(organization_id) = project.inner.organization_id {
            let organization = crate::database::models::organization_item::DBOrganization::get_id(
                organization_id,
                pool,
                redis,
            )
            .await?;

            if let Some(organization) = organization {
                owner = Some(WebhookAuthor {
                    name: organization.name,
                    url: format!(
                        "{}/organization/{}",
                        dotenvy::var("SITE_URL").unwrap_or_default(),
                        to_base62(organization.id.0 as u64)
                    ),
                    icon_url: organization.icon_url,
                });
            }
        } else {
            let team = crate::database::models::team_item::DBTeamMember::get_from_team_full(
                project.inner.team_id,
                pool,
                redis,
            )
            .await?;

            if let Some(member) = team.into_iter().find(|x| x.is_owner) {
                let user = crate::database::models::user_item::DBUser::get_id(
                    member.user_id,
                    pool,
                    redis,
                )
                .await?;

                if let Some(user) = user {
                    owner = Some(WebhookAuthor {
                        url: format!(
                            "{}/user/{}",
                            dotenvy::var("SITE_URL").unwrap_or_default(),
                            to_base62(user.id.0 as u64)
                        ),
                        name: user.username,
                        icon_url: user.avatar_url,
                    });
                }
            }
        };

        let all_game_versions =
            MinecraftGameVersion::list(None, None, pool, redis).await?;

        let versions = project
            .aggregate_version_fields
            .clone()
            .into_iter()
            .find_map(|vf| {
                MinecraftGameVersion::try_from_version_field(&vf).ok()
            })
            .unwrap_or_default();

        let formatted_game_versions = get_gv_range(versions, all_game_versions);

        let mut project_type = project.project_types.pop().unwrap_or_default(); // TODO: Should this grab a not-first?

        if project
            .inner
            .loaders
            .iter()
            .all(|x| PLUGIN_LOADERS.contains(&&**x))
        {
            project_type = "plugin".to_string();
        } else if project.inner.loaders.iter().any(|x| x == "datapack") {
            project_type = "datapack".to_string();
        }

        let mut display_project_type = match &*project_type {
            "datapack" => "data pack",
            "resourcepack" => "resource pack",
            _ => &*project_type,
        }
        .to_string();

        Ok(Some(WebhookMetadata {
            project_url: format!(
                "{}/{}/{}",
                dotenvy::var("SITE_URL").unwrap_or_default(),
                project_type,
                to_base62(project.inner.id.0 as u64)
            ),
            project_title: project.inner.name,
            project_summary: project.inner.summary,
            display_project_type: format!(
                "{}{display_project_type}",
                display_project_type.remove(0).to_uppercase()
            ),
            project_icon_url: project.inner.icon_url,
            color: project.inner.color,
            author: owner,
            categories_formatted: project
                .categories
                .into_iter()
                .map(format_category_or_loader)
                .collect(),
            loaders_formatted: project
                .inner
                .loaders
                .into_iter()
                .map(format_category_or_loader)
                .collect(),
            versions_formatted: formatted_game_versions,
            gallery_image: project
                .gallery_items
                .into_iter()
                .find(|x| x.featured)
                .map(|x| x.image_url),
        }))
    } else {
        Ok(None)
    }
}

pub async fn send_slack_webhook(
    project_id: ProjectId,
    pool: &PgPool,
    redis: &RedisPool,
    webhook_url: String,
    message: Option<String>,
) -> Result<(), ApiError> {
    let metadata = get_webhook_metadata(project_id, pool, redis).await?;

    if let Some(metadata) = metadata {
        let mut blocks = vec![];

        if let Some(message) = message {
            blocks.push(serde_json::json!({
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": message,
                }
            }));
        }

        if let Some(ref author) = metadata.author {
            let mut elements = vec![];

            if let Some(ref icon_url) = author.icon_url {
                elements.push(serde_json::json!({
                    "type": "image",
                    "image_url": icon_url,
                    "alt_text": "Author"
                }));
            }

            elements.push(serde_json::json!({
                "type": "mrkdwn",
                "text": format!("<{}|{}>", author.url, author.name)
            }));

            blocks.push(serde_json::json!({
                "type": "context",
                "elements": elements
            }));
        }

        let mut project_block = serde_json::json!({
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": format!(
                    "*<{}|{}>*\n\n{}\n\n*Categories:* {}\n\n*Loaders:* {}\n\n*Versions:* {}",
                    metadata.project_url,
                    metadata.project_title,
                    metadata.project_summary,
                    metadata.categories_formatted.join(", "),
                    metadata.loaders_formatted.join(", "),
                    metadata.versions_formatted.join(", ")
                )
            }
        });

        if let Some(icon_url) = metadata.project_icon_url {
            if let Some(project_block) = project_block.as_object_mut() {
                project_block.insert(
                    "accessory".to_string(),
                    serde_json::json!({
                        "type": "image",
                        "image_url": icon_url,
                        "alt_text": metadata.project_title
                    }),
                );
            }
        }

        blocks.push(project_block);

        if let Some(gallery_image) = metadata.gallery_image {
            blocks.push(serde_json::json!({
                "type": "image",
                "image_url": gallery_image,
                "alt_text": metadata.project_title
            }));
        }

        blocks.push(
            serde_json::json!({
                "type": "context",
                "elements": [
                    {
                        "type": "image",
                        "image_url": "https://cdn-raw.modrinth.com/modrinth-new.png",
                        "alt_text": "Author"
                    },
                    {
                        "type": "mrkdwn",
                        "text": format!("{} on Modrinth • <!date^{}^{{date_short_pretty}} at {{time}}|Unknown date>", metadata.display_project_type, Utc::now().timestamp())
                    }
                ]
            })
        );

        let client = reqwest::Client::new();

        client
            .post(&webhook_url)
            .json(&serde_json::json!({
                "blocks": blocks,
            }))
            .send()
            .await
            .map_err(|_| {
                ApiError::Discord(
                    "Error while sending projects webhook".to_string(),
                )
            })?;
    }

    Ok(())
}

#[derive(Serialize)]
struct DiscordEmbed {
    pub author: Option<DiscordEmbedAuthor>,
    pub title: String,
    pub description: String,
    pub url: String,
    pub timestamp: DateTime<Utc>,
    pub color: u32,
    pub fields: Vec<DiscordEmbedField>,
    pub thumbnail: DiscordEmbedThumbnail,
    pub image: Option<DiscordEmbedImage>,
    pub footer: Option<DiscordEmbedFooter>,
}

#[derive(Serialize)]
struct DiscordEmbedAuthor {
    pub name: String,
    pub url: Option<String>,
    pub icon_url: Option<String>,
}

#[derive(Serialize)]
struct DiscordEmbedField {
    pub name: &'static str,
    pub value: String,
    pub inline: bool,
}

#[derive(Serialize)]
struct DiscordEmbedImage {
    pub url: Option<String>,
}

#[derive(Serialize)]
struct DiscordEmbedThumbnail {
    pub url: Option<String>,
}

#[derive(Serialize)]
struct DiscordEmbedFooter {
    pub text: String,
    pub icon_url: Option<String>,
}

#[derive(Serialize)]
struct DiscordWebhook {
    pub avatar_url: Option<String>,
    pub username: Option<String>,
    pub embeds: Vec<DiscordEmbed>,
    pub content: Option<String>,
}

pub async fn send_discord_webhook(
    project_id: ProjectId,
    pool: &PgPool,
    redis: &RedisPool,
    webhook_url: String,
    message: Option<String>,
) -> Result<(), ApiError> {
    let metadata = get_webhook_metadata(project_id, pool, redis).await?;

    if let Some(project) = metadata {
        let mut fields = vec![];
        if !project.categories_formatted.is_empty() {
            fields.push(DiscordEmbedField {
                name: "Categories",
                value: project.categories_formatted.join("\n"),
                inline: true,
            });
        }

        if !project.loaders_formatted.is_empty() {
            fields.push(DiscordEmbedField {
                name: "Loaders",
                value: project.loaders_formatted.join("\n"),
                inline: true,
            });
        }

        if !project.versions_formatted.is_empty() {
            fields.push(DiscordEmbedField {
                name: "Versions",
                value: project.versions_formatted.join("\n"),
                inline: true,
            });
        }

        let embed = DiscordEmbed {
            author: project.author.map(|x| DiscordEmbedAuthor {
                name: x.name,
                url: Some(x.url),
                icon_url: x.icon_url,
            }),
            url: project.project_url,
            title: project.project_title, // Do not change DiscordEmbed
            description: project.project_summary,
            timestamp: Utc::now(),
            color: project.color.unwrap_or(0x1bd96a),
            fields,
            thumbnail: DiscordEmbedThumbnail {
                url: project.project_icon_url,
            },
            image: project
                .gallery_image
                .map(|x| DiscordEmbedImage { url: Some(x) }),
            footer: Some(DiscordEmbedFooter {
                text: format!("{} on Modrinth", project.display_project_type),
                icon_url: Some(
                    "https://cdn-raw.modrinth.com/modrinth-new.png".to_string(),
                ),
            }),
        };

        let client = reqwest::Client::new();

        client
            .post(&webhook_url)
            .json(&DiscordWebhook {
                avatar_url: Some(
                    "https://cdn.modrinth.com/Modrinth_Dark_Logo.png"
                        .to_string(),
                ),
                username: Some("Modrinth Release".to_string()),
                embeds: vec![embed],
                content: message,
            })
            .send()
            .await
            .map_err(|_| {
                ApiError::Discord(
                    "Error while sending projects webhook".to_string(),
                )
            })?;
    }

    Ok(())
}

fn get_gv_range(
    mut game_versions: Vec<MinecraftGameVersion>,
    mut all_game_versions: Vec<MinecraftGameVersion>,
) -> Vec<String> {
    // both -> least to greatest
    game_versions.sort_by(|a, b| a.created.cmp(&b.created));
    game_versions.dedup_by(|a, b| a.version == b.version);

    all_game_versions.sort_by(|a, b| a.created.cmp(&b.created));

    let all_releases = all_game_versions
        .iter()
        .filter(|x| &*x.type_ == "release")
        .cloned()
        .collect::<Vec<_>>();

    let mut intervals = Vec::new();
    let mut current_interval = 0;

    const MAX_VALUE: usize = 1000000;

    for (i, current_version) in game_versions.iter().enumerate() {
        let current_version = &current_version.version;

        let index = all_game_versions
            .iter()
            .position(|x| &*x.version == current_version)
            .unwrap_or(MAX_VALUE);
        let release_index = all_releases
            .iter()
            .position(|x| &*x.version == current_version)
            .unwrap_or(MAX_VALUE);

        if i == 0 {
            intervals.push(vec![vec![i, index, release_index]])
        } else {
            let interval_base = &intervals[current_interval];

            if ((index as i32)
                - (interval_base[interval_base.len() - 1][1] as i32)
                == 1
                || (release_index as i32)
                    - (interval_base[interval_base.len() - 1][2] as i32)
                    == 1)
                && (all_game_versions[interval_base[0][1]].type_ == "release"
                    || all_game_versions[index].type_ != "release")
            {
                if intervals[current_interval].get(1).is_some() {
                    intervals[current_interval][1] =
                        vec![i, index, release_index];
                } else {
                    intervals[current_interval]
                        .insert(1, vec![i, index, release_index]);
                }
            } else {
                current_interval += 1;
                intervals.push(vec![vec![i, index, release_index]]);
            }
        }
    }

    let mut new_intervals = Vec::new();

    for interval in intervals {
        if interval.len() == 2
            && interval[0][2] != MAX_VALUE
            && interval[1][2] == MAX_VALUE
        {
            let mut last_snapshot: Option<usize> = None;

            for j in ((interval[0][1] + 1)..=interval[1][1]).rev() {
                if all_game_versions[j].type_ == "release" {
                    new_intervals.push(vec![
                        interval[0].clone(),
                        vec![
                            game_versions
                                .iter()
                                .position(|x| {
                                    x.version == all_game_versions[j].version
                                })
                                .unwrap_or(MAX_VALUE),
                            j,
                            all_releases
                                .iter()
                                .position(|x| {
                                    x.version == all_game_versions[j].version
                                })
                                .unwrap_or(MAX_VALUE),
                        ],
                    ]);

                    if let Some(last_snapshot) = last_snapshot {
                        if last_snapshot != j + 1 {
                            new_intervals.push(vec![
                                vec![
                                    game_versions
                                        .iter()
                                        .position(|x| {
                                            x.version
                                                == all_game_versions
                                                    [last_snapshot]
                                                    .version
                                        })
                                        .unwrap_or(MAX_VALUE),
                                    last_snapshot,
                                    MAX_VALUE,
                                ],
                                interval[1].clone(),
                            ])
                        }
                    } else {
                        new_intervals.push(vec![interval[1].clone()])
                    }

                    break;
                } else {
                    last_snapshot = Some(j);
                }
            }
        } else {
            new_intervals.push(interval);
        }
    }

    let mut output = Vec::new();

    for interval in new_intervals {
        if interval.len() == 2 {
            output.push(format!(
                "{}–{}",
                &game_versions[interval[0][0]].version,
                &game_versions[interval[1][0]].version
            ))
        } else {
            output.push(game_versions[interval[0][0]].version.clone())
        }
    }

    output
}

// Converted from knossos
// See: packages/utils/utils.ts
// https://github.com/modrinth/code/blob/47af459f24e541a844b42b1c8427af6a7b86381e/packages/utils/utils.ts#L147-L196
fn format_category_or_loader(mut x: String) -> String {
    match &*x {
        "modloader" => "Risugami's ModLoader".to_string(),
        "bungeecord" => "BungeeCord".to_string(),
        "liteloader" => "LiteLoader".to_string(),
        "neoforge" => "NeoForge".to_string(),
        "game-mechanics" => "Game Mechanics".to_string(),
        "worldgen" => "World Generation".to_string(),
        "core-shaders" => "Core Shaders".to_string(),
        "gui" => "GUI".to_string(),
        "8x-" => "8x or lower".to_string(),
        "512x+" => "512x or higher".to_string(),
        "kitchen-sink" => "Kitchen Sink".to_string(),
        "path-tracing" => "Path Tracing".to_string(),
        "pbr" => "PBR".to_string(),
        "datapack" => "Data Pack".to_string(),
        "colored-lighting" => "Colored Lighting".to_string(),
        "optifine" => "OptiFine".to_string(),
        "bta-babric" => "BTA (Babric)".to_string(),
        "legacy-fabric" => "Legacy Fabric".to_string(),
        "java-agent" => "Java Agent".to_string(),
        "nilloader" => "NilLoader".to_string(),
        "mrpack" => "Modpack".to_string(),
        "minecraft" => "Resource Pack".to_string(),
        "vanilla" => "Vanilla Shader".to_string(),
        _ => format!("{}{x}", x.remove(0).to_uppercase()),
    }
}
