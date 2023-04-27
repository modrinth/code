use crate::event::{
    EventError, LoadingBar, LoadingBarType, ProcessPayloadType,
    ProfilePayloadType,
};
use futures::prelude::*;
use std::path::PathBuf;

#[cfg(feature = "tauri")]
use crate::event::{
    LoadingPayload, ProcessPayload, ProfilePayload, WarningPayload,
};
#[cfg(feature = "tauri")]
use tauri::Manager;
use uuid::Uuid;

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
// title is the title of the loading bar
pub async fn init_loading(
    bar_type: LoadingBarType,
    total: f64,
    title: &str,
) -> crate::Result<Uuid> {
    let event_state = crate::EventState::get().await?;
    let key = Uuid::new_v4();

    event_state.loading_bars.write().await.insert(
        key,
        LoadingBar {
            loading_bar_id: key,
            message: title.to_string(),
            total,
            current: 0.0,
            bar_type,
        },
    );
    // attempt an initial loading_emit event to the frontend
    emit_loading(&key, 0.0, None).await?;
    Ok(key)
}

pub async fn init_or_edit_loading(
    id: Option<Uuid>,
    bar_type: LoadingBarType,
    total: f64,
    title: &str,
) -> crate::Result<Uuid> {
    if let Some(id) = id {
        edit_loading(id, bar_type, total, title).await?;

        Ok(id)
    } else {
        init_loading(bar_type, total, title).await
    }
}

// Edits a loading bar's type
pub async fn edit_loading(
    id: Uuid,
    bar_type: LoadingBarType,
    total: f64,
    title: &str,
) -> crate::Result<()> {
    let event_state = crate::EventState::get().await?;

    if let Some(bar) = event_state.loading_bars.write().await.get_mut(&id) {
        bar.bar_type = bar_type;
        bar.total = total;
        bar.message = title.to_string();
    };

    emit_loading(&id, 0.0, None).await?;
    Ok(())
}

// emit_loading emits a loading event to the frontend
// key refers to the loading bar to update
// increment refers to by what relative increment to the loading struct's total to update
// message is the message to display on the loading bar- if None, use the loading bar's default one
// By convention, fraction is the fraction of the progress bar that is filled
#[allow(unused_variables)]
pub async fn emit_loading(
    key: &Uuid,
    increment_frac: f64,
    message: Option<&str>,
) -> crate::Result<()> {
    let event_state = crate::EventState::get().await?;

    let mut loading_bar = event_state.loading_bars.write().await;
    let loading_bar = match loading_bar.get_mut(key) {
        Some(f) => f,
        None => {
            return Err(EventError::NoLoadingBar(*key).into());
        }
    };

    // Tick up loading bar
    loading_bar.current += increment_frac;
    let display_frac = loading_bar.current / loading_bar.total;
    let display_frac = if display_frac >= 1.0 {
        None // by convention, when its done, we submit None
             // any further updates will be ignored (also sending None)
    } else {
        Some(display_frac)
    };
    // Emit event to tauri
    #[cfg(feature = "tauri")]
    event_state
        .app
        .emit_all(
            "loading",
            LoadingPayload {
                fraction: display_frac,
                message: message.unwrap_or(&loading_bar.message).to_string(),
                event: loading_bar.bar_type.clone(),
                loader_uuid: loading_bar.loading_bar_id,
            },
        )
        .map_err(EventError::from)?;
    Ok(())
}

// emit_warning(message)
#[allow(dead_code)]
#[allow(unused_variables)]
pub async fn emit_warning(message: &str) -> crate::Result<()> {
    #[cfg(feature = "tauri")]
    {
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
    }
    Ok(())
}

// emit_process(uuid, pid, event, message)
#[allow(unused_variables)]
pub async fn emit_process(
    uuid: Uuid,
    pid: u32,
    event: ProcessPayloadType,
    message: &str,
) -> crate::Result<()> {
    #[cfg(feature = "tauri")]
    {
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
    }
    Ok(())
}

// emit_profile(path, event)
#[allow(unused_variables)]
pub async fn emit_profile(
    uuid: Uuid,
    path: PathBuf,
    name: &str,
    event: ProfilePayloadType,
) -> crate::Result<()> {
    #[cfg(feature = "tauri")]
    {
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
    }
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
#[macro_export]
macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + $crate::count!($($xs)*));
}
#[cfg(feature = "tauri")]
#[macro_export]
macro_rules! loading_join {
    ($key:expr, $total:expr, $message:expr; $($task:expr $(,)?)+) => {
        {
            let key = $key;
            let message : Option<&str> = $message;

            let num_futures = $crate::count!($($task)*);
            let increment = $total / num_futures as f64;


            paste::paste! {
                $( let [ <unique_name $task>] = {
                    {
                        let key = key.clone();
                        let message = message.clone();
                        async move {
                            let res = $task.await;
                            if let Some(key) = key {
                                $crate::event::emit::emit_loading(key, increment, message).await?;
                            }
                            res
                        }
                    }
                };)+
            }

            paste::paste! {
                tokio::try_join! (
                    $( [ <unique_name $task>] ),+
                )
            }
        }
    };

}
#[cfg(not(feature = "tauri"))]
#[macro_export]
macro_rules! loading_join {
    ($start:expr, $end:expr, $message:expr; $($future:expr $(,)?)+) => {{
        tokio::try_join!($($future),+)
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
    key: Option<&Uuid>,
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
    _key: Option<&Uuid>,
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
