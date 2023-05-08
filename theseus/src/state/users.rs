//! User login info
use crate::auth::Credentials;
use crate::data::DirectoryInfo;
use crate::util::fetch::{read_json, write, IoSemaphore};
use crate::State;
use std::collections::HashMap;
use uuid::Uuid;

const USERS_JSON: &str = "users.json";

/// The set of users stored in the launcher
#[derive(Clone)]
pub(crate) struct Users(pub(crate) HashMap<Uuid, Credentials>);

impl Users {
    pub async fn init(
        dirs: &DirectoryInfo,
        io_semaphore: &IoSemaphore,
    ) -> crate::Result<Self> {
        let users_path = dirs.caches_meta_dir().join(USERS_JSON);
        let users = read_json(&users_path, io_semaphore).await.ok();

        if let Some(users) = users {
            Ok(Self(users))
        } else {
            Ok(Self(HashMap::new()))
        }
    }

    pub async fn save(&self) -> crate::Result<()> {
        let state = State::get().await?;
        let users_path = state.directories.caches_meta_dir().join(USERS_JSON);
        write(
            &users_path,
            &serde_json::to_vec(&self.0)?,
            &state.io_semaphore,
        )
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn insert(
        &mut self,
        credentials: &Credentials,
    ) -> crate::Result<&Self> {
        self.0.insert(credentials.id, credentials.clone());
        self.save().await?;
        Ok(self)
    }

    #[tracing::instrument(skip(self))]
    pub fn contains(&self, id: Uuid) -> bool {
        self.0.contains_key(&id)
    }

    #[tracing::instrument(skip(self))]
    pub fn get(&self, id: Uuid) -> Option<Credentials> {
        self.0.get(&id).cloned()
    }

    #[tracing::instrument(skip(self))]
    pub async fn remove(&mut self, id: Uuid) -> crate::Result<&Self> {
        self.0.remove(&id);
        self.save().await?;
        Ok(self)
    }
}
