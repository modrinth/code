use std::path::{Path, PathBuf};
use std::{collections::HashMap, sync::Arc};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{ChildStderr, ChildStdout};
use tokio::sync::RwLock;

use crate::emit_process;

use super::Profile;

// Child processes (instances of Minecraft)
// A wrapper over a Hashmap connecting PID -> MinecraftChild
pub struct Children(HashMap<u32, Arc<RwLock<MinecraftChild>>>);

// Minecraft Child, bundles together the PID, the actual Child, and the easily queryable stdout and stderr streams
#[derive(Debug)]
pub struct MinecraftChild {
    pub uuid: uuid::Uuid,
    pub pid: u32,
    pub profile_path: PathBuf, //todo: make UUID when profiles are recognized by UUID
    pub child: tokio::process::Child,
    pub stdout: SharedOutput,
    pub stderr: SharedOutput,
}

impl Children {
    pub fn new() -> Children {
        Children(HashMap::new())
    }

    // Inserts a child process to keep track of, and returns a reference to the container struct MinecraftChild
    // The threads for stdout and stderr are spawned here
    // Unlike a Hashmap's 'insert', this directly returns the reference to the Child rather than any previously stored Child that may exist
    pub fn insert_process(
        &mut self,
        pid: u32,
        profile_path: PathBuf,
        mut child: tokio::process::Child,
    ) -> Arc<RwLock<MinecraftChild>> {
        let uuid = uuid::Uuid::new_v4();

        // Create std watcher threads for stdout and stderr
        let stdout = SharedOutput::new();
        if let Some(child_stdout) = child.stdout.take() {
            let stdout_clone = stdout.clone();
            tokio::spawn(async move {
                if let Err(e) = stdout_clone.read_stdout(child_stdout).await {
                    eprintln!("Stdout process died with error: {}", e);
                }
            });
        }
        let stderr = SharedOutput::new();
        if let Some(child_stderr) = child.stderr.take() {
            let stderr_clone = stderr.clone();
            tokio::spawn(async move {
                if let Err(e) = stderr_clone.read_stderr(child_stderr).await {
                    eprintln!("Stderr thread died with error: {}", e);
                }
            });
        }

        emit_process(
            uuid,
            pid,
            crate::ProcessPayloadType::Launched,
            "Launched Minecraft",
        );

        // Create MinecraftChild
        let mchild = MinecraftChild {
            uuid,
            pid,
            profile_path,
            child,
            stdout,
            stderr,
        };
        let mchild = Arc::new(RwLock::new(mchild));
        self.0.insert(pid, mchild.clone());
        mchild
    }

    // Returns a ref to the child
    pub fn get(&self, pid: &u32) -> Option<Arc<RwLock<MinecraftChild>>> {
        self.0.get(pid).cloned()
    }

    // Gets all PID keys
    pub fn keys(&self) -> Vec<u32> {
        self.0.keys().cloned().collect()
    }

    // Get exit status of a child by PID
    // Returns None if the child is still running
    pub async fn exit_status(
        &self,
        pid: &u32,
    ) -> crate::Result<Option<std::process::ExitStatus>> {
        if let Some(child) = self.get(pid) {
            let child = child.clone();
            let mut child = child.write().await;
            Ok(child.child.try_wait()?)
        } else {
            Ok(None)
        }
    }

    // Gets all PID keys of running children
    pub async fn running_keys(&self) -> crate::Result<Vec<u32>> {
        let mut keys = Vec::new();
        for key in self.keys() {
            if let Some(child) = self.get(&key) {
                let child = child.clone();
                let mut child = child.write().await;
                if child.child.try_wait()?.is_none() {
                    keys.push(key);
                }
            }
        }
        Ok(keys)
    }

    // Gets all PID keys of running children with a given profile path
    pub async fn running_keys_with_profile(
        &self,
        profile_path: &Path,
    ) -> crate::Result<Vec<u32>> {
        let running_keys = self.running_keys().await?;
        let mut keys = Vec::new();
        for key in running_keys {
            if let Some(child) = self.get(&key) {
                let child = child.clone();
                let child = child.read().await;
                if child.profile_path == profile_path {
                    keys.push(key);
                }
            }
        }
        Ok(keys)
    }

    // Gets all profiles of running children
    pub async fn running_profile_paths(&self) -> crate::Result<Vec<PathBuf>> {
        let mut profiles = Vec::new();
        for key in self.keys() {
            if let Some(child) = self.get(&key) {
                let child = child.clone();
                let mut child = child.write().await;
                if child.child.try_wait()?.is_none() {
                    profiles.push(child.profile_path.clone());
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
                let mut child = child.write().await;
                if child.child.try_wait()?.is_none() {
                    if let Some(prof) =
                        crate::api::profile::get(&child.profile_path.clone())
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
#[derive(Clone, Debug)]
pub struct SharedOutput {
    output: Arc<RwLock<String>>,
}

impl SharedOutput {
    fn new() -> Self {
        SharedOutput {
            output: Arc::new(RwLock::new(String::new())),
        }
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

        while buf_reader.read_line(&mut line).await? > 0 {
            {
                let mut output = self.output.write().await;
                output.push_str(&line);
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

        while buf_reader.read_line(&mut line).await? > 0 {
            {
                let mut output = self.output.write().await;
                output.push_str(&line);
            }
            line.clear();
        }
        Ok(())
    }
}
