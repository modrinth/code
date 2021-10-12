use crate::models::projects::Project;
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
struct DiscordEmbed {
    pub title: String,
    pub description: String,
    pub url: String,
    pub timestamp: DateTime<Utc>,
    pub color: u32,
    pub fields: Vec<DiscordEmbedField>,
    pub image: DiscordEmbedImage,
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
struct DiscordWebhook {
    pub embeds: Vec<DiscordEmbed>,
}

pub async fn send_discord_webhook(
    project: Project,
    webhook_url: String,
) -> Result<(), reqwest::Error> {
    let mut fields = vec![
        DiscordEmbedField {
            name: "id",
            value: project.id.to_string(),
            inline: true,
        },
        DiscordEmbedField {
            name: "project_type",
            value: project.project_type.clone(),
            inline: true,
        },
        DiscordEmbedField {
            name: "client_side",
            value: project.client_side.to_string(),
            inline: true,
        },
        DiscordEmbedField {
            name: "server_side",
            value: project.server_side.to_string(),
            inline: true,
        },
        DiscordEmbedField {
            name: "categories",
            value: project.categories.join(", "),
            inline: true,
        },
    ];

    if let Some(ref slug) = project.slug {
        fields.push(DiscordEmbedField {
            name: "slug",
            value: slug.clone(),
            inline: true,
        });
    }

    let embed = DiscordEmbed {
        url: format!(
            "{}/mod/{}",
            dotenv::var("SITE_URL").unwrap_or_default(),
            project
                .clone()
                .slug
                .unwrap_or_else(|| project.id.to_string())
        ),
        title: project.title,
        description: project.description,
        timestamp: project.published,
        color: 0x5DA545,
        fields,
        image: DiscordEmbedImage {
            url: project.icon_url,
        },
    };

    let client = reqwest::Client::new();

    client
        .post(&webhook_url)
        .json(&DiscordWebhook {
            embeds: vec![embed],
        })
        .send()
        .await?;

    Ok(())
}
