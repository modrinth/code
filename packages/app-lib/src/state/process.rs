use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde::Serialize;
use tokio::process::Command;

use crate::event::emit::emit_process;
use crate::event::ProcessPayloadType;
use crate::util::io::IOError;
use crate::{profile, ErrorKind};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Process {
    pub pid: i64,
    pub start_time: i64,
    pub name: String,
    pub executable: String,
    pub profile_path: String,
    pub post_exit_command: Option<String>,
}

macro_rules! select_process_with_predicate {
    ($predicate:tt, $param:ident) => {
        sqlx::query_as!(
            Process,
            r#"
            SELECT
                pid, start_time, name, executable, profile_path, post_exit_command
            FROM processes
            "#
                + $predicate,
            $param
        )
    };
}

impl Process {
    /// Runs on launcher startup. Queries all the cached processes and removes processes that no
    /// longer exist. If a PID is found, they are "rescued" and passed to our process manager
    pub async fn garbage_collect(
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
    ) -> crate::Result<()> {
        let processes = Self::get_all(exec).await?;

        let mut system = sysinfo::System::new();
        system.refresh_processes();
        for cached_process in processes {
            let process = system
                .process(sysinfo::Pid::from_u32(cached_process.pid as u32));

            if let Some(process) = process {
                if cached_process.start_time as u64 == process.start_time()
                    && cached_process.name == process.name()
                    && cached_process.executable
                        == process
                            .exe()
                            .map(|x| x.to_string_lossy())
                            .unwrap_or_default()
                {
                    tokio::spawn(cached_process.sequential_process_manager());

                    break;
                }
            }

            Self::remove(cached_process.pid as u32, exec).await?;
        }

        Ok(())
    }

