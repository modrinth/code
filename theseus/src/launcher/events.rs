use futures::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::async_runtime::RwLock;

/*
    Events are a way we can communciate with the Tauri frontend from the Rust backend.
    We include a feature flag for Tauri, so that we can compile this code without Tauri.

    To use events, we need to do the following:
    1) Use window_scoped!() to wrap a future in which we want to use events. Calling event functions from outside of this future will fail.
    2) For emit_loading() specifically, we need to inialize the loading bar with emit_loading_init() first.
    - If no Window or LoadingBar is initialized, emit_loading() will fail silently. 
    3) Within this scope, you can call emit_x functions to send events to the frontend.

    The window_scoped initializes *task specific* variable, so that it doesn't need to be passed through nested functions and can be separate
    from that passed to any other futures (in case the Tauri frontend gets complicated).
    This way it's also disconnected from the state.

    For example:
    pub async fn do_something_long() {
        window_scoped!(window, loading_function()).await;
    }

    pub async fn loading_function() {
        init_loading("do_func", 100.0, "Loading something long...").await;
        for i in 0..100 {
            emit_loading("do_func", 1.0, None).await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

 */


#[cfg(feature = "tauri")]
tokio::task_local! {
    pub static WINDOW: tauri::Window;

    pub static LOADING_PROGRESS_BARS : Arc<RwLock<HashMap<String, LoadingBar>>>
}

pub struct LoadingBar {
    pub message: String,
    pub total: f64,
    pub current: f64,
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
    pub uuid: uuid::Uuid,
    pub path: PathBuf,
    pub name: String,
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
        use std::collections::HashMap;
        use std::sync::Arc;
        use tauri::async_runtime::RwLock;

        $crate::WINDOW.scope($window, async move {
            $crate::LOADING_PROGRESS_BARS.scope(Arc::new(RwLock::new(HashMap::new())), async move {
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
            }).await
        })
    }};
}

// Initialize a loading bar for use in emit_loading
// Should be called before any uses of emit_loading, or they will fail.
// key is the key to use to refer to this loading bar
// total is the total amount of work to be done- all emissions will be considered a fraction of this value (should be 1 or 100 for simplicity)
// default_message is the message to display on the loading bar if no message is passed to emit_loading
// This function will fail silently if not called from within a window_scoped!() future
#[cfg(feature = "tauri")]
pub async fn init_loading(
    key: &str,
    total: f64,
    default_message: &str,
) {
    // Create a new entry in the local thread Loading progress bar map
    let loading_bars_clone_ref = match LOADING_PROGRESS_BARS.try_with(|f| f.clone()){
        Ok(f) => f,
        Err(e) => {
            eprintln!("Could not initialize loading '{key}', not inside window scope: {}", e);
            return;
        }
    };
    let mut loading_bar = loading_bars_clone_ref.write().await;
    loading_bar.insert(
        key.to_string(),
        LoadingBar {
            message: default_message.to_string(),
            total,
            current: 0.0,
        },
    );

    // attempt an initial loading_emit event to the frontend
    emit_loading(key, 0.0, None).await;
}

#[cfg(not(feature = "tauri"))]
pub async fn init_loading(
    key: &str,
    total: f64,
    default_message: &str,
) -> Result<(), EventError> {
    Ok(())
}

// emit_loading(key : &str, increment_frac: f64, message: Option<&str>)
// Passes the a LoadingPayload to the frontend in the window stored by the window_scoped macro
// key refers to the loading bar to update
// increment refers to by what relative increment to the loading struct's total to update
// message is the message to display on the loading bar- if None, use the loading bar's default one
// By convention, fraction is the fraction of the progress bar that is filled
// This function cannot fail (as the API should be usable without Tauri), but prints to stderr if it does
#[cfg(feature = "tauri")]
pub async fn emit_loading(key : &str, increment_frac: f64, message: Option<&str>) {
    let loading_bars_clone_ref = match LOADING_PROGRESS_BARS.try_with(|f| f.clone()) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Could not emit loading message '{key}', not inside window scope: {}", e);
            return;
        }
    };
    let mut loading_bar = loading_bars_clone_ref.write().await;
    let loading_bar = match loading_bar.get_mut(key) {
        Some(f) => f,
        None => {
            eprintln!("Could not emit loading '{key}', no loading bar recognized by that key",);
            return;
        }
    };

    if let Err(e) = WINDOW.try_with(|window| {
        // Tick up loading bar
        loading_bar.current += increment_frac;
        let display_frac = loading_bar.current / loading_bar.total;
        let display_frac = if display_frac > 1.0 {
            None // by convention, when its done, we submit None
            // any further updates will be ignored (also sending None)
        } else {
            Some(display_frac)
        };
        // Emit event to tauri
        window.emit(
            "loading",
            LoadingPayload {
                fraction: display_frac,
                message: message.unwrap_or(&loading_bar.message).to_string(),
            },
        )
    }) {
        eprintln!("Could not emit loading '{key}', not inside window scope: {}", e);
        return;
    }
}

