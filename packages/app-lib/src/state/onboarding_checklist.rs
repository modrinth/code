use serde::{Deserialize, Serialize};
use sqlx::{Executor, Sqlite};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OnboardingChecklist {
    pub has_created_instance: bool,
    pub has_logged_into_minecraft: bool,
    pub has_logged_into_modrinth: bool,
    pub show_checklist: bool,
}

pub(crate) enum OnboardingChecklistItem {
    CreatedInstance,
    LoggedIntoMinecraft,
    LoggedIntoModrinth,
}

pub(crate) async fn get_onboarding_checklist(
    exec: impl Executor<'_, Database = Sqlite>,
) -> crate::Result<OnboardingChecklist> {
    let row = sqlx::query!(
        "
        SELECT
            has_created_instance,
            has_logged_into_minecraft,
            has_logged_into_modrinth,
            show_checklist
        FROM onboarding_checklist
        WHERE id = 0
        ",
    )
    .fetch_one(exec)
    .await?;

    Ok(OnboardingChecklist {
        has_created_instance: row.has_created_instance == 1,
        has_logged_into_minecraft: row.has_logged_into_minecraft == 1,
        has_logged_into_modrinth: row.has_logged_into_modrinth == 1,
        show_checklist: row.show_checklist == 1,
    })
}

pub(crate) async fn mark_onboarding_checklist_item(
    item: OnboardingChecklistItem,
    pool: &sqlx::SqlitePool,
) -> crate::Result<Option<OnboardingChecklist>> {
    let result = match item {
        OnboardingChecklistItem::CreatedInstance => {
            sqlx::query!(
                "
                UPDATE onboarding_checklist
                SET has_created_instance = TRUE
                WHERE id = 0 AND has_created_instance = FALSE
                ",
            )
            .execute(pool)
            .await?
        }
        OnboardingChecklistItem::LoggedIntoMinecraft => {
            sqlx::query!(
                "
                UPDATE onboarding_checklist
                SET has_logged_into_minecraft = TRUE
                WHERE id = 0 AND has_logged_into_minecraft = FALSE
                ",
            )
            .execute(pool)
            .await?
        }
        OnboardingChecklistItem::LoggedIntoModrinth => {
            sqlx::query!(
                "
                UPDATE onboarding_checklist
                SET has_logged_into_modrinth = TRUE
                WHERE id = 0 AND has_logged_into_modrinth = FALSE
                ",
            )
            .execute(pool)
            .await?
        }
    };

    if result.rows_affected() == 0 {
        return Ok(None);
    }

    sqlx::query!(
        "
        UPDATE onboarding_checklist
        SET show_checklist = FALSE
        WHERE id = 0
            AND show_checklist = TRUE
            AND has_created_instance = TRUE
            AND has_logged_into_minecraft = TRUE
            AND has_logged_into_modrinth = TRUE
        "
    )
    .execute(pool)
    .await?;

    Ok(Some(get_onboarding_checklist(pool).await?))
}
