use futures::prelude::*;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::async_runtime::RwLock;

#[cfg(feature = "tauri")]
tokio::task_local! {
    pub static WINDOW: tauri::Window;
}

#[derive(Serialize, Clone)]
pub struct LoadingPayload {
    pub fraction: Option<f64>, // by convention, if optional, it means the loading is done
    pub message: String,
}

#[derive(Serialize, Clone)]
pub struct WarningPayload {
    pub message: String,
}

#[derive(Serialize, Clone)]
pub struct ProcessPayload {
    pub uuid: uuid::Uuid, // processes in state are going to be identified by UUIDs, as they might change to different processes
    pub pid: u32,
    pub event: ProcessPayloadType,
    pub message: String,
}
#[derive(Serialize, Clone)]
pub enum ProcessPayloadType {
    Launched,
    Finishing, // TODO: process restructing incoming, currently this is never emitted
    Finished, // TODO: process restructing incoming, currently this is never emitted
}

#[derive(Serialize, Clone)]
pub struct ProfilePayload {
    pub uuid : uuid::Uuid,
    pub path: PathBuf,
    pub name : String,
    pub event: ProfilePayloadType,
}
#[derive(Serialize, Clone)]
pub enum ProfilePayloadType {
    Created,
    Added, // also triggered when Created
    Edited,
    Removed,
}

// window_scoped!(window, task)
// Runs an synchronous function in the tokio task local scope
// All event-related macros used within this asynchronous task,
// no matter how deeply nested will be scoped to the window passed here
#[cfg(feature = "tauri")]
#[macro_export]
macro_rules! window_scoped {
    ($window:expr, $x:expr) => {{
        use tokio::task::LocalKey;
        $crate::WINDOW.scope($window, async move {
            let res = $x.await;

            if let Err(e) = $crate::WINDOW.try_with(|f| {
                f.emit(
                    "loading",
                    $crate::LoadingPayload {
                        fraction: None,
                        message: "Done loading.".to_string(),
                    },
                )
            }) {
                eprintln!("Error emitting loading event to Tauri: {}", e);
            }

            res
        })
    }};
}

// emit_loading(loading_frac, message)
// Passes the a LoadingPayload to the frontend in the window stored by the window_scoped! macro
// By convention, fraction is the fraction of the progress bar that is filled
// This function cannot fail (as the API should be usable without Tauri), but prints to stderr if it does
#[cfg(feature = "tauri")]
pub fn emit_loading(loading_frac: f64, message: &str) {
    println!("Loading: {} ({})", message, loading_frac);
    if let Err(e) = WINDOW.try_with(|f| {
        f.emit(
            "loading",
            LoadingPayload {
                fraction: Some(loading_frac),
                message: message.to_string(),
            },
        )
    }) {
        eprintln!("Error emitting loading event to Tauri: {}", e);
    }
}
#[cfg(not(feature = "tauri"))]
pub fn emit_loading(_loading_frac: f64, _message: &str) {}

// emit_warning(message)
// Passes the a WarningPayload to the frontend in the window stored by the window_scoped! macro
// This function cannot fail (as the API should be usable without Tauri), but prints to stderr if it does
#[cfg(feature = "tauri")]
pub fn emit_warning(message: &str) {
    println!("Warning: {} ", message);
    if let Err(e) = WINDOW.try_with(|f| {
        f.emit(
            "warning",
            WarningPayload {
                message: message.to_string(),
            },
        )
    }) {
        eprintln!("Error emitting warning event to Tauri: {}", e);
    }
}
#[cfg(not(feature = "tauri"))]
pub fn emit_warning(_message: &str) {}

// emit_process(pid, event, message)
// Passes the a ProcessPayload to the frontend in the window stored by the window_scoped! macro
// This function cannot fail (as the API should be usable without Tauri), but prints to stderr if it does
#[cfg(feature = "tauri")]
pub fn emit_process(uuid: uuid::Uuid, pid: u32, event: ProcessPayloadType, message: &str) {
    println!("Process: {} ({})", uuid, message);

    if let Err(e) = WINDOW.try_with(|f| {
        f.emit(
            "process",
            ProcessPayload {
                uuid,
                pid,
                event,
                message: message.to_string(),
            },
        )
    }) {
        eprintln!("Error emitting process event to Tauri: {}", e);
    }
}

#[cfg(not(feature = "tauri"))]
pub fn emit_process(_uuid: uuid::Uuid, _pid: u32, _event: ProcessPayloadType, _message: &str) {}

// emit_profile(path, event)
// Passes the a ProfilePayload to the frontend in the window stored by the window_scoped! macro
// This function cannot fail (as the API should be usable without Tauri), but prints to stderr if it does
#[cfg(feature = "tauri")]
pub fn emit_profile(uuid: uuid::Uuid, path: PathBuf, name: &str, event: ProfilePayloadType) {
    println!("Profile: {} ({})", uuid, name);
    if let Err(e) =
        WINDOW.try_with(|f| f.emit("profile", ProfilePayload { uuid, path, name: name.to_string(), event }))
    {
        eprintln!("Error emitting profile event to Tauri: {}", e);
    }
}

