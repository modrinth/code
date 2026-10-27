use crate::api::Result;
use tauri::plugin::TauriPlugin;
use theseus::prelude::*;

pub fn init<R: tauri::Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("onboarding-checklist")
        .invoke_handler(tauri::generate_handler![
            get_onboarding_checklist,
            skip_create_instance_onboarding
        ])
        .build()
}

#[tauri::command]
pub async fn get_onboarding_checklist() -> Result<OnboardingChecklist> {
    Ok(onboarding_checklist::get().await?)
}

#[tauri::command]
pub async fn skip_create_instance_onboarding() -> Result<()> {
    Ok(onboarding_checklist::mark_created_instance().await?)
}
