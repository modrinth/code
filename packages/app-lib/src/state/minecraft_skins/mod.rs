use std::collections::HashSet;

use futures::{Stream, StreamExt, stream};
use uuid::{Uuid, fmt::Hyphenated};

use super::MinecraftSkinVariant;

pub mod mojang_api;

/// Represents a saved skin row for a Minecraft player.
///
/// The same player and `texture_key` always point to the same saved skin.
/// Changing the model variant or cape updates that saved skin instead of
/// creating a second copy. Bundled default skins with a cape are also stored
/// here so the cape can stay associated with the default skin card.
#[derive(Debug, Clone)]
pub struct CustomMinecraftSkin {
    /// The key for the skin texture, which is akin to a hash that identifies it.
    pub texture_key: String,
    /// The variant of the skin model.
    pub variant: MinecraftSkinVariant,
    /// The UUID of the cape that this skin uses, which should match one of the
    /// cape UUIDs the player has in its profile.
    ///
    /// If `None`, the skin is saved without a cape.
    pub cape_id: Option<Uuid>,
    /// The saved skin display order within this player's saved skins.
    pub display_order: i64,
}

#[derive(Debug, Clone, Copy)]
pub enum CustomMinecraftSkinInsertPosition {
    Top,
    Bottom,
    At(i64),
}

struct CustomMinecraftSkinRow {
    texture_key: String,
    variant: MinecraftSkinVariant,
    cape_id: Option<Hyphenated>,
    display_order: i64,
}

