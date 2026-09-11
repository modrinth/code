use super::apply_content_install::{
    ContentScope, require_stopped_for_content, upsert_entry_for_file,
};
use crate::state::content_store::{
    BlobLease, ContentProjectionStatus, FileContent, content_file_path, input,
};
use crate::state::instances::adapters::sqlite::{content_rows, instance_rows};
use crate::state::instances::{ContentSourceKind, InstanceFile};
use crate::state::{
    KnownModrinthFile, ProjectType, State, cache_file_hash_metadata,
    file_modified_at_ns,
};
use chrono::Utc;
use std::path::Path;
use tokio::fs;
use tokio::sync::{MutexGuard, OwnedMutexGuard};

#[derive(Clone, Copy)]
pub(crate) struct ContentOrigin<'a> {
    pub project_id: &'a str,
    pub version_id: &'a str,
}

pub(crate) struct InstallContent<'a> {
    pub requested_path: &'a str,
    pub blob: &'a BlobLease,
    pub project_type: ProjectType,
    pub source_kind: ContentSourceKind,
    pub origin: Option<ContentOrigin<'a>>,
    pub enabled_override: Option<bool>,
    pub previous_path: Option<&'a str>,
}

pub(super) enum ContentMutation<'a> {
    Install(InstallContent<'a>),
    Toggle {
        project_path: &'a str,
        desired_enabled: Option<bool>,
    },
    Remove {
        project_path: &'a str,
    },
    Adopt {
        file: &'a InstanceFile,
    },
}

pub(super) enum ContentMutationResult {
    File(InstanceFile),
    Removed,
}

enum PreparedChange {
    Install {
        relative_path: String,
        rename_from: Option<String>,
        enabled: bool,
        blob: BlobLease,
        project_type: ProjectType,
        source_kind: ContentSourceKind,
        origin: Option<(String, String)>,
    },
    Toggle {
        file: InstanceFile,
        enabled: bool,
    },
    Remove {
        relative_path: String,
        file: Option<InstanceFile>,
    },
    Adopt {
        file: InstanceFile,
        rename_from: Option<String>,
    },
}

struct PreparedMutation {
    projection: crate::state::content_store::PreparedProjection,
    change: PreparedChange,
}

pub(super) struct ContentMutationExecutor<'a> {
    state: &'a State,
    instance: crate::state::Instance,
    content_set_id: Option<String>,
    _content_lock: OwnedMutexGuard<()>,
    _store_lock: MutexGuard<'a, ()>,
}

impl<'a> ContentMutationExecutor<'a> {
    pub(super) async fn lock(
        instance_id: &str,
        state: &'a State,
    ) -> crate::Result<Self> {
        let content_lock = state.lock_instance_content(instance_id).await;
        let store_lock = state.content_store.files_lock.lock().await;
        let instance =
            instance_rows::get_instance_by_id(instance_id, &state.pool)
                .await?
                .ok_or_else(|| input("Unknown instance"))?;
        let content_set_id = instance.applied_content_set_id.clone();
        state.content_store.recover(Some(instance_id)).await?;
        Ok(Self {
            state,
            instance,
            content_set_id,
            _content_lock: content_lock,
            _store_lock: store_lock,
        })
    }

    fn content_scope(&self) -> crate::Result<ContentScope> {
        Ok(ContentScope {
            instance: self.instance.clone(),
            content_set_id: self.content_set_id.clone().ok_or_else(|| {
                input(format!(
                    "Instance {} has no applied content set",
                    self.instance.id
                ))
            })?,
        })
    }

    pub(super) async fn execute(
        &self,
        request: ContentMutation<'_>,
    ) -> crate::Result<ContentMutationResult> {
        let mut prepared = self.prepare(request).await?;
        prepared.projection.apply(&self.state.content_store).await?;
        let output = match self.commit(&prepared).await {
            Ok(output) => output,
            Err(error) => {
                prepared
                    .projection
                    .rollback(&self.state.content_store)
                    .await?;
                return Err(error);
            }
        };
        super::mark_shared_instance_stale(&self.instance.id, &self.state.pool)
            .await?;
        if let PreparedChange::Install {
            project_type,
            origin,
            ..
        } = &prepared.change
            && let ContentMutationResult::File(file) = &output
        {
            self.cache_install(file, *project_type, origin.as_ref())
                .await?;
        }
        Ok(output)
    }

