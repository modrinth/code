use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use futures::stream::{self, StreamExt};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{ChildStdout, ChildStderr};

// Child processes (instances of Minecraft)
// A wrapper over a Hashmap connecting PID -> MinecraftChild
pub struct Children(HashMap<u32, Arc<RwLock<MinecraftChild>>>);

// Minecraft Child, bundles together the PID, the actual Child, and the easily queryable stdout and stderr streams
#[derive(Debug)]
pub struct MinecraftChild {
    pub pid: u32,
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
        mut child: tokio::process::Child,
    ) -> Arc<RwLock<MinecraftChild>> {

        // Create std watcher threads for stdout and stderr
        let stdout = SharedOutput::new();
        if let Some(child_stdout) = child.stdout.take() {
            let stdout_clone = stdout.clone();
            tokio::spawn(async move {
                stdout_clone.read_stdout(child_stdout).await.unwrap();
            });
        } 
        let stderr = SharedOutput::new();
        if let Some(child_stderr) = child.stderr.take() {
            let stderr_clone = stderr.clone();
            tokio::spawn(async move {
                stderr_clone.read_stderr(child_stderr).await.unwrap();
            });
        }

        // Create MinecraftChild
        let mchild = MinecraftChild {
            pid,
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
    pub async fn exit_status(&self, pid: &u32) -> crate::Result<Option<std::process::ExitStatus>> {
        if let Some(child) = self.get(pid) {
            let child = child.clone();
            let mut child = child.write().await;
            Ok(child.child.try_wait()?)
        } else {
            Ok(None)
        }
    }

    // Gets all PID keys of running children
    // If an error was collected in accessing the lock/child, that PID is discarded
    pub async fn running_keys(&self) -> Vec<u32> {
        stream::iter(self.0.iter())
            .filter(|(_, child)| {
                let child = child.clone();
                async move { child.write().await.child.try_wait().ok().is_none() }
            })
            .map(|(pid, _)| *pid)
            .collect()
            .await
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
    
    async fn read_stdout(&self, child_stdout: ChildStdout) -> crate::Result<()> {
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

    async fn read_stderr(&self, child_stderr: ChildStderr) -> crate::Result<()> {
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
