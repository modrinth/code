use crate::event::{
    EventError, LoadingBar, LoadingBarId, LoadingBarType, LoadingPayload,
    ProcessPayload, ProcessPayloadType, ProfilePayload, ProfilePayloadType,
    WarningPayload,
};
use futures::prelude::*;
use std::path::PathBuf;

/*
   Events are a way we can communciate with the Tauri frontend from the Rust backend.
   We include a feature flag for Tauri, so that we can compile this code without Tauri.

   To use events, we need to do the following:
    1) Make sure we are using the tauri feature flag
    2) Initialize the EventState with EventState::init() *before* initializing the theseus State
    3) Call emit_x functions to send events to the frontend
   For emit_loading() specifically, we need to inialize the loading bar with init_loading() first and pass the received loader in

   For example:
   pub async fn loading_function() -> crate::Result<()> {
        loading_function()).await;
   }

   pub async fn loading_function() -> crate::Result<()> {
       let loading_bar = init_loading(LoadingBarType::StateInit, 100.0, "Loading something long...").await;
       for i in 0..100 {
           emit_loading(&loading_bar, 1.0, None).await?;
           tokio::time::sleep(Duration::from_millis(100)).await;
       }
   }
*/

// Initialize a loading bar for use in emit_loading
// This will generate a LoadingBarId, which is used to refer to the loading bar uniquely.
// total is the total amount of work to be done- all emissions will be considered a fraction of this value (should be 1 or 100 for simplicity)
// default_message is the message to display on the loading bar if no message is passed to emit_loading
#[cfg(feature = "tauri")]
pub async fn init_loading(
    bar_type: LoadingBarType,
    total: f64,
    default_message: &str,
) -> crate::Result<LoadingBarId> {
    let event_state = crate::EventState::get().await?;
    let key = LoadingBarId::new(bar_type);

    event_state.loading_bars.write().await.insert(
        key.clone(),
        LoadingBar {
            loading_bar_uuid: key.clone(),
            message: default_message.to_string(),
            total,
            current: 0.0,
        },
    );
    // attempt an initial loading_emit event to the frontend
    emit_loading(&key, 0.0, None).await?;
    Ok(key)
}

#[cfg(not(feature = "tauri"))]
pub async fn init_loading(
    bar_type: LoadingBarType,
    total: f64,
    default_message: &str,
) -> crate::Result<LoadingBarId> {
    let event_state = crate::EventState::get().await?;
    let key = LoadingBarId::new(bar_type);

    event_state.loading_bars.write().await.insert(
        key.clone(),
        LoadingBar {
            loading_bar_uuid: key.clone(),
            message: default_message.to_string(),
            total,
            current: 0.0,
        },
    );
    Ok(key)
}

// emit_loading emits a loading event to the frontend
// key refers to the loading bar to update
// increment refers to by what relative increment to the loading struct's total to update
// message is the message to display on the loading bar- if None, use the loading bar's default one
// By convention, fraction is the fraction of the progress bar that is filled
#[cfg(feature = "tauri")]
pub async fn emit_loading(
    key: &LoadingBarId,
    increment_frac: f64,
    message: Option<&str>,
) -> crate::Result<()> {
    use tauri::Manager;

    let event_state = crate::EventState::get().await?;

    let mut loading_bar = event_state.loading_bars.write().await;
    let loading_bar = match loading_bar.get_mut(key) {
        Some(f) => f,
        None => {
            return Err(EventError::NoLoadingBar(key.clone()).into());
        }
    };

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
    event_state
        .app
        .emit_all(
            "loading",
            LoadingPayload {
                fraction: display_frac,
                message: message.unwrap_or(&loading_bar.message).to_string(),
                event_type: key.key,
                loader_uuid: key.uuid,
            },
        )
        .map_err(EventError::from)?;
    Ok(())
}

#[cfg(not(feature = "tauri"))]
pub async fn emit_loading(
    key: &LoadingBarId,
    increment_frac: f64,
    message: Option<&str>,
) -> crate::Result<()> {
    let event_state = crate::EventState::get().await?;

    let mut loading_bar = event_state.loading_bars.write().await;
    let loading_bar = match loading_bar.get_mut(key) {
        Some(f) => f,
        None => {
            return Err(EventError::NoLoadingBar(key.clone()).into());
        }
    };

    // Tick up loading bar
    loading_bar.current += increment_frac;
    Ok(())
}

