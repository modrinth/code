use crate::State;
use crate::state::{
    ContentItem, InstanceMetadata, LinkedModpackInfo, ProjectType,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContentListFormat {
    #[default]
    Detailed,
    Names,
    FileNames,
    Urls,
    Markdown,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub struct ContentListOptions {
    #[serde(default)]
    pub format: ContentListFormat,
}

#[derive(Debug, Serialize)]
pub(crate) struct InstanceContentSnapshot {
    pub linked_modpack: Option<LinkedModpackInfo>,
    pub modpack_content: Vec<ContentItem>,
    pub content: Vec<ContentItem>,
}

impl InstanceContentSnapshot {
    pub(crate) async fn load(instance_id: &str) -> crate::Result<Self> {
        let (content, linked_modpack, modpack_content) = tokio::try_join!(
            super::get_content_items(instance_id, None),
            super::get_linked_modpack_info(instance_id, None),
            super::get_linked_modpack_content(instance_id, None),
        )?;
        Ok(Self {
            linked_modpack,
            modpack_content,
            content,
        })
    }

    fn items(&self) -> impl Iterator<Item = &ContentItem> {
        self.modpack_content.iter().chain(&self.content)
    }

    fn modpack_url(&self) -> Option<String> {
        self.linked_modpack.as_ref().map(|modpack| {
            format!(
                "https://modrinth.com/{}/{}",
                modpack.project.project_type,
                modpack.project.slug.as_ref().unwrap_or(&modpack.project.id)
            )
        })
    }

    fn entries(&self) -> Vec<(&str, Option<String>)> {
        self.linked_modpack
            .as_ref()
            .map(|modpack| (modpack.project.title.as_str(), self.modpack_url()))
            .into_iter()
            .chain(
                self.items()
                    .map(|item| (display_name(item), item_url(item))),
            )
            .collect()
    }

    pub(crate) fn format(
        &self,
        metadata: &InstanceMetadata,
        format: ContentListFormat,
    ) -> String {
        match format {
            ContentListFormat::Detailed => self.detailed(metadata),
            ContentListFormat::Names => self
                .entries()
                .into_iter()
                .map(|(name, _)| name)
                .collect::<Vec<_>>()
                .join("\n"),
            ContentListFormat::FileNames => self
                .items()
                .map(|item| item.file_name.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
            ContentListFormat::Urls => self
                .entries()
                .into_iter()
                .filter_map(|(_, url)| url)
                .collect::<Vec<_>>()
                .join("\n"),
            ContentListFormat::Markdown => self
                .entries()
                .into_iter()
                .map(|(name, url)| match url {
                    Some(url) => format!("[{name}]({url})"),
                    None => name.to_string(),
                })
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }

    fn detailed(&self, metadata: &InstanceMetadata) -> String {
        let content_set = &metadata.applied_content_set;
        let total = self.items().count();
        let disabled = self.items().filter(|item| !item.enabled).count();
        let mut lines = vec![
            metadata.instance.name.clone(),
            format!(
                "Minecraft {} · {}{}",
                content_set.game_version,
                content_set.loader.display_name(),
                content_set
                    .loader_version
                    .as_ref()
                    .map(|version| format!(" {version}"))
                    .unwrap_or_default()
            ),
            if disabled > 0 {
                format!("{total} files ({disabled} disabled)")
            } else {
                format!("{total} files")
            },
        ];

        if let Some(modpack) = &self.linked_modpack {
            lines.push(String::new());
            lines.push(match &modpack.version {
                Some(version) => format!(
                    "Modpack: {} {}",
                    modpack.project.title, version.version_number
                ),
                None => format!("Modpack: {}", modpack.project.title),
            });
            if let Some(url) = self.modpack_url() {
                lines.push(format!("  Link:    {url}"));
            }
        }

        let has_modpack =
            self.linked_modpack.is_some() || !self.modpack_content.is_empty();
        push_groups(
            &mut lines,
            &self.modpack_content,
            has_modpack.then_some("Modpack"),
        );
        push_groups(&mut lines, &self.content, has_modpack.then_some("Added"));
        lines.join("\n")
    }
}

fn display_name(item: &ContentItem) -> &str {
    item.project
        .as_ref()
        .map(|project| project.title.as_str())
        .or_else(|| {
            item.embedded_metadata
                .as_ref()
                .and_then(|metadata| metadata.name.as_deref())
        })
        .unwrap_or(&item.file_name)
}

fn item_url(item: &ContentItem) -> Option<String> {
    item.project.as_ref().map(|project| {
        format!(
            "https://modrinth.com/{}/{}",
            item.project_type.get_name(),
            project.slug.as_ref().unwrap_or(&project.id)
        )
    })
}

fn push_groups(
    lines: &mut Vec<String>,
    items: &[ContentItem],
    label: Option<&str>,
) {
    for project_type in ProjectType::iterator() {
        let mut group = items
            .iter()
            .filter(|item| item.project_type == project_type)
            .collect::<Vec<_>>();
        if group.is_empty() {
            continue;
        }
        group.sort_by_key(|item| display_name(item).to_lowercase());
        lines.push(String::new());
        lines.push(format!(
            "== {}{} ({}) ==",
            label.map(|label| format!("{label}: ")).unwrap_or_default(),
            project_type.plural_display_name(),
            group.len()
        ));
        for item in group {
            lines.push(String::new());
            lines.push(if item.enabled {
                display_name(item).to_string()
            } else {
                format!("{} (disabled)", display_name(item))
            });
            let version = item
                .version
                .as_ref()
                .map(|version| version.version_number.as_str())
                .or_else(|| {
                    item.embedded_metadata
                        .as_ref()
                        .and_then(|metadata| metadata.version.as_deref())
                });
            if let Some(version) = version {
                lines.push(format!("  Version: {version}"));
            }
            lines.push(format!("  File:    {}", item.file_name));
            if let Some(hash) = &item.hash {
                lines.push(format!("  SHA-1:   {hash}"));
            }
            if let Some(url) = item_url(item) {
                lines.push(format!("  Link:    {url}"));
            }
        }
    }
}

/// Builds a list of the instance's content, including its linked modpack and the modpack's
/// files, in the given format.
#[tracing::instrument]
pub async fn get_content_list(
    instance_id: &str,
    options: ContentListOptions,
) -> crate::Result<String> {
    let state = State::get().await?;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError(format!(
                "Instance {instance_id} does not exist"
            ))
        })?;
    let snapshot = InstanceContentSnapshot::load(instance_id).await?;
    Ok(snapshot.format(&metadata, options.format))
}
