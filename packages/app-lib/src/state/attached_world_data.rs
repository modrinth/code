use crate::worlds::{DisplayStatus, WorldType};
use paste::paste;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct AttachedWorldData {
    pub display_status: DisplayStatus,
}

impl AttachedWorldData {
    pub async fn get_for_world(
        instance: &str,
        world_type: WorldType,
        world_id: &str,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<Option<Self>> {
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

        Ok(attached_data.map(|x| AttachedWorldData {
            display_status: DisplayStatus::from_string(&x.display_status),
        }))
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
                let world_type = WorldType::from_string(&x.world_type);
                let display_status =
                    DisplayStatus::from_string(&x.display_status);
                (
                    (world_type, x.world_id),
                    AttachedWorldData { display_status },
                )
            })
            .collect())
    }

    pub async fn remove_for_world(
        instance: &str,
        world_type: WorldType,
        world_id: &str,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<()> {
        let world_type = world_type.as_str();

        sqlx::query!(
            "
            DELETE FROM attached_world_data
            WHERE profile_path = $1 and world_type = $2 and world_id = $3
            ",
            instance,
            world_type,
            world_id
        )
        .execute(exec)
        .await?;

        Ok(())
    }
}

macro_rules! attached_data_setter {
    ($parameter:ident: $parameter_type:ty, $column:expr $(=> $adapter:expr)?) => {
        paste! {
            pub async fn [<set_ $parameter>](
                instance: &str,
                world_type: WorldType,
                world_id: &str,
                $parameter: $parameter_type,
                exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
            ) -> crate::Result<()> {
                let world_type = world_type.as_str();
                $(let $parameter = $adapter;)?

                sqlx::query!(
                    "INSERT INTO attached_world_data (profile_path, world_type, world_id, " + $column + ")\n" +
                    "VALUES ($1, $2, $3, $4)\n" +
                    "ON CONFLICT (profile_path, world_type, world_id) DO UPDATE\n" +
                    "    SET " + $column + " = $4",
                    instance,
                    world_type,
                    world_id,
                    $parameter
                )
                .execute(exec)
                .await?;

                Ok(())
            }
        }
    }
}

attached_data_setter!(display_status: DisplayStatus, "display_status" => display_status.as_str());
