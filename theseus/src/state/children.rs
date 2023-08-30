use super::{Profile, ProfilePathId};
use chrono::{DateTime, Utc};
use std::process::ExitStatus;
use std::{collections::HashMap, sync::Arc};
use tokio::process::Child;
use tokio::process::Command;
use tokio::sync::RwLock;

use crate::event::emit::emit_process;
use crate::event::ProcessPayloadType;
use crate::profile;
use crate::util::io::IOError;

use tokio::task::JoinHandle;
use uuid::Uuid;

// Child processes (instances of Minecraft)
// A wrapper over a Hashmap connecting PID -> MinecraftChild
pub struct Children(HashMap<Uuid, Arc<RwLock<MinecraftChild>>>);

// Minecraft Child, bundles together the PID, the actual Child, and the easily queryable stdout and stderr streams
#[derive(Debug)]
pub struct MinecraftChild {
    pub uuid: Uuid,
    pub profile_relative_path: ProfilePathId,
    pub manager: Option<JoinHandle<crate::Result<ExitStatus>>>, // None when future has completed and been handled
    pub current_child: Arc<RwLock<Child>>,
    pub last_updated_playtime: DateTime<Utc>, // The last time we updated the playtime for the associated profile
}

impl Children {
    pub fn new() -> Children {
        Children(HashMap::new())
    }

    // Runs the command in process, inserts a child process to keep track of, and returns a reference to the container struct MinecraftChild
    // The threads for stdout and stderr are spawned here
    // Unlike a Hashmap's 'insert', this directly returns the reference to the MinecraftChild rather than any previously stored MinecraftChild that may exist

