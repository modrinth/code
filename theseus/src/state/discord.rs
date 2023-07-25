use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use discord_presence::Client;
use tokio::{sync::RwLock, time::sleep};

use crate::State;

pub struct DiscordGuard {
    thread_guard: std::thread::JoinHandle<()>,
    client: Arc<RwLock<discord_presence::Client>>,
    ready: Arc<AtomicBool>,
}

impl DiscordGuard {
    pub async fn init() -> DiscordGuard {
        let mut drpc = Client::new(1084015525241311292);

        let ready = Arc::new(AtomicBool::new(false));
        let ready_copy = Arc::clone(&ready);
        drpc.on_ready(move |_ctx| {
            ready_copy.store(true, Ordering::SeqCst);
        });

        // TODO: We will be adding these later
        // drpc.on_activity_join_request(|ctx| {
        //     println!("Join request: {:?}", ctx.event);
        // });

        // drpc.on_activity_join(|ctx| {
        //     println!("Joined: {:?}", ctx.event);
        // });

        // drpc.on_activity_spectate(|ctx| {
        //     println!("Spectate: {:?}", ctx.event);
        // });

        let drpc = Arc::new(RwLock::new(drpc));

        // Start DRPC connection
        let drpc_loop = Arc::clone(&drpc);
        let drpc_thread = std::thread::spawn(move || loop {
            tracing::debug!("Starting Discord Rich Presence thread");
            let mut drpc = drpc_loop.blocking_write();
            let drpc_thread = drpc.start();
            drop(drpc);
            let _ = drpc_thread.join();
            thread::sleep(Duration::from_secs(1));
        });

        DiscordGuard {
            thread_guard: drpc_thread,
            client: drpc,
            ready,
        }
    }

    pub async fn set_activity<S>(&self, msg: S) -> crate::Result<()>
    where
        S: Into<String>,
    {
        if self.is_ready() {
            let mut client = self.client.write().await;
            client
                .set_activity(|act| {
                    act.state(msg).assets(|ass| {
                        ass.large_image("modrinth_simple")
                            .large_text("Modrinth Logo")
                    })
                })
                .map_err(|_e: discord_presence::DiscordError| {
                    crate::ErrorKind::OtherError(
                        "Could not update Discord activity.".to_string(),
                    )
                })?;
        }
        Ok(())
    }

    pub async fn clear_to_default(&self) -> crate::Result<()> {
        let state = State::get().await?;
        if let Some(existing_child) = state
            .children
            .read()
            .await
            .running_profile_paths()
            .await?
            .first()
        {
            self.set_activity(format!("Playing {}", existing_child))
                .await?;
        } else {
            self.clear_activity().await?;
        }
        Ok(())
    }

    pub async fn clear_activity(&self) -> crate::Result<()> {
        if self.is_ready() {
            let mut client = self.client.write().await;
            client.clear_activity().map_err(
                |_e: discord_presence::DiscordError| {
                    crate::ErrorKind::OtherError(
                        "Could not clear Discord activity.".to_string(),
                    )
                },
            )?;
        }
        Ok(())
    }

    pub async fn wait_until_ready(&self) {
        // drpc.block_until_event(Event::Ready).unwrap(); <- alternative, threadblocking
        while !self.ready.load(Ordering::SeqCst) {
            // Sleep for a short duration.
            sleep(Duration::from_millis(10)).await;
        }
    }

    pub fn is_ready(&self) -> bool {
        if self.thread_guard.is_finished() {
            self.ready.store(false, Ordering::SeqCst)
        }
        self.ready.load(Ordering::SeqCst)
    }

    pub async fn reset(&mut self) {
        *self = DiscordGuard::init().await;
    }
}
