use std::path::PathBuf;

use serde::Serialize;

tokio::task_local! {
    pub static WINDOW: tauri::Window;
}

#[derive(Serialize, Clone)]
pub struct LoadingPayload {
    pub fraction: f64,
    pub message: String,
}

#[derive(Serialize, Clone)]
pub struct WarningPayload {
    pub message: String,
}

#[derive(Serialize, Clone)]
pub struct ProcessPayload {
    pub pid : u32,
    pub event: ProcessPayloadType,
    pub message: String,
}
#[derive(Serialize, Clone)]
pub enum ProcessPayloadType {
    Launched,
    Finished
}

#[derive(Serialize, Clone)]
pub struct ProfilePayload {
    pub path: PathBuf,
    pub event: ProfilePayloadType,
}
#[derive(Serialize, Clone)]
pub enum ProfilePayloadType {
    Created,
    Added, // also triggered when Created
    Changed,
    Deleted
}

// window_scoped!(window, task)
// Runs an synchronous function in the tokio task local scope
// All event-related macros used within this asynchronous task,
// no matter how deeply nested will be scoped to the window passed here

#[macro_export]
macro_rules! window_scoped {
    ($window:expr, $x:expr) => {{
        use tokio::task::LocalKey;        
        $crate::WINDOW.scope($window, async move {
            $x.await
        })
    }};
}

// emit_loading(loading_frac, message)
// Passes the a LoadingPayload to the frontend in the window stored by the window_scoped! macro
// By convention, fraction is the fraction of the progress bar that is filled
// This function cannot fail (as the API should be usable without Tauri), but prints to stderr if it does
pub fn emit_loading(loading_frac : f64, message : &str) {
    if let Err(e) = WINDOW.with(|f| { 
        f.emit("loading", LoadingPayload {
            fraction: loading_frac,
            message: message.to_string(),
        }) 
    }) {
        eprintln!("Error emitting loading event to Tauri: {}", e);
    }
}

// emit_warning(message)
// Passes the a WarningPayload to the frontend in the window stored by the window_scoped! macro
// This function cannot fail (as the API should be usable without Tauri), but prints to stderr if it does
pub fn emit_warning(message : &str) {
    if let Err(e) = WINDOW.with(|f| { 
        f.emit("warning", WarningPayload {
            message: message.to_string(),
        }) 
    }) {
        eprintln!("Error emitting warning event to Tauri: {}", e);
    }
}

// emit_process(pid, event, message)
// Passes the a ProcessPayload to the frontend in the window stored by the window_scoped! macro
// This function cannot fail (as the API should be usable without Tauri), but prints to stderr if it does
pub fn emit_process(pid : u32, event : ProcessPayloadType, message : &str) {
    if let Err(e) = WINDOW.with(|f| { 
        f.emit("process", ProcessPayload {
            pid,
            event,
            message: message.to_string(),
        }) 
    }) {
        eprintln!("Error emitting process event to Tauri: {}", e);
    }
}

// emit_profile(path, event)
// Passes the a ProfilePayload to the frontend in the window stored by the window_scoped! macro
// This function cannot fail (as the API should be usable without Tauri), but prints to stderr if it does
pub fn emit_profile(path : PathBuf, event : ProfilePayloadType) {
    if let Err(e) = WINDOW.with(|f| { 
        f.emit("profile", ProfilePayload {
            path,
            event,
        }) 
    }) {
        eprintln!("Error emitting profile event to Tauri: {}", e);
    }
}

// loading_join! macro
// loading_join!(i,j,message; task1, task2, task3...)
// task1, task2, task3 are async tasks that yuo want to to join on await on
// i and j are the start and end of the progress bar over the course of the loading
// Message is the message to display as these futures are loading
// For example, if you want the tasks to range as 0.1, 0.2, 0.3 (of the progress bar), you would do:
// loading_join!(0.0, 0.3; task1, task2, task3)
// This will await on each of the tasks, and as each completes, it will emit a loading event for 0.1, 0.2, 0.3, etc
// This should function as a drop-in replacement for tokio::try_join_all! in most cases
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
                    let mut [<done_ $future>] = false;
                }
            )*

            // Resolve each future and call respective loading as each resolves in any order
            let mut resolved_values = vec![];
            for i in 0..num_futures {
                paste::paste! {
                    tokio::select! {
                        $(
                            v = &mut [<unique_name_ $future>], if ![<done_$future>] => {
                                [<done_ $future>] = true;
                                $crate::emit_loading(($start + (i as f64 * increment)), $message);
                                resolved_values.push(v)
                            },
                        )*
                        else => break,
                    }
                }
            }

            // Generate results as tuple
            let mut resolved_iter = resolved_values.into_iter();
            (
                $(
                    {
                        let _ = $future;
                        resolved_iter.next().unwrap() // uwnrap here acceptable as numbers of futures and resolved values is guaranteed to be the same
                    },
                )*
            )
    }};
}