    async fn prepare(
        &self,
        request: ContentMutation<'_>,
    ) -> crate::Result<PreparedMutation> {
        match request {
            ContentMutation::Install(request) => {
                self.prepare_install(request).await
            }
            ContentMutation::Toggle {
                project_path,
                desired_enabled,
            } => self.prepare_toggle(project_path, desired_enabled).await,
            ContentMutation::Remove { project_path } => {
                self.prepare_remove(project_path).await
            }
            ContentMutation::Adopt { file } => self.prepare_adopt(file).await,
        }
    }

    async fn prepare_install(
        &self,
        request: InstallContent<'_>,
    ) -> crate::Result<PreparedMutation> {
        self.content_scope()?;
        require_stopped_for_content(
            &self.instance.id,
            request.project_type,
            self.state,
        )
        .await?;
        let relative_path = canonical_content_path(request.requested_path);
        if !crate::state::content_store::eligible(relative_path) {
            return Err(input("Unsupported content destination"));
        }
        let previous_path = request.previous_path.map(canonical_content_path);
        let lookup_path = previous_path.unwrap_or(relative_path);
        let existing = content_rows::get_instance_file_by_relative_path(
            &self.instance.id,
            lookup_path,
            &self.state.pool,
        )
        .await?;
        if request.previous_path.is_some() && existing.is_none() {
            return Err(input(
                "Content changed while its update was downloading; refresh and try again",
            ));
        }
        if previous_path.is_some_and(|previous| previous != relative_path)
            && content_rows::get_instance_file_by_relative_path(
                &self.instance.id,
                relative_path,
                &self.state.pool,
            )
            .await?
            .is_some()
        {
            return Err(input(
                "The updated filename belongs to another content item",
            ));
        }
        let enabled = if request.previous_path.is_some() {
            existing.as_ref().is_some_and(|file| file.enabled)
        } else {
            request.enabled_override.unwrap_or_else(|| {
                !request.requested_path.ends_with(".disabled")
                    && existing.as_ref().is_none_or(|file| file.enabled)
            })
        };
        let requested_source = self
            .state
            .content_store
            .instance_path(&self.instance.path, request.requested_path)
            .await?;
        let legacy_path =
            if previous_path.is_some_and(|path| path != relative_path) {
                previous_path
            } else if existing.is_none()
                && fs::symlink_metadata(&requested_source).await.is_ok()
            {
                Some(request.requested_path)
            } else {
                None
            };
        let projection = self
            .state
            .content_store
            .prepare(
                &self.instance,
                relative_path,
                Some(request.blob),
                enabled,
                legacy_path,
                None,
            )
            .await?;
        let rename_from = existing
            .as_ref()
            .map(|file| file.relative_path.as_str())
            .filter(|path| *path != relative_path)
            .map(str::to_string);
        Ok(PreparedMutation {
            projection,
            change: PreparedChange::Install {
                relative_path: relative_path.to_string(),
                rename_from,
                enabled,
                blob: request.blob.clone(),
                project_type: request.project_type,
                source_kind: request.source_kind,
                origin: request.origin.map(|origin| {
                    (
                        origin.project_id.to_string(),
                        origin.version_id.to_string(),
                    )
                }),
            },
        })
    }

