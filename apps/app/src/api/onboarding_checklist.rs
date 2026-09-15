use crate::api::Result;
use tauri::plugin::TauriPlugin;
use theseus::prelude::*;

pub fn init<R: tauri::Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("onboarding-checklist")
        .invoke_handler(tauri::generate_handler![
            get_onboarding_checklist,
            mark_logged_into_owyx_site
        ])
        .build()
}

#[tauri::command]
pub async fn get_onboarding_checklist() -> Result<OnboardingChecklist> {
    Ok(onboarding_checklist::get().await?)
}

/// Marks the site-account onboarding step (same DB flag as legacy Modrinth login).
#[tauri::command]
pub async fn mark_logged_into_owyx_site() -> Result<()> {
    Ok(onboarding_checklist::mark_logged_into_modrinth().await?)
}