#[cfg(not(feature = "tauri"))]
pub fn emit_loading(_key : &str, _increment_frac: f64, _message: Option<&str>) {}

// emit_warning(message)
// Passes the a WarningPayload to the frontend in the window stored by the window_scoped macro
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
// Passes the a ProcessPayload to the frontend in the window stored by the window_scoped macro
// This function cannot fail (as the API should be usable without Tauri), but prints to stderr if it does
#[cfg(feature = "tauri")]
pub fn emit_process(
    uuid: uuid::Uuid,
    pid: u32,
    event: ProcessPayloadType,
    message: &str,
) {
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
pub fn emit_process(
    _uuid: uuid::Uuid,
    _pid: u32,
    _event: ProcessPayloadType,
    _message: &str,
) {
}

// emit_profile(path, event)
// Passes the a ProfilePayload to the frontend in the window stored by the window_scoped macro
// This function cannot fail (as the API should be usable without Tauri), but prints to stderr if it does
#[cfg(feature = "tauri")]
pub fn emit_profile(
    uuid: uuid::Uuid,
    path: PathBuf,
    name: &str,
    event: ProfilePayloadType,
) {
    println!("Profile: {} ({})", uuid, name);
    if let Err(e) = WINDOW.try_with(|f| {
        f.emit(
            "profile",
            ProfilePayload {
                uuid,
                path,
                name: name.to_string(),
                event,
            },
        )
    }) {
        eprintln!("Error emitting profile event to Tauri: {}", e);
    }
}

#[cfg(not(feature = "tauri"))]
pub fn emit_profile(
    _uuid: uuid::Uuid,
    _path: PathBuf,
    _event: ProfilePayloadType,
) {
}

// loading_join! macro
// loading_join!(key: &str, total: f64, message: Option<&str>; task1, task2, task3...)
// This will submit a loading event with the given message for each task as they complete
// It must be called within a window_scoped macro asynchronous task
// task1, task2, task3 are async tasks that yuo want to to join on await on
// Key is the key to use for which loading bar
// Total is the total amount of progress that the loading bar should take up by all futures in this (will be split evenly amongst them).
// If message is Some(t) you will overwrite this loading bar's message with a custom one
// For example, if you want the tasks to range as 0.1, 0.2, 0.3 (of the progress bar), you would do:
// loading_join!('loading files', 0.1; task1, task2, task3)
// This will await on each of the tasks, and as each completes, it will emit a loading event for 0.033, 0.066, 0.099, etc
// This should function as a drop-in replacement for tokio::try_join_all! in most cases- except the function *itself* calls ? rather than needing it.

#[cfg(feature = "tauri")]
#[macro_export]
macro_rules! loading_join {
    ($key:expr, $total:expr, $message:expr; $($future:expr $(,)?)+) => {{
            let mut num_futures = 0;
            $(
                {
                    let _ = &$future; // useless to allow matching to $future
                    num_futures += 1;
                }
            )*
            let increment = $total / num_futures as f64;

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
            for _ in 0..num_futures {
                paste::paste! {
                    tokio::select! {
                        $(
                            v = &mut [<unique_name_ $future>], if ![<result_$future>].is_some() => {
                                $crate::emit_loading($key, increment, $message).await;
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
    key: &str,
    total: f64,
    num_futs: usize, // num is in here as we allow Iterator to be passed in, which doesn't have a size
    message: Option<&str>,
    f: F,
) -> crate::Result<()>
where
    I: futures::TryStreamExt<Error = crate::Error> + TryStream<Ok = T>,
    F: FnMut(T) -> Fut + Send,
    Fut: Future<Output = crate::Result<()>> + Send,
    T: Send,
{
    let mut f = f;
    stream
        .try_for_each_concurrent(limit, |item| {
            let f = f(item);
            async move {
                f.await?;
                emit_loading(key, total/(num_futs as f64), message).await;
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

#[derive(Debug, thiserror::Error)]
pub enum EventError {
    // TauriError(tauri::Error),
    // JsonError(serde_json::Error),
    #[error("No window found to emit event to in this task")]
    NoWindow,

    #[error("No loading bars stored in this task")]
    NoLoadingBars,

    #[error("Non-existent loading bar of key: {0}")]
    NoLoadingBar(String),

    #[error("Tauri error: {0}")]
    TauriError(#[from] tauri::Error)
}