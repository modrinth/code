use chrono::{DateTime, Utc};

use crate::database::models::DBUserId;

pub struct ExternalLicense {
    pub id: i64,
    pub title: Option<String>,
    pub status: String,
    pub link: Option<String>,
    pub proof: Option<String>,
    pub flame_project_id: Option<i32>,
}

impl ExternalLicense {
    pub async fn insert_many(
        exec: impl sqlx::PgExecutor<'_>,
        licenses: &[ExternalLicense],
        user_id: DBUserId,
    ) -> sqlx::Result<()> {
        let now = Utc::now();

        let ids: Vec<i64> = licenses.iter().map(|x| x.id).collect();
        let titles: Vec<Option<String>> =
            licenses.iter().map(|x| x.title.clone()).collect();
        let statuses: Vec<String> =
            licenses.iter().map(|x| x.status.clone()).collect();
        let links: Vec<Option<String>> =
            licenses.iter().map(|x| x.link.clone()).collect();
        let proofs: Vec<Option<String>> =
            licenses.iter().map(|x| x.proof.clone()).collect();
        let flame_ids: Vec<Option<i32>> =
            licenses.iter().map(|x| x.flame_project_id).collect();
        let nows: Vec<DateTime<Utc>> = vec![now; licenses.len()];
        let user_ids: Vec<i64> = vec![user_id.0; licenses.len()];

        sqlx::query!(
            r#"
            INSERT INTO moderation_external_licenses (id, title, status, link, proof, flame_project_id, inserted_at, inserted_by, updated_at, updated_by)
            SELECT * FROM UNNEST ($1::bigint[], $2::varchar[], $3::varchar[], $4::varchar[], $5::varchar[], $6::integer[], $7::timestamptz[], $8::bigint[], $7::timestamptz[], $8::bigint[])
            ON CONFLICT (id) DO UPDATE SET
                title = EXCLUDED.title,
                status = EXCLUDED.status,
                link = EXCLUDED.link,
                proof = EXCLUDED.proof,
                flame_project_id = EXCLUDED.flame_project_id,
                updated_at = EXCLUDED.updated_at,
                updated_by = EXCLUDED.updated_by
            "#,
            &ids,
            &titles as _,
            &statuses,
            &links as _,
            &proofs as _,
            &flame_ids as _,
            &nows,
            &user_ids,
        )
        .execute(exec)
        .await?;

        Ok(())
    }

    pub async fn insert_files(
        exec: impl sqlx::PgExecutor<'_>,
        hashes: &[Vec<u8>],
        filenames: &[Option<String>],
        license_ids: &[i64],
        user_id: DBUserId,
    ) -> sqlx::Result<()> {
        let now = Utc::now();
        let nows: Vec<DateTime<Utc>> = vec![now; license_ids.len()];
        let user_ids: Vec<i64> = vec![user_id.0; license_ids.len()];

        let filenames: Vec<Option<String>> = filenames.to_vec();

        sqlx::query!(
            r#"
            INSERT INTO moderation_external_files (sha1, filename, external_license_id, inserted_at, inserted_by, updated_at, updated_by)
            SELECT * FROM UNNEST ($1::bytea[], $2::varchar[], $3::bigint[], $4::timestamptz[], $5::bigint[], $4::timestamptz[], $5::bigint[])
            ON CONFLICT (sha1) DO UPDATE SET
                filename = COALESCE(EXCLUDED.filename, moderation_external_files.filename),
                external_license_id = EXCLUDED.external_license_id,
                updated_at = EXCLUDED.updated_at,
                updated_by = EXCLUDED.updated_by
            "#,
            hashes,
            &filenames as _,
            license_ids,
            &nows,
            &user_ids,
        )
        .execute(exec)
        .await?;

        Ok(())
    }
}
