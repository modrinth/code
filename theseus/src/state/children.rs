use super::{Profile, ProfilePathId};
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use std::{collections::HashMap, sync::Arc};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Child;
use tokio::process::Command;
use tokio::process::{ChildStderr, ChildStdout};
use tokio::sync::RwLock;
use tracing::error;

use crate::event::emit::emit_process;
use crate::event::ProcessPayloadType;
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
    pub output: SharedOutput,
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
        log_path,
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
        log_path: PathBuf,
        mut mc_command: Command,
        post_command: Option<Command>, // Command to run after minecraft.
        censor_strings: HashMap<String, String>,
    ) -> crate::Result<Arc<RwLock<MinecraftChild>>> {
        // Takes the first element of the commands vector and spawns it
        let mut child = mc_command.spawn().map_err(IOError::from)?;

        // Create std watcher threads for stdout and stderr
        let shared_output =
            SharedOutput::build(&log_path, censor_strings).await?;
        if let Some(child_stdout) = child.stdout.take() {
            let stdout_clone = shared_output.clone();
            tokio::spawn(async move {
                if let Err(e) = stdout_clone.read_stdout(child_stdout).await {
                    error!("Stdout process died with error: {}", e);
                }
            });
        }
        if let Some(child_stderr) = child.stderr.take() {
            let stderr_clone = shared_output.clone();
            tokio::spawn(async move {
                if let Err(e) = stderr_clone.read_stderr(child_stderr).await {
                    error!("Stderr process died with error: {}", e);
                }
            });
        }

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
        )));

        emit_process(
            uuid,
            pid,
            ProcessPayloadType::Launched,
            "Launched Minecraft",
        )
        .await?;

        // Create MinecraftChild
        let mchild = MinecraftChild {
            uuid,
            profile_relative_path,
            current_child,
            output: shared_output,
            manager,
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
    ) -> crate::Result<ExitStatus> {
        let current_child = current_child.clone();

        // Wait on current Minecraft Child
        let mut mc_exit_status;
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
        }

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

// SharedOutput, a wrapper around a String that can be read from and written to concurrently
// Designed to be used with ChildStdout and ChildStderr in a tokio thread to have a simple String storage for the output of a child process
#[derive(Debug, Clone)]
pub struct SharedOutput {
    output: Arc<RwLock<String>>,
    log_file: Arc<RwLock<File>>,
    censor_strings: HashMap<String, String>,
}

impl SharedOutput {
    async fn build(
        log_file_path: &Path,
        censor_strings: HashMap<String, String>,
    ) -> crate::Result<Self> {
        Ok(SharedOutput {
            output: Arc::new(RwLock::new(String::new())),
            log_file: Arc::new(RwLock::new(
                File::create(log_file_path)
                    .await
                    .map_err(|e| IOError::with_path(e, log_file_path))?,
            )),
            censor_strings,
        })
    }

    // Main entry function to a created SharedOutput, returns the log as a String
    pub async fn get_output(&self) -> crate::Result<String> {
        let output = self.output.read().await;
        Ok(output.clone())
    }

    async fn read_stdout(
        &self,
        child_stdout: ChildStdout,
    ) -> crate::Result<()> {
        let mut buf_reader = BufReader::new(child_stdout);
        let mut line = String::new();

        while buf_reader
            .read_line(&mut line)
            .await
            .map_err(IOError::from)?
            > 0
        {
            let val_line = self.censor_log(line.clone());

            {
                let mut output = self.output.write().await;
                output.push_str(&val_line);
            }
            {
                let mut log_file = self.log_file.write().await;
                log_file
                    .write_all(val_line.as_bytes())
                    .await
                    .map_err(IOError::from)?;
            }

            line.clear();
        }
        Ok(())
    }

    async fn read_stderr(
        &self,
        child_stderr: ChildStderr,
    ) -> crate::Result<()> {
        let mut buf_reader = BufReader::new(child_stderr);
        let mut line = String::new();

        while buf_reader
            .read_line(&mut line)
            .await
            .map_err(IOError::from)?
            > 0
        {
            let val_line = self.censor_log(line.clone());

            {
                let mut output = self.output.write().await;
                output.push_str(&val_line);
            }
            {
                let mut log_file = self.log_file.write().await;
                log_file
                    .write_all(val_line.as_bytes())
                    .await
                    .map_err(IOError::from)?;
            }

            line.clear();
        }
        Ok(())
    }

    fn censor_log(&self, mut val: String) -> String {
        for (find, replace) in &self.censor_strings {
            val = val.replace(find, replace);
        }

        val
    }
}
