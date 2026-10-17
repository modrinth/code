use crate::api::Result;
use anyhow::anyhow;

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
    Err(anyhow!("updates are disabled in this build").into())
}

#[tauri::command]
pub fn remove_enqueued_update() {}
