use futures::{Stream, StreamExt, stream};
use uuid::{Uuid, fmt::Hyphenated};

use super::MinecraftSkinVariant;

pub mod mojang_api;

/// Represents the default cape for a Minecraft player.
#[derive(Debug, Clone)]
pub struct DefaultMinecraftCape {
    /// The UUID of a cape for a Minecraft player, which comes from its profile.
    ///
    /// This UUID may or may not be different for every player, even if they refer to the same cape.
    pub id: Uuid,
}

impl DefaultMinecraftCape {
    pub async fn set(
        minecraft_user_id: Uuid,
        cape_id: Uuid,
        db: impl sqlx::Acquire<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<()> {
        let minecraft_user_id = minecraft_user_id.as_hyphenated();
        let cape_id = cape_id.as_hyphenated();

        sqlx::query!(
            "INSERT OR REPLACE INTO default_minecraft_capes (minecraft_user_uuid, id) VALUES (?, ?)",
            minecraft_user_id, cape_id
        )
        .execute(&mut *db.acquire().await?)
        .await?;

        Ok(())
    }

    pub async fn get(
        minecraft_user_id: Uuid,
        db: impl sqlx::Acquire<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<Option<Self>> {
        let minecraft_user_id = minecraft_user_id.as_hyphenated();

        Ok(sqlx::query_as!(
            Self,
            "SELECT id AS 'id: Hyphenated' FROM default_minecraft_capes WHERE minecraft_user_uuid = ?",
            minecraft_user_id
        )
        .fetch_optional(&mut *db.acquire().await?)
        .await?)
    }

    pub async fn remove(
        minecraft_user_id: Uuid,
        db: impl sqlx::Acquire<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<()> {
        let minecraft_user_id = minecraft_user_id.as_hyphenated();

        sqlx::query!(
            "DELETE FROM default_minecraft_capes WHERE minecraft_user_uuid = ?",
            minecraft_user_id
        )
        .execute(&mut *db.acquire().await?)
        .await?;

        Ok(())
    }
}

/// Represents a custom skin for a Minecraft player.
#[derive(Debug, Clone)]
pub struct CustomMinecraftSkin {
    /// The key for the texture skin, which is akin to a hash that identifies it.
    pub texture_key: String,
    /// The variant of the skin model.
    pub variant: MinecraftSkinVariant,
    /// The UUID of the cape that this skin uses, which should match one of the
    /// cape UUIDs the player has in its profile.
    ///
    /// If `None`, the skin does not have an explicit cape set, and the default
    /// cape for this player, if any, should be used.
    pub cape_id: Option<Uuid>,
}

impl CustomMinecraftSkin {
    pub async fn add(
        minecraft_user_id: Uuid,
        texture_key: &str,
        texture: &[u8],
        variant: MinecraftSkinVariant,
        cape_id: Option<Uuid>,
        db: impl sqlx::Acquire<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<()> {
        let minecraft_user_id = minecraft_user_id.as_hyphenated();
        let cape_id = cape_id.map(|id| id.hyphenated());

        let mut transaction = db.begin().await?;

        sqlx::query!(
            "INSERT OR REPLACE INTO custom_minecraft_skin_textures (texture_key, texture) VALUES (?, ?)",
            texture_key, texture
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "INSERT OR REPLACE INTO custom_minecraft_skins (minecraft_user_uuid, texture_key, variant, cape_id) VALUES (?, ?, ?, ?)",
            minecraft_user_id, texture_key, variant, cape_id
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(())
    }

    pub async fn get_many(
        minecraft_user_id: Uuid,
        offset: u32,
        count: u32,
        db: impl sqlx::Acquire<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<impl Stream<Item = Self>> {
        let minecraft_user_id = minecraft_user_id.as_hyphenated();

        Ok(stream::iter(sqlx::query!(
            "SELECT texture_key, variant AS 'variant: MinecraftSkinVariant', cape_id AS 'cape_id: Hyphenated' \
            FROM custom_minecraft_skins \
            WHERE minecraft_user_uuid = ? \
            ORDER BY rowid ASC \
            LIMIT ? OFFSET ?",
            minecraft_user_id, count, offset
        )
        .fetch_all(&mut *db.acquire().await?)
        .await?)
        .map(|row| Self {
            texture_key: row.texture_key,
            variant: row.variant,
            cape_id: row.cape_id.map(Uuid::from),
        }))
    }

    pub async fn get_all(
        minecraft_user_id: Uuid,
        db: impl sqlx::Acquire<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<impl Stream<Item = Self>> {
        // Limit ourselves to 2048 skins, so that memory usage even when storing base64
        // PNG data of a 64x64 texture with random pixels stays around ~150 MiB
        Self::get_many(minecraft_user_id, 0, 2048, db).await
    }

    pub async fn texture_blob(
        &self,
        db: impl sqlx::Acquire<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<Vec<u8>> {
        Ok(sqlx::query_scalar!(
            "SELECT texture FROM custom_minecraft_skin_textures WHERE texture_key = ?",
            self.texture_key
        )
        .fetch_one(&mut *db.acquire().await?)
        .await?)
    }

    pub async fn remove(
        &self,
        minecraft_user_id: Uuid,
        db: impl sqlx::Acquire<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<()> {
        let minecraft_user_id = minecraft_user_id.as_hyphenated();
        let cape_id = self.cape_id.map(|id| id.hyphenated());

        sqlx::query!(
            "DELETE FROM custom_minecraft_skins \
            WHERE minecraft_user_uuid = ? AND texture_key = ? AND variant = ? AND cape_id IS ?",
            minecraft_user_id, self.texture_key, self.variant, cape_id
        )
        .execute(&mut *db.acquire().await?)
        .await?;

        Ok(())
    }
}
