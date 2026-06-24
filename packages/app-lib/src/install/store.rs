use super::model::{
	InstallJobKind, InstallJobSnapshot, InstallJobState, InstallJobStatus,
};
use crate::state::State;
use chrono::{DateTime, TimeZone, Utc};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct InstallJobRecord {
	pub id: Uuid,
	pub instance_id: Option<String>,
	pub kind: InstallJobKind,
	pub status: InstallJobStatus,
	pub state: InstallJobState,
	pub created: DateTime<Utc>,
	pub modified: DateTime<Utc>,
	pub finished: Option<DateTime<Utc>>,
	pub dismissed: bool,
}

#[derive(Debug)]
struct InstallJobRow {
	pub id: String,
	pub instance_id: Option<String>,
	pub kind: String,
	pub status: String,
	pub state: String,
	pub created: i64,
	pub modified: i64,
	pub finished: Option<i64>,
	pub dismissed: i64,
}

impl InstallJobRecord {
	pub fn snapshot(&self) -> InstallJobSnapshot {
		InstallJobSnapshot {
			job_id: self.id,
			instance_id: self.instance_id.clone(),
			kind: self.kind,
			status: self.status,
			target: self.state.target.clone(),
			phase: self.state.progress.phase,
			progress: self.state.progress.progress.clone(),
			details: self.state.progress.details.clone(),
			display: self.state.display.clone(),
			error: self.state.error.clone(),
			created: self.created,
			modified: self.modified,
			finished: self.finished,
		}
	}
}

pub async fn insert(
	id: Uuid,
	state: &InstallJobState,
	status: InstallJobStatus,
	app_state: &State,
) -> crate::Result<InstallJobRecord> {
	let now = Utc::now();
	let kind = state.request.kind();
	let json = serde_json::to_string(state)?;
	let status_value = status.as_str();
	let kind_value = kind.as_str();
	let instance_id = instance_id(state);
	let id_value = id.to_string();
	let created = now.timestamp();
	let modified = created;

	sqlx::query!(
		"
		INSERT INTO install_jobs (
			id, instance_id, kind, status, state, created, modified, finished, dismissed
		)
		VALUES (?, ?, ?, ?, ?, ?, ?, NULL, 0)
		",
		id_value,
		instance_id,
		kind_value,
		status_value,
		json,
		created,
		modified,
	)
	.execute(&app_state.pool)
	.await?;

	get(id, app_state).await?.ok_or_else(|| {
		crate::ErrorKind::OtherError(format!("Install job {id} was not inserted")).into()
	})
}

pub async fn get(
	id: Uuid,
	app_state: &State,
) -> crate::Result<Option<InstallJobRecord>> {
	let id = id.to_string();
	let row = sqlx::query_as!(
		InstallJobRow,
		"
		SELECT
			id AS \"id!: String\",
			instance_id,
			kind AS \"kind!: String\",
			status AS \"status!: String\",
			state AS \"state!: String\",
			created AS \"created!: i64\",
			modified AS \"modified!: i64\",
			finished,
			dismissed AS \"dismissed!: i64\"
		FROM install_jobs
		WHERE id = ?
		",
		id,
	)
	.fetch_optional(&app_state.pool)
	.await?;

	row.map(row_to_record).transpose()
}

pub async fn list(
	include_finished: bool,
	app_state: &State,
) -> crate::Result<Vec<InstallJobRecord>> {
	let rows = if include_finished {
		sqlx::query_as!(
			InstallJobRow,
			"
			SELECT
				id AS \"id!: String\",
				instance_id,
				kind AS \"kind!: String\",
				status AS \"status!: String\",
				state AS \"state!: String\",
				created AS \"created!: i64\",
				modified AS \"modified!: i64\",
				finished,
				dismissed AS \"dismissed!: i64\"
			FROM install_jobs
			WHERE dismissed = 0
			ORDER BY created ASC
			",
		)
		.fetch_all(&app_state.pool)
		.await?
	} else {
		sqlx::query_as!(
			InstallJobRow,
			"
			SELECT
				id AS \"id!: String\",
				instance_id,
				kind AS \"kind!: String\",
				status AS \"status!: String\",
				state AS \"state!: String\",
				created AS \"created!: i64\",
				modified AS \"modified!: i64\",
				finished,
				dismissed AS \"dismissed!: i64\"
			FROM install_jobs
			WHERE dismissed = 0 AND status IN ('queued', 'running', 'failed', 'interrupted')
			ORDER BY created ASC
			",
		)
		.fetch_all(&app_state.pool)
		.await?
	};

	rows.into_iter().map(row_to_record).collect()
}

