use crate::routes::ApiError;
use crate::search::SearchConfig;
use crate::util::guards::admin_key_guard;
use actix_web::{HttpResponse, delete, get, web};
use meilisearch_sdk::tasks::{Task, TasksCancelQuery};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use utoipa::ToSchema;

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(tasks).service(tasks_cancel);
}

#[utoipa::path]
#[get("tasks", guard = "admin_key_guard")]
pub async fn tasks(
    config: web::Data<SearchConfig>,
) -> Result<HttpResponse, ApiError> {
    let client = config.make_batch_client()?;
    let tasks = client
        .with_all_clients("get_tasks", async |client| {
            let tasks = client.get_tasks().await?;

            Ok(tasks.results)
        })
        .await?;

    #[derive(Serialize, ToSchema)]
    struct MeiliTask<Time> {
        uid: u32,
        status: &'static str,
        duration: Option<Duration>,
        enqueued_at: Option<Time>,
    }

    #[derive(Serialize, ToSchema)]
    struct TaskList<Time> {
        by_instance: HashMap<String, Vec<MeiliTask<Time>>>,
    }

    let response = tasks
        .into_iter()
        .enumerate()
        .map(|(idx, instance_tasks)| {
            let tasks = instance_tasks
                .into_iter()
                .filter_map(|task| {
                    Some(match task {
                        Task::Enqueued { content } => MeiliTask {
                            uid: content.uid,
                            status: "enqueued",
                            duration: None,
                            enqueued_at: Some(content.enqueued_at),
                        },
                        Task::Processing { content } => MeiliTask {
                            uid: content.uid,
                            status: "processing",
                            duration: None,
                            enqueued_at: Some(content.enqueued_at),
                        },
                        Task::Failed { content } => MeiliTask {
                            uid: content.task.uid,
                            status: "failed",
                            duration: Some(content.task.duration),
                            enqueued_at: Some(content.task.enqueued_at),
                        },
                        Task::Succeeded { content: _ } => return None,
                    })
                })
                .collect();

            (idx.to_string(), tasks)
        })
        .collect::<HashMap<String, Vec<MeiliTask<_>>>>();

    Ok(HttpResponse::Ok().json(TaskList {
        by_instance: response,
    }))
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
enum TasksCancelFilter {
    All,
    AllEnqueued,
    Indexes { indexes: Vec<String> },
}

#[utoipa::path]
#[delete("tasks", guard = "admin_key_guard")]
pub async fn tasks_cancel(
    config: web::Data<SearchConfig>,
    body: web::Json<TasksCancelFilter>,
) -> Result<HttpResponse, ApiError> {
    let client = config.make_batch_client()?;
    let all_results = client
        .with_all_clients("cancel_tasks", async |client| {
            let mut q = TasksCancelQuery::new(client);
            match &body.0 {
                TasksCancelFilter::All => {}
                TasksCancelFilter::Indexes { indexes } => {
                    q.with_index_uids(indexes.iter().map(|s| s.as_str()));
                }
                TasksCancelFilter::AllEnqueued => {
                    q.with_statuses(["enqueued"]);
                }
            };

            let result = client.cancel_tasks_with(&q).await;

            Ok(result)
        })
        .await?;

    for r in all_results {
        r?;
    }

    Ok(HttpResponse::Ok().finish())
}
