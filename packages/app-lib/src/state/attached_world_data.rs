use std::collections::HashMap;

use crate::worlds::DisplayStatus;

#[derive(Debug, Clone)]
pub struct AttachedWorldDataReference {
    pub profile_path: String,
    pub world_type: WorldType,
    pub world_id: String,
    pub data: AttachedWorldData,
}

#[derive(Debug, Clone, Default)]
pub struct AttachedWorldData {
    pub display_status: DisplayStatus,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum WorldType {
    World,
    Server,
}

impl WorldType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::World => "normal",
            Self::Server => "hidden",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "world" => Self::World,
            "server" => Self::Server,
            _ => Self::World,
        }
    }
}

impl AttachedWorldDataReference {
    pub async fn upsert(
        &self,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<()> {
        let world_type = self.world_type.as_str();
        let display_status = self.data.display_status.as_str();

        sqlx::query!(
            "
            INSERT INTO attached_world_data (profile_path, world_type, world_id, display_status)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (profile_path, world_type, world_id) DO UPDATE SET
                display_status = $4
            ",
            self.profile_path,
            world_type,
            self.world_id,
            display_status
        )
        .execute(exec)
        .await?;

        Ok(())
    }
}

impl AttachedWorldData {
    pub async fn get_for_world(
        instance: &str,
        world_type: WorldType,
        world_id: &str,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<Self> {
        let world_type = world_type.as_str();

        let attached_data = sqlx::query!(
            "
            SELECT display_status
            FROM attached_world_data
            WHERE profile_path = $1 and world_type = $2 and world_id = $3
            ",
            instance,
            world_type,
            world_id
        )
        .fetch_optional(exec)
        .await?;

        Ok(attached_data
            .map(|x| AttachedWorldData {
                display_status: DisplayStatus::from_str(&x.display_status),
            })
            .unwrap_or_else(Default::default))
    }

    pub async fn get_all_for_instance(
        instance: &str,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<HashMap<(WorldType, String), Self>> {
        let attached_data = sqlx::query!(
            "
            SELECT world_type, world_id, display_status
            FROM attached_world_data
            WHERE profile_path = $1
            ",
            instance
        )
        .fetch_all(exec)
        .await?;

        Ok(attached_data
            .into_iter()
            .map(|x| {
                let world_type = WorldType::from_str(&x.world_type);
                let display_status = DisplayStatus::from_str(&x.display_status);
                (
                    (world_type, x.world_id),
                    AttachedWorldData { display_status },
                )
            })
            .collect())
    }
}