pub async fn list_interrupted_candidates(
	app_state: &State,
) -> crate::Result<Vec<InstallJobRecord>> {
	let rows = sqlx::query_as!(
		InstallJobRow,
		"
		SELECT
			id AS \"id!: String\",
			instance_id,
			kind AS \"kind!: String\",
			status AS \"status!: String\",
			state AS \"state!: String\",
			created AS \"created!: i64\",
			modified AS \"modified!: i64\",
			finished,
			dismissed AS \"dismissed!: i64\"
		FROM install_jobs
		WHERE status IN ('queued', 'running')
		ORDER BY created ASC
		",
	)
	.fetch_all(&app_state.pool)
	.await?;

	rows.into_iter().map(row_to_record).collect()
}

pub async fn update_state(
	id: Uuid,
	state: &InstallJobState,
	app_state: &State,
) -> crate::Result<InstallJobRecord> {
	let now = Utc::now();
	let json = serde_json::to_string(state)?;
	let instance_id = instance_id(state);
	let id_value = id.to_string();
	let modified = now.timestamp();

	sqlx::query!(
		"
		UPDATE install_jobs
		SET instance_id = ?, state = ?, modified = ?
		WHERE id = ?
		",
		instance_id,
		json,
		modified,
		id_value,
	)
	.execute(&app_state.pool)
	.await?;

	get_required(id, app_state).await
}

pub async fn update_status(
	id: Uuid,
	status: InstallJobStatus,
	state: &InstallJobState,
	app_state: &State,
) -> crate::Result<InstallJobRecord> {
	let now = Utc::now();
	let finished = status.is_finished().then_some(now.timestamp());
	let json = serde_json::to_string(state)?;
	let status_value = status.as_str();
	let instance_id = instance_id(state);
	let id_value = id.to_string();
	let modified = now.timestamp();

	sqlx::query!(
		"
		UPDATE install_jobs
		SET instance_id = ?, status = ?, state = ?, modified = ?, finished = ?
		WHERE id = ?
		",
		instance_id,
		status_value,
		json,
		modified,
		finished,
		id_value,
	)
	.execute(&app_state.pool)
	.await?;

	get_required(id, app_state).await
}

pub async fn dismiss(id: Uuid, app_state: &State) -> crate::Result<()> {
	let id = id.to_string();
	let modified = Utc::now().timestamp();
	sqlx::query!(
		"
		UPDATE install_jobs
		SET dismissed = 1, modified = ?
		WHERE id = ?
		",
		modified,
		id,
	)
	.execute(&app_state.pool)
	.await?;

	Ok(())
}

pub async fn get_required(
	id: Uuid,
	app_state: &State,
) -> crate::Result<InstallJobRecord> {
	get(id, app_state).await?.ok_or_else(|| {
		crate::ErrorKind::InputError(format!("Unknown install job {id}")).into()
	})
}

fn row_to_record(row: InstallJobRow) -> crate::Result<InstallJobRecord> {
	Ok(InstallJobRecord {
		id: Uuid::parse_str(&row.id).map_err(|err| {
			crate::ErrorKind::InputError(format!(
				"Invalid install job id {}: {err}",
				row.id
			))
		})?,
		instance_id: row.instance_id,
		kind: InstallJobKind::from_str(&row.kind),
		status: InstallJobStatus::from_str(&row.status),
		state: serde_json::from_str(&row.state)?,
		created: timestamp(row.created),
		modified: timestamp(row.modified),
		finished: row.finished.and_then(optional_timestamp),
		dismissed: row.dismissed != 0,
	})
}

fn instance_id(state: &InstallJobState) -> Option<String> {
	match &state.target {
		super::model::InstallTarget::NewInstance { instance_id } => {
			instance_id.clone()
		}
		super::model::InstallTarget::ExistingInstance { instance_id } => {
			Some(instance_id.clone())
		}
	}
}

fn timestamp(value: i64) -> DateTime<Utc> {
	Utc.timestamp_opt(value, 0).single().unwrap_or_else(Utc::now)
}

fn optional_timestamp(value: i64) -> Option<DateTime<Utc>> {
	Utc.timestamp_opt(value, 0).single()
}
