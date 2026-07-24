use crate::api::Result;
use theseus::reports::{CreateReportRequest, CreateReportResponse};

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("reports")
        .invoke_handler(tauri::generate_handler![reports_create])
        .build()
}

#[tauri::command]
pub async fn reports_create(
    request: CreateReportRequest,
) -> Result<CreateReportResponse> {
    Ok(theseus::reports::create_report(request).await?)
}
