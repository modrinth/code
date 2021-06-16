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
    pub name: String,
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
    let mut fields = Vec::new();

    fields.push(DiscordEmbedField {
        name: "id".to_string(),
        value: project.id.to_string(),
        inline: true,
    });

    if let Some(slug) = project.slug.clone() {
        fields.push(DiscordEmbedField {
            name: "slug".to_string(),
            value: slug,
            inline: true,
        });
    }

    fields.push(DiscordEmbedField {
        name: "project_type".to_string(),
        value: project.project_type.to_string(),
        inline: true,
    });

    fields.push(DiscordEmbedField {
        name: "client_side".to_string(),
        value: project.client_side.to_string(),
        inline: true,
    });

    fields.push(DiscordEmbedField {
        name: "server_side".to_string(),
        value: project.server_side.to_string(),
        inline: true,
    });

    fields.push(DiscordEmbedField {
        name: "categories".to_string(),
        value: project.categories.join(", "),
        inline: true,
    });

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
        color: 6137157,
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