    async fn prepare_toggle(
        &self,
        project_path: &str,
        desired_enabled: Option<bool>,
    ) -> crate::Result<PreparedMutation> {
        self.content_scope()?;
        let canonical_path = canonical_content_path(project_path);
        let file = content_rows::get_instance_file_by_relative_path(
            &self.instance.id,
            canonical_path,
            &self.state.pool,
        )
        .await?
        .ok_or_else(|| {
            input(
                "Content file is not registered; refresh the Content tab first",
            )
        })?;
        let project_type =
            super::sync_content_files::project_type_for_file(&file)
                .ok_or_else(|| input("Unsupported content type"))?;
        require_stopped_for_content(
            &self.instance.id,
            project_type,
            self.state,
        )
        .await?;
        let enabled = desired_enabled.unwrap_or(!file.enabled);
        let projection = match self
            .state
            .content_store
            .file_content(&file)
            .await?
        {
            FileContent::Stored(blob) => {
                let binding = crate::state::content_store::catalog::binding(
                    &self.state.pool,
                    &file.id,
                )
                .await?
                .ok_or_else(|| input("Content binding disappeared"))?;
                match self
                    .state
                    .content_store
                    .inspect_projection(&self.instance, &file, &binding)
                    .await?
                {
                    ContentProjectionStatus::Healthy => {
                        self.state
                            .content_store
                            .prepare_move(
                                &self.instance,
                                &file,
                                &binding,
                                enabled,
                            )
                            .await?
                    }
                    ContentProjectionStatus::Missing => {
                        self.state
                            .content_store
                            .prepare(
                                &self.instance,
                                canonical_path,
                                Some(&blob),
                                enabled,
                                None,
                                None,
                            )
                            .await?
                    }
                    ContentProjectionStatus::Conflict => {
                        return Err(input(
                            "Content was changed outside the app; resolve the conflict first",
                        ));
                    }
                }
            }
            FileContent::Damaged(binding) if !enabled => {
                self.state
                    .content_store
                    .prepare_move(&self.instance, &file, &binding, false)
                    .await?
            }
            FileContent::Damaged(_) => {
                return Err(input(
                    "Content needs repair or re-import before it can be enabled",
                ));
            }
            FileContent::Unmanaged => {
                let physical_path = content_file_path(&file);
                let path = self
                    .state
                    .content_store
                    .instance_path(&self.instance.path, &physical_path)
                    .await?;
                if fs::symlink_metadata(&path).await?.file_type().is_symlink() {
                    return Err(input("Cannot adopt an external symlink"));
                }
                let blob = self.state.content_store.ingest_file(&path).await?;
                self.state
                    .content_store
                    .prepare(
                        &self.instance,
                        canonical_path,
                        Some(&blob),
                        enabled,
                        Some(&physical_path),
                        Some(&blob),
                    )
                    .await?
            }
        };
        Ok(PreparedMutation {
            projection,
            change: PreparedChange::Toggle { file, enabled },
        })
    }

    async fn prepare_remove(
        &self,
        project_path: &str,
    ) -> crate::Result<PreparedMutation> {
        self.content_scope()?;
        let relative_path = canonical_content_path(project_path);
        let project_type = ProjectType::get_from_parent_folder(relative_path)
            .ok_or_else(|| input("Unsupported content type"))?;
        require_stopped_for_content(
            &self.instance.id,
            project_type,
            self.state,
        )
        .await?;
        let file = content_rows::get_instance_file_by_relative_path(
            &self.instance.id,
            relative_path,
            &self.state.pool,
        )
        .await?;
        let legacy_path = file.is_none().then_some(project_path);
        let projection = self
            .state
            .content_store
            .prepare(
                &self.instance,
                relative_path,
                None,
                false,
                legacy_path,
                None,
            )
            .await?;
        Ok(PreparedMutation {
            projection,
            change: PreparedChange::Remove {
                relative_path: relative_path.to_string(),
                file,
            },
        })
    }

