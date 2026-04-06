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
        let titles: Vec<Option<&str>> =
            licenses.iter().map(|x| x.title.as_deref()).collect();
        let statuses: Vec<&str> =
            licenses.iter().map(|x| x.status.as_str()).collect();
        let links: Vec<Option<&str>> =
            licenses.iter().map(|x| x.link.as_deref()).collect();
        let proofs: Vec<Option<&str>> =
            licenses.iter().map(|x| x.proof.as_deref()).collect();
        let flame_ids: Vec<Option<i32>> =
            licenses.iter().map(|x| x.flame_project_id).collect();
        let nows: Vec<DateTime<Utc>> = vec![now; licenses.len()];
        let user_ids: Vec<i64> = vec![user_id.0; licenses.len()];

        sqlx::query(
            "
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
            ",
        )
        .bind(&ids[..])
        .bind(&titles[..])
        .bind(&statuses[..])
        .bind(&links[..])
        .bind(&proofs[..])
        .bind(&flame_ids[..])
        .bind(&nows[..])
        .bind(&user_ids[..])
        .execute(exec)
        .await?;

        Ok(())
    }

    pub async fn insert_files(
        exec: impl sqlx::PgExecutor<'_>,
        hashes: &[Vec<u8>],
        license_ids: &[i64],
        user_id: DBUserId,
    ) -> sqlx::Result<()> {
        let now = Utc::now();
        let nows: Vec<DateTime<Utc>> = vec![now; license_ids.len()];
        let user_ids: Vec<i64> = vec![user_id.0; license_ids.len()];

        sqlx::query(
            "
            INSERT INTO moderation_external_files (sha1, external_license_id, inserted_at, inserted_by, updated_at, updated_by)
            SELECT * FROM UNNEST ($1::bytea[], $2::bigint[], $3::timestamptz[], $4::bigint[], $3::timestamptz[], $4::bigint[])
            ON CONFLICT (sha1) DO UPDATE SET
                external_license_id = EXCLUDED.external_license_id,
                updated_at = EXCLUDED.updated_at,
                updated_by = EXCLUDED.updated_by
            ",
        )
        .bind(hashes)
        .bind(license_ids)
        .bind(&nows[..])
        .bind(&user_ids[..])
        .execute(exec)
        .await?;

        Ok(())
    }
}