#[cfg(not(feature = "tauri"))]
pub fn emit_profile(_uuid: uuid::Uuid, _path: PathBuf, _event: ProfilePayloadType) {}

// loading_join! macro
// loading_join!(i,j,message; task1, task2, task3...)
// task1, task2, task3 are async tasks that yuo want to to join on await on
// i and j are the start and end of the progress bar over the course of the loading
// Message is the message to display as these futures are loading
// For example, if you want the tasks to range as 0.1, 0.2, 0.3 (of the progress bar), you would do:
// loading_join!(0.0, 0.3; task1, task2, task3)
// This will await on each of the tasks, and as each completes, it will emit a loading event for 0.1, 0.2, 0.3, etc
// This should function as a drop-in replacement for tokio::try_join_all! in most cases- except the function *itself* calls ? rather than needing it.

#[cfg(feature = "tauri")]
#[macro_export]
macro_rules! loading_join {
    ($start:expr, $end:expr, $message:expr; $($future:expr $(,)?)+) => {{
            let range = $end - $start;
            let mut num_futures = 0;
            $(
                {
                    let _ = &$future; // useless to allow matching to $future
                    num_futures += 1;
                }
            )*
            let increment = range / num_futures as f64;

            // Create tokio::pinned values
            $(
                paste::paste! {
                    tokio::pin! {
                        let [<unique_name_ $future>] = $future;
                    }
                }
            )*
            $(
                paste::paste! {
                    let mut [<result_ $future>] = None;
                }
            )*

            // Resolve each future and call respective loading as each resolves in any order
            for i in 0..num_futures {
                paste::paste! {
                    tokio::select! {
                        $(
                            v = &mut [<unique_name_ $future>], if ![<result_$future>].is_some() => {
                                $crate::emit_loading(($start + (i as f64 * increment)), $message);
                                [<result_ $future>] = Some(v);
                            },
                        )*
                        else => break,
                    }
                }
            }

            // Extract values out of option, then out of error, returning if any errors happened
            $(
                paste::paste! {
                    let [<result_ $future>] = [<result_ $future>].take().unwrap()?; // unwrap here acceptable as numbers of futures and resolved values is guaranteed to be the same
                }
            )*

            paste::paste!{
                ($(
                    [<result_ $future>], // unwrap here acceptable as numbers of futures and resolved values is guaranteed to be the same
                )+)
            }
    }};
}

#[cfg(not(feature = "tauri"))]
#[macro_export]
macro_rules! loading_join {
    ($start:expr, $end:expr, $message:expr; $($future:expr $(,)?)+) => {{
        tokio::try_join!($($future),+)
    }};
}

#[cfg(feature = "tauri")]
pub async fn loading_try_for_each_concurrent<I, F, Fut, T>(
    stream: I,
    limit: Option<usize>,
    loading_frac_start: f64,
    loading_frac_end: f64,
    num_futs: usize, // num is in here as we allow Iterator to be passed in, which doesn't have a size
    message: &str,
    f: F,
) -> crate::Result<()>
where
    I: futures::TryStreamExt<Error = crate::Error> + TryStream<Ok = T>,
    F: FnMut(T) -> Fut + Send,
    Fut: Future<Output = crate::Result<()>> + Send,
    T: Send,
{
    let futs_count = Arc::new(RwLock::new(0.0));
    let mut f = f;

    stream
        .try_for_each_concurrent(limit, |item| {
            let f = f(item);
            let futs_count = futs_count.clone();
            async move {
                f.await?;
                let loading_frac = {
                    let mut futs_count = futs_count.write().await;
                    *futs_count += 1.0;
                    (loading_frac_end - loading_frac_start)
                        * (*futs_count / num_futs as f64)
                        + loading_frac_start
                };
                emit_loading(loading_frac, message);
                Ok(())
            }
        })
        .await
}

#[cfg(not(feature = "tauri"))]
pub async fn loading_try_for_each_concurrent<I, F, Fut, T>(
    stream: I,
    limit: Option<usize>,
    _loading_frac_start: f64,
    _loading_frac_end: f64,
    _num_futs: usize, // num is in here as we allow Iterator to be passed in, which doesn't have a size
    _message: &str,
    f: F,
) -> crate::Result<()>
where
    I: futures::TryStreamExt<Error = crate::Error>
        + TryStream<Ok = T>
        + Iterator,
    F: FnMut(T) -> Fut + Send + 'static,
    Fut: Future<Output = crate::Result<()>> + Send + 'static,
    T: Send + 'static,
{
    let mut f = f;
    stream
        .try_for_each_concurrent(limit, |item| {
            let f = f(item);
            async move {
                f.await?;
                Ok(())
            }
        })
        .await
}