impl CustomMinecraftSkin {
    pub async fn add(
        minecraft_user_id: Uuid,
        texture_key: &str,
        texture: &[u8],
        variant: MinecraftSkinVariant,
        cape_id: Option<Uuid>,
        insert_position: CustomMinecraftSkinInsertPosition,
        db: impl sqlx::Acquire<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<()> {
        let minecraft_user_id = minecraft_user_id.as_hyphenated();
        let cape_id = cape_id.map(|id| id.hyphenated());

        let mut transaction = db.begin().await?;

        let existing_order = sqlx::query_scalar!(
            "SELECT display_order FROM custom_minecraft_skins WHERE minecraft_user_uuid = ? AND texture_key = ?",
            minecraft_user_id,
            texture_key
        )
        .fetch_optional(&mut *transaction)
        .await?;

        let display_order = match existing_order {
            Some(display_order) => display_order,
            None => match insert_position {
                CustomMinecraftSkinInsertPosition::Top => {
                    sqlx::query!(
                        "UPDATE custom_minecraft_skins SET display_order = display_order + 1 WHERE minecraft_user_uuid = ?",
                        minecraft_user_id
                    )
                    .execute(&mut *transaction)
                    .await?;

                    0
                }
                CustomMinecraftSkinInsertPosition::Bottom => {
                    sqlx::query_scalar!(
                        "SELECT COALESCE(MAX(display_order) + 1, 0) AS 'display_order!: i64' \
                        FROM custom_minecraft_skins WHERE minecraft_user_uuid = ?",
                        minecraft_user_id
                    )
                    .fetch_one(&mut *transaction)
                    .await?
                }
                CustomMinecraftSkinInsertPosition::At(display_order) => {
                    sqlx::query!(
                        "UPDATE custom_minecraft_skins SET display_order = display_order + 1 \
                        WHERE minecraft_user_uuid = ? AND display_order >= ?",
                        minecraft_user_id,
                        display_order
                    )
                    .execute(&mut *transaction)
                    .await?;

                    display_order
                }
            },
        };

        sqlx::query!(
            "DELETE FROM custom_minecraft_skins WHERE minecraft_user_uuid = ? AND texture_key = ?",
            minecraft_user_id,
            texture_key
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
			"INSERT OR REPLACE INTO custom_minecraft_skin_textures (texture_key, texture) VALUES (?, ?)",
			texture_key, texture
		)
		.execute(&mut *transaction)
        .await?;

        sqlx::query!(
			"INSERT INTO custom_minecraft_skins (minecraft_user_uuid, texture_key, variant, cape_id, display_order) VALUES (?, ?, ?, ?, ?)",
			minecraft_user_id, texture_key, variant, cape_id, display_order
		)
		.execute(&mut *transaction)
		.await?;

        transaction.commit().await?;

        Ok(())
    }

    pub async fn get_by_texture(
        minecraft_user_id: Uuid,
        texture_key: &str,
        db: impl sqlx::Acquire<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<Option<Self>> {
        let minecraft_user_id = minecraft_user_id.as_hyphenated();

        sqlx::query_as!(
            CustomMinecraftSkinRow,
            "SELECT texture_key, variant AS 'variant: MinecraftSkinVariant', cape_id AS 'cape_id: Hyphenated', display_order \
            FROM custom_minecraft_skins \
            WHERE minecraft_user_uuid = ? AND texture_key = ?",
            minecraft_user_id,
            texture_key
        )
        .fetch_optional(&mut *db.acquire().await?)
        .await?
        .map(|row| {
            Ok(Self {
                texture_key: row.texture_key,
                variant: row.variant,
                cape_id: row.cape_id.map(Uuid::from),
                display_order: row.display_order,
            })
        })
        .transpose()
    }

    pub async fn get_many(
        minecraft_user_id: Uuid,
        offset: u32,
        count: u32,
        db: impl sqlx::Acquire<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<impl Stream<Item = Self>> {
        let minecraft_user_id = minecraft_user_id.as_hyphenated();

        Ok(stream::iter(sqlx::query!(
            "SELECT texture_key, variant AS 'variant: MinecraftSkinVariant', cape_id AS 'cape_id: Hyphenated', display_order \
            FROM custom_minecraft_skins \
            WHERE minecraft_user_uuid = ? \
            ORDER BY display_order ASC, rowid ASC \
            LIMIT ? OFFSET ?",
            minecraft_user_id, count, offset
        )
        .fetch_all(&mut *db.acquire().await?)
        .await?)
        .map(|row| Self {
            texture_key: row.texture_key,
            variant: row.variant,
            cape_id: row.cape_id.map(Uuid::from),
            display_order: row.display_order,
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

        sqlx::query!(
            "DELETE FROM custom_minecraft_skins WHERE minecraft_user_uuid = ? AND texture_key = ?",
            minecraft_user_id,
            self.texture_key
        )
        .execute(&mut *db.acquire().await?)
        .await?;

        Ok(())
    }

    pub async fn set_order(
        minecraft_user_id: Uuid,
        texture_keys: &[String],
        db: impl sqlx::Acquire<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<()> {
        let minecraft_user_id = minecraft_user_id.as_hyphenated();
        let mut transaction = db.begin().await?;

        let existing_rows = sqlx::query!(
            "SELECT texture_key FROM custom_minecraft_skins \
            WHERE minecraft_user_uuid = ? \
            ORDER BY display_order ASC, rowid ASC",
            minecraft_user_id
        )
        .fetch_all(&mut *transaction)
        .await?;

        let existing_keys = existing_rows
            .iter()
            .map(|row| row.texture_key.as_str())
            .collect::<HashSet<_>>();
        let mut seen_keys = HashSet::new();
        let mut ordered_keys = Vec::with_capacity(existing_rows.len());

        for texture_key in texture_keys {
            if seen_keys.insert(texture_key.as_str())
                && existing_keys.contains(texture_key.as_str())
            {
                ordered_keys.push(texture_key.as_str());
            }
        }

        for row in &existing_rows {
            if seen_keys.insert(row.texture_key.as_str()) {
                ordered_keys.push(row.texture_key.as_str());
            }
        }

        for (display_order, texture_key) in ordered_keys.into_iter().enumerate()
        {
            let display_order = display_order as i64;

            sqlx::query!(
                "UPDATE custom_minecraft_skins SET display_order = ? \
                WHERE minecraft_user_uuid = ? AND texture_key = ?",
                display_order,
                minecraft_user_id,
                texture_key
            )
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;

        Ok(())
    }
}