    async fn prepare_adopt(
        &self,
        file: &InstanceFile,
    ) -> crate::Result<PreparedMutation> {
        let canonical = canonical_content_path(&file.relative_path);
        if !crate::state::content_store::eligible(canonical) {
            return Err(input("Unsupported managed content path"));
        }
        let source_path = self
            .state
            .content_store
            .instance_path(&self.instance.path, &file.relative_path)
            .await?;
        if fs::symlink_metadata(&source_path)
            .await?
            .file_type()
            .is_symlink()
        {
            return Err(input("External symlinks must be imported explicitly"));
        }
        let blob = self.state.content_store.ingest_file(&source_path).await?;
        let enabled = file.enabled && canonical == file.relative_path;
        let canonical_record =
            content_rows::get_instance_file_by_relative_path(
                &self.instance.id,
                canonical,
                &self.state.pool,
            )
            .await?;
        let source_record = if canonical == file.relative_path {
            canonical_record.clone()
        } else {
            content_rows::get_instance_file_by_relative_path(
                &self.instance.id,
                &file.relative_path,
                &self.state.pool,
            )
            .await?
        };
        if let (Some(source), Some(target)) =
            (source_record.as_ref(), canonical_record.as_ref())
            && source.id != target.id
        {
            return Err(input(format!(
                "Both enabled and disabled records exist for {canonical}; preserve both files and resolve the duplicate first"
            )));
        }
        let projection = self
            .state
            .content_store
            .prepare(
                &self.instance,
                canonical,
                Some(&blob),
                enabled,
                Some(&file.relative_path),
                Some(&blob),
            )
            .await?;
        let mut adopted = source_record
            .or(canonical_record)
            .unwrap_or_else(|| file.clone());
        adopted.relative_path = canonical.to_string();
        adopted.file_name = canonical_file_name(canonical)?.to_string();
        adopted.enabled = enabled;
        adopted.sha1 = blob.blob.sha1.clone();
        adopted.size = blob.blob.size as u64;
        adopted.missing = false;
        adopted.modified_at = Utc::now();
        let rename_from = (file.relative_path != canonical)
            .then(|| file.relative_path.clone());
        Ok(PreparedMutation {
            projection,
            change: PreparedChange::Adopt {
                file: adopted,
                rename_from,
            },
        })
    }

    async fn commit(
        &self,
        prepared: &PreparedMutation,
    ) -> crate::Result<ContentMutationResult> {
        let mut tx = self.state.pool.begin().await?;
        let content_scope = match &prepared.change {
            PreparedChange::Adopt { .. } => None,
            _ => Some(self.content_scope()?),
        };
        let result = match &prepared.change {
            PreparedChange::Install {
                relative_path,
                rename_from,
                enabled,
                blob,
                project_type,
                source_kind,
                origin,
                ..
            } => {
                let file_name = canonical_file_name(relative_path)?;
                if let Some(rename_from) = rename_from {
                    content_rows::rename_instance_file(
                        &self.instance.id,
                        rename_from,
                        relative_path,
                        file_name,
                        *enabled,
                        &mut tx,
                    )
                    .await?;
                }
                let file = content_rows::upsert_instance_file_from_parts(
                    content_rows::UpsertInstanceFile {
                        instance_id: &self.instance.id,
                        relative_path,
                        file_name,
                        enabled: *enabled,
                        sha1: &blob.blob.sha1,
                        size: blob.blob.size as u64,
                        missing: false,
                    },
                    &mut tx,
                )
                .await?;
                upsert_entry_for_file(
                    content_scope
                        .as_ref()
                        .expect("install requires content scope"),
                    &file,
                    *project_type,
                    origin.as_ref().map(|origin| origin.0.as_str()),
                    origin.as_ref().map(|origin| origin.1.as_str()),
                    *source_kind,
                    &mut tx,
                )
                .await?;
                prepared.projection.commit(&mut tx, Some(&file.id)).await?;
                ContentMutationResult::File(file)
            }
            PreparedChange::Toggle { file, enabled } => {
                let mut updated = file.clone();
                updated.enabled = *enabled;
                updated.missing = false;
                updated.modified_at = Utc::now();
                let updated =
                    content_rows::upsert_instance_file(&updated, &mut tx)
                        .await?;
                content_rows::set_content_entry_enabled_for_file(
                    &content_scope
                        .as_ref()
                        .expect("toggle requires content scope")
                        .content_set_id,
                    &updated.id,
                    *enabled,
                    &mut tx,
                )
                .await?;
                prepared
                    .projection
                    .commit(&mut tx, Some(&updated.id))
                    .await?;
                ContentMutationResult::File(updated)
            }
            PreparedChange::Remove {
                relative_path,
                file,
            } => {
                if let Some(file) = file {
                    content_rows::remove_content_entries_for_file(
                        &content_scope
                            .as_ref()
                            .expect("remove requires content scope")
                            .content_set_id,
                        &file.id,
                        &mut tx,
                    )
                    .await?;
                    content_rows::remove_instance_file_by_relative_path(
                        &self.instance.id,
                        relative_path,
                        &mut tx,
                    )
                    .await?;
                }
                prepared.projection.commit(&mut tx, None).await?;
                ContentMutationResult::Removed
            }
            PreparedChange::Adopt { file, rename_from } => {
                if let Some(rename_from) = rename_from {
                    content_rows::rename_instance_file(
                        &self.instance.id,
                        rename_from,
                        &file.relative_path,
                        &file.file_name,
                        file.enabled,
                        &mut tx,
                    )
                    .await?;
                }
                let adopted =
                    content_rows::upsert_instance_file(file, &mut tx).await?;
                if let Some(content_set_id) = &self.content_set_id {
                    content_rows::set_content_entry_enabled_for_file(
                        content_set_id,
                        &adopted.id,
                        adopted.enabled,
                        &mut tx,
                    )
                    .await?;
                }
                prepared
                    .projection
                    .commit(&mut tx, Some(&adopted.id))
                    .await?;
                ContentMutationResult::File(adopted)
            }
        };
        tx.commit().await?;
        Ok(result)
    }

