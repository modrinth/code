pub mod theseus_proto {
    tonic::include_proto!("theseus");
}

use async_stream::try_stream;
use futures::Stream;
use std::{collections::HashMap, path::Path, pin::Pin, sync::Arc};
use theseus::launcher::{
    self,
    auth::provider::Credentials,
    meta::{fetch_version_info, fetch_version_manifest},
};
use theseus_proto::{theseus_server::*, *};
use tokio::{
    spawn,
    sync::{
        watch::{self, Receiver},
        RwLock,
    },
};
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;

#[derive(Debug, Default)]
struct TheseusService {
    task_map: Arc<RwLock<HashMap<u64, (Task, Receiver<TaskProgress>)>>>,
}

#[tonic::async_trait]
impl Theseus for TheseusService {
    async fn get_catalogue(&self, _: Request<Empty>) -> Result<Response<Catalogue>, Status> {
        let manifest = fetch_version_manifest().await.unwrap();
        Ok(Response::new(Catalogue {
            catalogue: manifest
                .versions
                .into_iter()
                .map(|v| Version { id: v.id })
                .collect(),
        }))
    }

    async fn launch(&self, request: Request<LaunchOptions>) -> Result<Response<Task>, Status> {
        let request = request.into_inner();
        let (tx, rx) = watch::channel(TaskProgress {
            finished: 0,
            total: 3,
            message: None,
        });
        let id = rand::random::<u64>() % 300;
        let task = Task {
            id,
            name: request.version_id.clone(),
        };
        let task_map = self.task_map.clone();
        task_map.write().await.insert(id, (task.clone(), rx));
        spawn(async move {
            tx.send(TaskProgress {
                finished: 0,
                total: 3,
                message: Some("Fetching Version Manifest".into()),
            })
            .unwrap();
            let manifest = fetch_version_manifest().await.unwrap();
            let version = manifest
                .versions
                .into_iter()
                .find(|v| v.id == request.version_id)
                .unwrap();
            tx.send(TaskProgress {
                finished: 1,
                total: 3,
                message: Some("Fetching Version Profile".into()),
            })
            .unwrap();
            let version = fetch_version_info(&version).await.unwrap();
            tx.send(TaskProgress {
                finished: 2,
                total: 3,
                message: Some("Downloading Minecraft".into()),
            })
            .unwrap();
            launcher::download_minecraft(&version, Path::new("minecraft"))
                .await
                .unwrap();
            tx.send(TaskProgress {
                finished: 3,
                total: 3,
                message: Some("Done".into()),
            })
            .unwrap();
            launcher::launch_minecraft(
                &request.version_id,
                Path::new("minecraft"),
                &Credentials {
                    access_token: "null".into(),
                    id: Uuid::new_v4(),
                    username: request.username,
                },
            )
            .await
            .unwrap();
            task_map.write().await.remove(&id).unwrap();
        });
        Ok(Response::new(task))
    }

    async fn get_task_list(&self, _: Request<Empty>) -> Result<Response<TaskList>, Status> {
        Ok(Response::new(TaskList {
            task_list: self
                .task_map
                .read()
                .await
                .iter()
                .map(|(_, (task, _))| task.clone())
                .collect(),
        }))
    }

    async fn get_task_progress(
        &self,
        request: Request<Task>,
    ) -> Result<Response<TaskProgress>, Status> {
        let request = request.into_inner();
        if let Some((task, receiver)) = self.task_map.read().await.get(&request.id) {
            if task.name == request.name {
                return Ok(Response::new(receiver.borrow().clone()));
            }
        }
        Err(Status::not_found(format!(
            "Task {} with the name {} was not found",
            request.id, request.name
        )))
    }

    type StreamTaskProgressStream =
        Pin<Box<dyn Stream<Item = Result<TaskProgress, Status>> + Send + Sync + 'static>>;

    async fn stream_task_progress(
        &self,
        request: Request<Task>,
    ) -> Result<Response<Self::StreamTaskProgressStream>, Status> {
        let request = request.into_inner();
        if let Some((task, receiver)) = self.task_map.read().await.get(&request.id) {
            if task.name == request.name {
                let mut receiver = receiver.clone();
                let stream = try_stream! {
                    while let Ok(_) = receiver.changed().await {
                        let progress = receiver.borrow().clone();
                        if progress.finished == progress.total {
                            yield progress;
                            return;
                        }
                        yield progress;
                    }
                };
                return Ok(Response::new(Box::pin(stream)));
            }
        }
        Err(Status::not_found(format!(
            "Task {} with the name {} was not found",
            request.id, request.name
        )))
    }
}

#[tokio::main]
async fn main() {
    let service = TheseusService::default();
    Server::builder()
        .add_service(TheseusServer::new(service))
        .serve("0.0.0.0:1234".parse().unwrap())
        .await
        .unwrap()
}
