use super::apply_content_install::{
    ContentScope, require_stopped_for_content, upsert_entry_for_file,
};
use crate::state::content_store::{
    FileContent, InstanceFileStatus, StoredFileHandle, content_file_path, input,
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
    pub stored_file: &'a StoredFileHandle,
    pub project_type: ProjectType,
    pub source_kind: ContentSourceKind,
    pub origin: Option<ContentOrigin<'a>>,
    pub enabled_override: Option<bool>,
    pub previous_path: Option<&'a str>,
}

pub(super) enum ContentChange<'a> {
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

pub(super) enum ContentChangeResult {
    File(InstanceFile),
    Removed,
    Deferred { reason: String },
}

enum PreparedChange {
    Install {
        relative_path: String,
        rename_from: Option<String>,
        enabled: bool,
        stored_file: StoredFileHandle,
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

fn adoption_can_be_deferred(error: &crate::Error) -> bool {
    matches!(
        error.raw.as_ref(),
        crate::ErrorKind::StdIOError(_)
            | crate::ErrorKind::IOError(_)
            | crate::ErrorKind::InputError(_)
    )
}

struct PendingContentChange {
    file_change: crate::state::content_store::PendingFileChange,
    change: PreparedChange,
}

pub(super) struct InstanceContent<'a> {
    state: &'a State,
    instance: crate::state::Instance,
    content_set_id: Option<String>,
    _content_lock: OwnedMutexGuard<()>,
    _store_lock: MutexGuard<'a, ()>,
}

impl<'a> InstanceContent<'a> {
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

