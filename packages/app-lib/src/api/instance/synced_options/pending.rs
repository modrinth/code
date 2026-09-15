use super::orchestration::{SyncedOptionJoinResolution, synced_options_path};
use crate::State;
use crate::state::SyncedOption;
use crate::util::io;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub(super) enum PendingAction {
    SelectSource,
    Join {
        resolution: Option<SyncedOptionJoinResolution>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(super) struct PendingChange {
    pub instance_id: String,
    pub option: SyncedOption,
    pub action: PendingAction,
}

pub(super) async fn read(state: &State) -> crate::Result<Vec<PendingChange>> {
    let path = synced_options_path(state).join("pending-changes.json");
    match tokio::fs::read(path).await {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(Vec::new())
        }
        Err(error) => Err(error.into()),
    }
}

async fn write(changes: &[PendingChange], state: &State) -> crate::Result<()> {
    let directory = synced_options_path(state);
    io::create_dir_all(&directory).await?;
    let temporary = directory.join("pending-changes.json.tmp");
    io::write(&temporary, serde_json::to_vec(changes)?).await?;
    tokio::fs::rename(temporary, directory.join("pending-changes.json"))
        .await?;
    Ok(())
}

/// Callers hold the synced-options lock while updating or replaying requests.
pub(super) async fn save(
    change: PendingChange,
    state: &State,
) -> crate::Result<()> {
    let mut changes = read(state).await?;
    changes.retain(|previous| {
        previous.option != change.option
            || (change.action != PendingAction::SelectSource
                && previous.instance_id != change.instance_id)
    });
    changes.push(change);
    write(&changes, state).await
}

pub(super) async fn remove(
    change: &PendingChange,
    state: &State,
) -> crate::Result<()> {
    let mut changes = read(state).await?;
    let previous_len = changes.len();
    changes.retain(|previous| previous != change);
    if changes.len() != previous_len {
        write(&changes, state).await?;
    }
    Ok(())
}

pub(super) async fn cancel(
    option: SyncedOption,
    instance_id: Option<&str>,
    state: &State,
) -> crate::Result<()> {
    let mut changes = read(state).await?;
    let previous_len = changes.len();
    changes.retain(|change| {
        change.option != option
            || instance_id.is_some_and(|id| change.instance_id != id)
    });
    if changes.len() != previous_len {
        write(&changes, state).await?;
    }
    Ok(())
}

pub(in crate::api::instance) async fn contains(
    instance_id: &str,
    option: SyncedOption,
    state: &State,
) -> crate::Result<bool> {
    Ok(read(state).await?.iter().any(|change| {
        change.instance_id == instance_id && change.option == option
    }))
}
