use crate::api::Result;
use theseus::ErrorKind;

#[derive(Default)]
pub struct PendingUpdateData(());

#[tauri::command]
pub fn get_update_size() -> Result<()> {
    updates_are_disabled()
}

#[tauri::command]
pub fn enqueue_update_for_installation() -> Result<()> {
    updates_are_disabled()
}

fn updates_are_disabled() -> Result<()> {
    let error: theseus::Error = ErrorKind::OtherError(
        "Updates are disabled in this build.".to_string(),
    )
    .into();
    Err(error.into())
}

#[tauri::command]
pub fn remove_enqueued_update() {}