// emit_warning(message)
#[allow(dead_code)]
#[cfg(feature = "tauri")]
pub async fn emit_warning(message: &str) -> crate::Result<()> {
    use tauri::Manager;

    let event_state = crate::EventState::get().await?;
    event_state
        .app
        .emit_all(
            "warning",
            WarningPayload {
                message: message.to_string(),
            },
        )
        .map_err(EventError::from)?;
    Ok(())
}
#[allow(dead_code)]
#[cfg(not(feature = "tauri"))]
pub async fn emit_warning(_message: &str) -> crate::Result<()> {
    Ok(())
}

// emit_process(pid, event, message)
#[cfg(feature = "tauri")]
pub async fn emit_process(
    uuid: uuid::Uuid,
    pid: u32,
    event: ProcessPayloadType,
    message: &str,
) -> crate::Result<()> {
    use tauri::Manager;

    let event_state = crate::EventState::get().await?;
    event_state
        .app
        .emit_all(
            "process",
            ProcessPayload {
                uuid,
                pid,
                event,
                message: message.to_string(),
            },
        )
        .map_err(EventError::from)?;
    Ok(())
}

#[cfg(not(feature = "tauri"))]
pub async fn emit_process(
    _uuid: uuid::Uuid,
    _pid: u32,
    _event: ProcessPayloadType,
    _message: &str,
) -> crate::Result<()> {
    Ok(())
}

// emit_profile(path, event)
#[cfg(feature = "tauri")]
pub async fn emit_profile(
    uuid: uuid::Uuid,
    path: PathBuf,
    name: &str,
    event: ProfilePayloadType,
) -> crate::Result<()> {
    use tauri::Manager;

    let event_state = crate::EventState::get().await?;
    event_state
        .app
        .emit_all(
            "profile",
            ProfilePayload {
                uuid,
                path,
                name: name.to_string(),
                event,
            },
        )
        .map_err(EventError::from)?;
    Ok(())
}

#[cfg(not(feature = "tauri"))]
pub async fn emit_profile(
    _uuid: uuid::Uuid,
    _path: PathBuf,
    _name: &str,
    _event: ProfilePayloadType,
) -> crate::Result<()> {
    Ok(())
}

// loading_join! macro
// loading_join!(key: Option<&LoadingBarId>, total: f64, message: Option<&str>; task1, task2, task3...)
// This will submit a loading event with the given message for each task as they complete
// task1, task2, task3 are async tasks that yuo want to to join on await on
// Key is the key to use for which loading bar to submit these results to- a LoadingBarId. If None, it does nothing
// Total is the total amount of progress that the loading bar should take up by all futures in this (will be split evenly amongst them).
// If message is Some(t) you will overwrite this loading bar's message with a custom one
// For example, if you want the tasks to range as 0.1, 0.2, 0.3 (of the progress bar), you would do:
// loading_join!(loading_bar, 0.1; task1, task2, task3)
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
                                if let Some(key) = $key {
                                    $crate::event::emit::emit_loading(key, increment, $message).await?;
                                }
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
        tokio::try_join!($($future),+)?
    }};
}

// A drop in replacement to try_for_each_concurrent that emits loading events as it goes
// Key is the key to use for which loading bar- a LoadingBarId. If None, does nothing
// Total is the total amount of progress that the loading bar should take up by all futures in this (will be split evenly amongst them).
// If message is Some(t) you will overwrite this loading bar's message with a custom one
// num_futs is the number of futures that will be run, which is needed as we allow Iterator to be passed in, which doesn't have a size
#[cfg(feature = "tauri")]
pub async fn loading_try_for_each_concurrent<I, F, Fut, T>(
    stream: I,
    limit: Option<usize>,
    key: Option<&LoadingBarId>,
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
                if let Some(key) = key {
                    emit_loading(key, total / (num_futs as f64), message)
                        .await?;
                }
                Ok(())
            }
        })
        .await
}

#[cfg(not(feature = "tauri"))]
pub async fn loading_try_for_each_concurrent<I, F, Fut, T>(
    stream: I,
    limit: Option<usize>,
    _key: Option<&LoadingBarId>,
    _total: f64,
    _num_futs: usize, // num is in here as we allow Iterator to be passed in, which doesn't have a size
    _message: Option<&str>,
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
                Ok(())
            }
        })
        .await
}