    pub(super) fn instance(&self) -> &crate::state::Instance {
        &self.instance
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

    pub(super) async fn apply_change(
        &self,
        request: ContentChange<'_>,
    ) -> crate::Result<ContentChangeResult> {
        let adopting = matches!(&request, ContentChange::Adopt { .. });
        let mut prepared = match request {
            ContentChange::Adopt { file } => {
                match self.prepare_adopt(file).await {
                    Ok(Some(prepared)) => prepared,
                    Ok(None) => return Ok(ContentChangeResult::Deferred {
                        reason:
                            "Content links are unavailable in this directory"
                                .to_string(),
                    }),
                    Err(error) if adoption_can_be_deferred(&error) => {
                        return Ok(ContentChangeResult::Deferred {
                            reason: error.to_string(),
                        });
                    }
                    Err(error) => return Err(error),
                }
            }
            ContentChange::Install(request) => {
                self.prepare_install(request).await?
            }
            ContentChange::Toggle {
                project_path,
                desired_enabled,
            } => self.prepare_toggle(project_path, desired_enabled).await?,
            ContentChange::Remove { project_path } => {
                self.prepare_remove(project_path).await?
            }
        };
        if let Err(error) =
            prepared.file_change.apply(&self.state.content_store).await
        {
            if adopting
                && prepared.file_change.safe_to_defer
                && adoption_can_be_deferred(&error)
            {
                return Ok(ContentChangeResult::Deferred {
                    reason: error.to_string(),
                });
            }
            return Err(error);
        }
        let output = match self.commit(&prepared).await {
            Ok(output) => output,
            Err(error) => {
                prepared
                    .file_change
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
            && let ContentChangeResult::File(file) = &output
        {
            self.cache_install(file, *project_type, origin.as_ref())
                .await?;
        }
        Ok(output)
    }

    async fn prepare_install(
        &self,
        request: InstallContent<'_>,
    ) -> crate::Result<PendingContentChange> {
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
        for file in content_rows::get_instance_files(
            &self.instance.id,
            &self.state.pool,
        )
        .await?
        {
            let registered = canonical_content_path(&file.relative_path);
            if registered != relative_path
                && registered.eq_ignore_ascii_case(relative_path)
                && Some(registered) != previous_path
            {
                return Err(input(format!(
                    "Content destination {relative_path} differs only in case from the registered file {registered}; rename or remove that file first",
                )));
            }
        }
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
        let file_change = self
            .state
            .content_store
            .prepare_file_change(
                &self.instance,
                relative_path,
                Some(request.stored_file),
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
        Ok(PendingContentChange {
            file_change,
            change: PreparedChange::Install {
                relative_path: relative_path.to_string(),
                rename_from,
                enabled,
                stored_file: request.stored_file.clone(),
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
    ) -> crate::Result<PendingContentChange> {
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
        let file_change = match self
            .state
            .content_store
            .file_content(&file)
            .await?
        {
            FileContent::Stored {
                storage: binding,
                stored_file,
            } => {
                match self
                    .state
                    .content_store
                    .check_instance_file(&self.instance, &file, &binding)
                    .await?
                {
                    InstanceFileStatus::Healthy => {
                        self.state
                            .content_store
                            .prepare_file_move(
                                &self.instance,
                                &file,
                                &binding,
                                enabled,
                            )
                            .await?
                    }
                    InstanceFileStatus::Missing => {
                        self.state
                            .content_store
                            .prepare_file_change(
                                &self.instance,
                                canonical_path,
                                Some(&stored_file),
                                enabled,
                                None,
                                None,
                            )
                            .await?
                    }
                    InstanceFileStatus::Conflict => {
                        return Err(input(
                            "Content was changed outside the app; resolve the conflict first",
                        ));
                    }
                }
            }
            FileContent::Damaged(binding) if !enabled => {
                self.state
                    .content_store
                    .prepare_file_move(&self.instance, &file, &binding, false)
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
                let stored_file =
                    self.state.content_store.store_file(&path).await?;
                self.state
                    .content_store
                    .prepare_file_change(
                        &self.instance,
                        canonical_path,
                        Some(&stored_file),
                        enabled,
                        Some(&physical_path),
                        Some(&stored_file),
                    )
                    .await?
            }
        };
        Ok(PendingContentChange {
            file_change,
            change: PreparedChange::Toggle { file, enabled },
        })
    }

    async fn prepare_remove(
        &self,
        project_path: &str,
    ) -> crate::Result<PendingContentChange> {
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
        let file_change = self
            .state
            .content_store
            .prepare_file_change(
                &self.instance,
                relative_path,
                None,
                false,
                legacy_path,
                None,
            )
            .await?;
        Ok(PendingContentChange {
            file_change,
            change: PreparedChange::Remove {
                relative_path: relative_path.to_string(),
                file,
            },
        })
    }

    async fn prepare_adopt(
        &self,
        file: &InstanceFile,
    ) -> crate::Result<Option<PendingContentChange>> {
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
        if !self
            .state
            .content_store
            .supports_content_links(&source_path)
            .await?
        {
            return Ok(None);
        }
        let stored_file =
            self.state.content_store.store_file(&source_path).await?;
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
        let file_change = self
            .state
            .content_store
            .prepare_file_change(
                &self.instance,
                canonical,
                Some(&stored_file),
                enabled,
                Some(&file.relative_path),
                Some(&stored_file),
            )
            .await?;
        let mut adopted = source_record
            .or(canonical_record)
            .unwrap_or_else(|| file.clone());
        adopted.relative_path = canonical.to_string();
        adopted.file_name = canonical_file_name(canonical)?.to_string();
        adopted.enabled = enabled;
        adopted.sha1 = stored_file.metadata.sha1.clone();
        adopted.size = stored_file.metadata.size as u64;
        adopted.missing = false;
        adopted.modified_at = Utc::now();
        let rename_from = (file.relative_path != canonical)
            .then(|| file.relative_path.clone());
        Ok(Some(PendingContentChange {
            file_change,
            change: PreparedChange::Adopt {
                file: adopted,
                rename_from,
            },
        }))
    }

    async fn commit(
        &self,
        prepared: &PendingContentChange,
    ) -> crate::Result<ContentChangeResult> {
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
                stored_file,
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
                        sha1: &stored_file.metadata.sha1,
                        size: stored_file.metadata.size as u64,
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
                prepared.file_change.commit(&mut tx, Some(&file.id)).await?;
                ContentChangeResult::File(file)
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
                    .file_change
                    .commit(&mut tx, Some(&updated.id))
                    .await?;
                ContentChangeResult::File(updated)
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
                prepared.file_change.commit(&mut tx, None).await?;
                ContentChangeResult::Removed
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
                    .file_change
                    .commit(&mut tx, Some(&adopted.id))
                    .await?;
                ContentChangeResult::File(adopted)
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

pub(crate) async fn install_stored_file(
    instance_id: &str,
    request: InstallContent<'_>,
    state: &State,
) -> crate::Result<String> {
    let instance_content = InstanceContent::lock(instance_id, state).await?;
    match instance_content
        .apply_change(ContentChange::Install(request))
        .await?
    {
        ContentChangeResult::File(file) => Ok(file.relative_path),
        _ => unreachable!("install mutations return a content path"),
    }
}

pub(crate) async fn toggle_disable_project(
    instance_id: &str,
    project_path: &str,
    desired_enabled: Option<bool>,
    state: &State,
) -> crate::Result<String> {
    let instance_content = InstanceContent::lock(instance_id, state).await?;
    match instance_content
        .apply_change(ContentChange::Toggle {
            project_path,
            desired_enabled,
        })
        .await?
    {
        ContentChangeResult::File(file) => Ok(file.relative_path),
        _ => unreachable!("toggle mutations return a content path"),
    }
}

pub(crate) async fn remove_project(
    instance_id: &str,
    project_path: &str,
    state: &State,
) -> crate::Result<()> {
    let instance_content = InstanceContent::lock(instance_id, state).await?;
    match instance_content
        .apply_change(ContentChange::Remove { project_path })
        .await?
    {
        ContentChangeResult::Removed => Ok(()),
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