    #[tracing::instrument(skip(
        self,
        uuid,
        mc_command,
        post_command,
        censor_strings
    ))]
    #[tracing::instrument(level = "trace", skip(self))]
    #[theseus_macros::debug_pin]
    pub async fn insert_process(
        &mut self,
        uuid: Uuid,
        profile_relative_path: ProfilePathId,
        mut mc_command: Command,
        post_command: Option<Command>, // Command to run after minecraft.
        censor_strings: HashMap<String, String>,
    ) -> crate::Result<Arc<RwLock<MinecraftChild>>> {
        // Takes the first element of the commands vector and spawns it
        let child = mc_command.spawn().map_err(IOError::from)?;

        // Slots child into manager
        let pid = child.id().ok_or_else(|| {
            crate::ErrorKind::LauncherError(
                "Process immediately failed, could not get PID".to_string(),
            )
        })?;
        let current_child = Arc::new(RwLock::new(child));
        let manager = Some(tokio::spawn(Self::sequential_process_manager(
            uuid,
            post_command,
            pid,
            current_child.clone(),
            profile_relative_path.clone(),
        )));

        emit_process(
            uuid,
            pid,
            ProcessPayloadType::Launched,
            "Launched Minecraft",
        )
        .await?;

        let last_updated_playtime = Utc::now();

        // Create MinecraftChild
        let mchild = MinecraftChild {
            uuid,
            profile_relative_path,
            current_child,
            manager,
            last_updated_playtime,
        };

        let mchild = Arc::new(RwLock::new(mchild));
        self.0.insert(uuid, mchild.clone());
        Ok(mchild)
    }

    // Spawns a new child process and inserts it into the hashmap
    // Also, as the process ends, it spawns the follow-up process if it exists
    // By convention, ExitStatus is last command's exit status, and we exit on the first non-zero exit status
    #[tracing::instrument(skip(current_child))]
    #[theseus_macros::debug_pin]
    async fn sequential_process_manager(
        uuid: Uuid,
        post_command: Option<Command>,
        mut current_pid: u32,
        current_child: Arc<RwLock<Child>>,
        associated_profile: ProfilePathId,
    ) -> crate::Result<ExitStatus> {
        let current_child = current_child.clone();

        // Wait on current Minecraft Child
        let mut mc_exit_status;
        let mut last_updated_playtime = Utc::now();
        loop {
            if let Some(t) = current_child
                .write()
                .await
                .try_wait()
                .map_err(IOError::from)?
            {
                mc_exit_status = t;
                break;
            }
            // sleep for 10ms
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

            // Auto-update playtime every minute
            let diff = Utc::now()
                .signed_duration_since(last_updated_playtime)
                .num_seconds();
            if diff >= 60 {
                if let Err(e) = profile::edit(&associated_profile, |prof| {
                    prof.metadata.recent_time_played += diff as u64;
                    async { Ok(()) }
                })
                .await
                {
                    tracing::warn!(
                        "Failed to update playtime for profile {}: {}",
                        associated_profile,
                        e
                    );
                }
                last_updated_playtime = Utc::now();
            }
        }

        // Now fully complete- update playtime one last time
        let diff = Utc::now()
            .signed_duration_since(last_updated_playtime)
            .num_seconds();
        if let Err(e) = profile::edit(&associated_profile, |prof| {
            prof.metadata.recent_time_played += diff as u64;
            async { Ok(()) }
        })
        .await
        {
            tracing::warn!(
                "Failed to update playtime for profile {}: {}",
                associated_profile,
                e
            );
        }

        // Publish play time update
        // Allow failure, it will be stored locally and sent next time
        // Sent in another thread as first call may take a couple seconds and hold up process ending
        tokio::spawn(async move {
            if let Err(e) =
                profile::try_update_playtime(&associated_profile).await
            {
                tracing::warn!(
                    "Failed to update playtime for profile {}: {}",
                    associated_profile,
                    e
                );
            }
        });

        {
            // Clear game played for Discord RPC
            // May have other active processes, so we clear to the next running process
            let state = crate::State::get().await?;
            let _ = state.discord_rpc.clear_to_default(true).await;
        }

        // If in tauri, window should show itself again after process exists if it was hidden
        #[cfg(feature = "tauri")]
        {
            let window = crate::EventState::get_main_window().await?;
            if let Some(window) = window {
                window.unminimize()?;
            }
        }

        if !mc_exit_status.success() {
            emit_process(
                uuid,
                current_pid,
                ProcessPayloadType::Finished,
                "Exited process",
            )
            .await?;

            return Ok(mc_exit_status); // Err for a non-zero exit is handled in helper
        }

        // If a post-command exist, switch to it and wait on it
        if let Some(mut m_command) = post_command {
            {
                let mut current_child = current_child.write().await;
                let new_child = m_command.spawn().map_err(IOError::from)?;
                current_pid = new_child.id().ok_or_else(|| {
                    crate::ErrorKind::LauncherError(
                        "Process immediately failed, could not get PID"
                            .to_string(),
                    )
                })?;
                *current_child = new_child;
            }
            emit_process(
                uuid,
                current_pid,
                ProcessPayloadType::Updated,
                "Completed Minecraft, switching to post-commands",
            )
            .await?;

            loop {
                if let Some(t) = current_child
                    .write()
                    .await
                    .try_wait()
                    .map_err(IOError::from)?
                {
                    mc_exit_status = t;
                    break;
                }
                // sleep for 10ms
                tokio::time::sleep(tokio::time::Duration::from_millis(10))
                    .await;
            }
        }

        emit_process(
            uuid,
            current_pid,
            ProcessPayloadType::Finished,
            "Exited process",
        )
        .await?;

        Ok(mc_exit_status)
    }

    // Returns a ref to the child
    pub fn get(&self, uuid: &Uuid) -> Option<Arc<RwLock<MinecraftChild>>> {
        self.0.get(uuid).cloned()
    }

    // Gets all PID keys
    pub fn keys(&self) -> Vec<Uuid> {
        self.0.keys().cloned().collect()
    }

    // Get exit status of a child by PID
    // Returns None if the child is still running
    pub async fn exit_status(
        &self,
        uuid: &Uuid,
    ) -> crate::Result<Option<std::process::ExitStatus>> {
        if let Some(child) = self.get(uuid) {
            let child = child.write().await;
            let status = child
                .current_child
                .write()
                .await
                .try_wait()
                .map_err(IOError::from)?;
            Ok(status)
        } else {
            Ok(None)
        }
    }

    // Gets all PID keys of running children
    pub async fn running_keys(&self) -> crate::Result<Vec<Uuid>> {
        let mut keys = Vec::new();
        for key in self.keys() {
            if let Some(child) = self.get(&key) {
                let child = child.clone();
                let child = child.write().await;
                if child
                    .current_child
                    .write()
                    .await
                    .try_wait()
                    .map_err(IOError::from)?
                    .is_none()
                {
                    keys.push(key);
                }
            }
        }
        Ok(keys)
    }

    // Gets all PID keys of running children with a given profile path
    pub async fn running_keys_with_profile(
        &self,
        profile_path: ProfilePathId,
    ) -> crate::Result<Vec<Uuid>> {
        let running_keys = self.running_keys().await?;
        let mut keys = Vec::new();
        for key in running_keys {
            if let Some(child) = self.get(&key) {
                let child = child.clone();
                let child = child.read().await;
                if child.profile_relative_path == profile_path {
                    keys.push(key);
                }
            }
        }
        Ok(keys)
    }

    // Gets all profiles of running children
    pub async fn running_profile_paths(
        &self,
    ) -> crate::Result<Vec<ProfilePathId>> {
        let mut profiles = Vec::new();
        for key in self.keys() {
            if let Some(child) = self.get(&key) {
                let child = child.clone();
                let child = child.write().await;
                if child
                    .current_child
                    .write()
                    .await
                    .try_wait()
                    .map_err(IOError::from)?
                    .is_none()
                {
                    profiles.push(child.profile_relative_path.clone());
                }
            }
        }
        Ok(profiles)
    }

    // Gets all profiles of running children
    // Returns clones because it would be serialized anyway
    pub async fn running_profiles(&self) -> crate::Result<Vec<Profile>> {
        let mut profiles = Vec::new();
        for key in self.keys() {
            if let Some(child) = self.get(&key) {
                let child = child.clone();
                let child = child.write().await;
                if child
                    .current_child
                    .write()
                    .await
                    .try_wait()
                    .map_err(IOError::from)?
                    .is_none()
                {
                    if let Some(prof) = crate::api::profile::get(
                        &child.profile_relative_path.clone(),
                        None,
                    )
                    .await?
                    {
                        profiles.push(prof);
                    }
                }
            }
        }
        Ok(profiles)
    }
}

impl Default for Children {
    fn default() -> Self {
        Self::new()
    }
}
