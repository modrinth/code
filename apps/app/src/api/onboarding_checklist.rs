use crate::api::Result;
use tauri::plugin::TauriPlugin;
use theseus::prelude::*;

pub fn init<R: tauri::Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("onboarding-checklist")
        .invoke_handler(tauri::generate_handler![get_onboarding_checklist])
        .build()
}

#[tauri::command]
pub async fn get_onboarding_checklist() -> Result<OnboardingChecklist> {
    Ok(onboarding_checklist::get().await?)
}