    pub async fn insert_new_process(
        profile_path: &str,
        mut mc_command: Command,
        post_exit_command: Option<String>, // Command to run after minecraft.
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<Self> {
        let mc_proc = mc_command.spawn().map_err(IOError::from)?;

        let pid = mc_proc.id().ok_or_else(|| {
            crate::ErrorKind::LauncherError(
                "Process immediately failed, could not get PID".to_string(),
            )
        })?;

        let mut system = sysinfo::System::new();
        system.refresh_processes();
        let process =
            system.process(sysinfo::Pid::from_u32(pid)).ok_or_else(|| {
                crate::ErrorKind::LauncherError(format!(
                    "Could not find process {}",
                    pid
                ))
            })?;
        let start_time = process.start_time();
        let name = process.name().to_string();

        let Some(path) = process.exe() else {
            return Err(ErrorKind::LauncherError(format!(
                "Cached process {} has no accessible path",
                pid
            ))
            .into());
        };

        let executable = path.to_string_lossy().to_string();

        let process = Self {
            pid: pid as i64,
            start_time: start_time as i64,
            name,
            executable,
            profile_path: profile_path.to_string(),
            post_exit_command,
        };
        process.upsert(exec).await?;

        tokio::spawn(process.clone().sequential_process_manager());

        emit_process(
            profile_path,
            pid,
            ProcessPayloadType::Launched,
            "Launched Minecraft",
        )
        .await?;

        Ok(process)
    }

    // Spawns a new child process and inserts it into the hashmap
    // Also, as the process ends, it spawns the follow-up process if it exists
    // By convention, ExitStatus is last command's exit status, and we exit on the first non-zero exit status
    async fn sequential_process_manager(self) -> crate::Result<i32> {
        async fn update_playtime(
            last_updated_playtime: &mut DateTime<Utc>,
            profile_path: &str,
            force_update: bool,
        ) {
            let diff = Utc::now()
                .signed_duration_since(*last_updated_playtime)
                .num_seconds();
            if diff >= 60 || force_update {
                if let Err(e) = profile::edit(profile_path, |prof| {
                    prof.recent_time_played += diff as u64;
                    async { Ok(()) }
                })
                .await
                {
                    tracing::warn!(
                        "Failed to update playtime for profile {}: {}",
                        &profile_path,
                        e
                    );
                }
                *last_updated_playtime = Utc::now();
            }
        }

        // Wait on current Minecraft Child
        let mc_exit_status;
        let mut last_updated_playtime = Utc::now();

        loop {
            if let Some(t) = self.try_wait().await? {
                mc_exit_status = t;
                break;
            }
            // sleep for 10ms
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

            // Auto-update playtime every minute
            update_playtime(
                &mut last_updated_playtime,
                &self.profile_path,
                false,
            )
            .await;
        }

        // Now fully complete- update playtime one last time
        update_playtime(&mut last_updated_playtime, &self.profile_path, true)
            .await;

        // Publish play time update
        // Allow failure, it will be stored locally and sent next time
        // Sent in another thread as first call may take a couple seconds and hold up process ending
        let profile_path = self.profile_path.clone();
        tokio::spawn(async move {
            if let Err(e) =
                profile::try_update_playtime(&profile_path.clone()).await
            {
                tracing::warn!(
                    "Failed to update playtime for profile {}: {}",
                    &profile_path,
                    e
                );
            }
        });

        let state = crate::State::get().await?;
        let _ = state.discord_rpc.clear_to_default(true).await;

        Self::remove(self.pid as u32, &state.pool).await?;

        // If in tauri, window should show itself again after process exists if it was hidden
        #[cfg(feature = "tauri")]
        {
            let window = crate::EventState::get_main_window().await?;
            if let Some(window) = window {
                window.unminimize()?;
            }
        }

        if mc_exit_status == 0 {
            // We do not wait on the post exist command to finish running! We let it spawn + run on its own.
            // This behaviour may be changed in the future
            if let Some(hook) = self.post_exit_command {
                let mut cmd = hook.split(' ');
                if let Some(command) = cmd.next() {
                    let mut command = Command::new(command);
                    command.args(&cmd.collect::<Vec<&str>>()).current_dir(
                        crate::api::profile::get_full_path(&self.profile_path)
                            .await?,
                    );
                    command.spawn().map_err(IOError::from)?;
                }
            }
        }

        emit_process(
            &self.profile_path,
            self.pid as u32,
            ProcessPayloadType::Finished,
            "Exited process",
        )
        .await?;

        Ok(mc_exit_status)
    }

    async fn try_wait(&self) -> crate::Result<Option<i32>> {
        let mut system = sysinfo::System::new();
        if !system.refresh_process(sysinfo::Pid::from_u32(self.pid as u32)) {
            return Ok(Some(0));
        }

        let process = system.process(sysinfo::Pid::from_u32(self.pid as u32));

        if let Some(process) = process {
            if process.status() == sysinfo::ProcessStatus::Run {
                Ok(None)
            } else {
                Ok(Some(0))
            }
        } else {
            Ok(Some(0))
        }
    }

    pub async fn wait_for(&self) -> crate::Result<()> {
        loop {
            if self.try_wait().await?.is_some() {
                break;
            }
            // sleep for 10ms
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        Ok(())
    }

    pub async fn kill(&self) -> crate::Result<()> {
        let mut system = sysinfo::System::new();
        if system.refresh_process(sysinfo::Pid::from_u32(self.pid as u32)) {
            let process =
                system.process(sysinfo::Pid::from_u32(self.pid as u32));
            if let Some(process) = process {
                process.kill();
            }
        }

        Ok(())
    }

    pub async fn get(
        pid: i32,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<Option<Self>> {
        let res = select_process_with_predicate!("WHERE pid = $1", pid)
            .fetch_optional(exec)
            .await?;

        Ok(res)
    }

    pub async fn get_from_profile(
        profile_path: &str,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<Vec<Self>> {
        let results = select_process_with_predicate!(
            "WHERE profile_path = $1",
            profile_path
        )
        .fetch_all(exec)
        .await?;

        Ok(results)
    }

    pub async fn get_all(
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<Vec<Self>> {
        let true_val = 1;
        let results = select_process_with_predicate!("WHERE 1=$1", true_val)
            .fetch_all(exec)
            .await?;

        Ok(results)
    }

    pub async fn upsert(
        &self,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<()> {
        sqlx::query!(
            "
            INSERT INTO processes (pid, start_time, name, executable, profile_path, post_exit_command)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (pid) DO UPDATE SET
                start_time = $2,
                name = $3,
                executable = $4,
                profile_path = $5,
                post_exit_command = $6
            ",
            self.pid,
            self.start_time,
            self.name,
            self.executable,
            self.profile_path,
            self.post_exit_command
        )
            .execute(exec)
            .await?;

        Ok(())
    }

    pub async fn remove(
        pid: u32,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<()> {
        let pid = pid as i32;

        sqlx::query!(
            "
            DELETE FROM processes WHERE pid = $1
            ",
            pid,
        )
        .execute(exec)
        .await?;

        Ok(())
    }
}
