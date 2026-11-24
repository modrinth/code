use actix_web::{HttpResponse, query, web};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(query_queue);
}

#[derive(Debug, Default, Deserialize, ToSchema)]
#[serde(default)]
pub struct QueryQueue {
    pub queue_kind: QueueKind,
    pub project_types: Vec<ProjectType>,
    pub submission_kind: Option<SubmissionKind>,
    pub project_flags: Vec<String>,
    pub order: QueueOrder,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(transparent)]
pub struct ProjectType(pub String);

#[derive(Debug, Default, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueueKind {
    #[default]
    InReview,
    ScheduledForApproval,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionKind {
    New,
    Resubmission,
}

#[derive(Debug, Default, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueueOrder {
    #[default]
    OldestFirst,
    NewestFirst,
}

#[query("/queue")]
async fn query_queue(_query: web::Json<QueryQueue>) -> HttpResponse {
    HttpResponse::NotImplemented().finish()
}