    async fn cache_install(
        &self,
        file: &InstanceFile,
        project_type: ProjectType,
        origin: Option<&(String, String)>,
    ) -> crate::Result<()> {
        let physical_path = content_file_path(file);
        let metadata = fs::metadata(
            self.state
                .content_store
                .instance_path(&self.instance.path, &physical_path)
                .await?,
        )
        .await?;
        cache_file_hash_metadata(
            &self.instance.path,
            &physical_path,
            metadata.len(),
            file_modified_at_ns(&metadata)?,
            file.sha1.clone(),
            Some(project_type),
            origin.map(|origin| KnownModrinthFile {
                project_id: &origin.0,
                version_id: &origin.1,
            }),
            &self.state.pool,
        )
        .await
    }
}

pub(crate) async fn install_content_blob(
    instance_id: &str,
    request: InstallContent<'_>,
    state: &State,
) -> crate::Result<String> {
    let executor = ContentMutationExecutor::lock(instance_id, state).await?;
    match executor.execute(ContentMutation::Install(request)).await? {
        ContentMutationResult::File(file) => Ok(file.relative_path),
        _ => unreachable!("install mutations return a content path"),
    }
}

pub(crate) async fn toggle_disable_project(
    instance_id: &str,
    project_path: &str,
    desired_enabled: Option<bool>,
    state: &State,
) -> crate::Result<String> {
    let executor = ContentMutationExecutor::lock(instance_id, state).await?;
    match executor
        .execute(ContentMutation::Toggle {
            project_path,
            desired_enabled,
        })
        .await?
    {
        ContentMutationResult::File(file) => Ok(file.relative_path),
        _ => unreachable!("toggle mutations return a content path"),
    }
}

pub(crate) async fn remove_project(
    instance_id: &str,
    project_path: &str,
    state: &State,
) -> crate::Result<()> {
    let executor = ContentMutationExecutor::lock(instance_id, state).await?;
    match executor
        .execute(ContentMutation::Remove { project_path })
        .await?
    {
        ContentMutationResult::Removed => Ok(()),
        _ => unreachable!("remove mutations return no content"),
    }
}

fn canonical_content_path(path: &str) -> &str {
    path.trim_end_matches(".disabled")
}

fn canonical_file_name(relative_path: &str) -> crate::Result<&str> {
    Path::new(relative_path)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| input("Invalid content filename"))
}
