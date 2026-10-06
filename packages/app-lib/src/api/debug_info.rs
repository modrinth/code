use crate::State;
use crate::util::io::IOError;
use serde::Serialize;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::sync::CancellationToken;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DebugInfoExportStage {
	Preparing,
	Exporting,
	Finishing,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugInfoExportProgress {
	pub stage: DebugInfoExportStage,
	pub processed_bytes: u64,
	pub total_bytes: Option<u64>,
}

struct ExportFile {
	path: PathBuf,
	name: String,
	length: u64,
}

pub async fn export_debug_info(
	export_path: PathBuf,
	cancel: CancellationToken,
	on_progress: impl Fn(DebugInfoExportProgress) + Send + Sync + 'static,
) -> crate::Result<bool> {
	on_progress(DebugInfoExportProgress {
		stage: DebugInfoExportStage::Preparing,
		processed_bytes: 0,
		total_bytes: None,
	});
	let state = tokio::select! {
		_ = cancel.cancelled() => return Ok(false),
		state = State::get() => state?,
	};
	let directories = state.directories.clone();
	let database_files = tempfile::tempdir()?;
	let transaction = tokio::select! {
		_ = cancel.cancelled() => return Ok(false),
		transaction = state.pool.begin_with("BEGIN IMMEDIATE") => transaction?,
	};
	let mut buffer = vec![0; 64 * 1024];
	for name in ["app.db", "app.db-wal", "app.db-shm", "app.db-journal"] {
		let source = directories.settings_dir.join(name);
		let mut input = match tokio::fs::File::open(&source).await {
			Ok(file) => file,
			Err(error)
				if name != "app.db"
					&& error.kind() == std::io::ErrorKind::NotFound => continue,
			Err(error) => return Err(IOError::with_path(error, &source).into()),
		};
		let mut output =
			tokio::fs::File::create(database_files.path().join(name)).await?;
		loop {
			if cancel.is_cancelled() {
				return Ok(false);
			}
			let count = input.read(&mut buffer).await?;
			if count == 0 {
				break;
			}
			output.write_all(&buffer[..count]).await?;
		}
		output.flush().await?;
	}
	transaction.rollback().await?;

	tokio::task::spawn_blocking(move || -> crate::Result<bool> {
		let parent = export_path.parent().ok_or_else(|| {
			crate::ErrorKind::InputError(
				"Export path has no parent directory".to_string(),
			)
		})?;
		let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
		let excluded_paths = [
			std::fs::canonicalize(temporary.path())?,
			std::fs::canonicalize(&export_path)
				.unwrap_or_else(|_| export_path.clone()),
		];
		let mut files = Vec::new();
		collect_files(
			&mut files,
			database_files.path(),
			"database",
			&excluded_paths,
			&cancel,
		)?;
		if let Some(logs_dir) = directories.launcher_logs_dir() {
			collect_files(
				&mut files,
				&logs_dir,
				"launcher_logs",
				&excluded_paths,
				&cancel,
			)?;
		}
		let instances_dir = directories.instances_dir();
		match std::fs::read_dir(&instances_dir) {
			Ok(entries) => {
				for entry in entries {
					if cancel.is_cancelled() {
						return Ok(false);
					}
					let entry = entry?;
					if !entry.file_type()?.is_dir() {
						continue;
					}
					let instance_dir = entry.path();
					let prefix = format!(
						"instances/{}",
						entry.file_name().to_string_lossy()
					);
					for name in ["logs", "crash-reports"] {
						collect_files(
							&mut files,
							&instance_dir.join(name),
							&format!("{prefix}/{name}"),
							&excluded_paths,
							&cancel,
						)?;
					}
					for file in std::fs::read_dir(&instance_dir)? {
						if cancel.is_cancelled() {
							return Ok(false);
						}
						let file = file?;
						let name = file.file_name();
						let name = name.to_string_lossy();
						if name.starts_with("hs_err_pid") && name.ends_with(".log") {
							collect_files(
								&mut files,
								&file.path(),
								&format!("{prefix}/{name}"),
								&excluded_paths,
								&cancel,
							)?;
						}
					}
				}
			}
			Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
			Err(error) => return Err(IOError::with_path(error, &instances_dir).into()),
		}
		if cancel.is_cancelled() {
			return Ok(false);
		}

		let total_bytes = files.iter().map(|file| file.length).sum();
		let mut progress = DebugInfoExportProgress {
			stage: DebugInfoExportStage::Exporting,
			processed_bytes: 0,
			total_bytes: Some(total_bytes),
		};
		on_progress(progress);
		let mut last_sent = Instant::now();
		let mut archive = ZipWriter::new(BufWriter::new(temporary.as_file_mut()));
		for file in files {
			if cancel.is_cancelled() {
				return Ok(false);
			}
			let input = match File::open(&file.path) {
				Ok(input) => input,
				Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
					progress.total_bytes = progress.total_bytes.map(|total| total - file.length);
					continue;
				}
				Err(error) => return Err(IOError::with_path(error, &file.path).into()),
			};
			archive
				.start_file(
					&file.name,
					file_options().large_file(file.length >= zip::ZIP64_BYTES_THR),
				)
				.map_err(std::io::Error::from)?;
			let mut input = input.take(file.length);
			let mut copied = 0;
			loop {
				if cancel.is_cancelled() {
					return Ok(false);
				}
				let count = std::io::Read::read(&mut input, &mut buffer)
					.map_err(|error| IOError::with_path(error, &file.path))?;
				if count == 0 {
					break;
				}
				archive.write_all(&buffer[..count])?;
				copied += count as u64;
				progress.processed_bytes += count as u64;
				if last_sent.elapsed() >= Duration::from_millis(200) {
					on_progress(progress);
					last_sent = Instant::now();
				}
			}
			progress.total_bytes = progress.total_bytes
				.map(|total| total - (file.length - copied));
		}
		if cancel.is_cancelled() {
			return Ok(false);
		}
		progress.stage = DebugInfoExportStage::Finishing;
		on_progress(progress);
		archive
			.start_file("environment.json", file_options())
			.map_err(std::io::Error::from)?;
		serde_json::to_writer_pretty(&mut archive, &serde_json::json!({
			"exported_at": chrono::Utc::now(),
			"app_version": env!("CARGO_PKG_VERSION"),
			"app_identifier": directories.app_identifier,
			"os": std::env::consts::OS,
			"os_version": sysinfo::System::long_os_version(),
			"kernel_version": sysinfo::System::kernel_version(),
			"architecture": std::env::consts::ARCH,
			"settings_dir": directories.settings_dir,
			"config_dir": directories.config_dir,
		}))?;
		archive.finish().map_err(std::io::Error::from)?.flush()?;
		temporary.as_file().sync_all()?;
		if cancel.is_cancelled() {
			return Ok(false);
		}
		temporary.persist(&export_path).map_err(|error| {
			IOError::with_path(error.error, &export_path)
		})?;
		Ok(true)
	})
	.await?
}

fn file_options() -> SimpleFileOptions {
	SimpleFileOptions::default()
		.compression_method(CompressionMethod::Deflated)
		.unix_permissions(0o600)
}

fn collect_files(
	files: &mut Vec<ExportFile>,
	path: &Path,
	archive_name: &str,
	excluded_paths: &[PathBuf],
	cancel: &CancellationToken,
) -> crate::Result<()> {
	if cancel.is_cancelled() {
		return Ok(());
	}
	let metadata = match std::fs::symlink_metadata(path) {
		Ok(metadata) => metadata,
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
		Err(error) => return Err(IOError::with_path(error, path).into()),
	};
	if metadata.is_dir() {
		let entries = std::fs::read_dir(path)
			.map_err(|error| IOError::with_path(error, path))?;
		for entry in entries {
			if cancel.is_cancelled() {
				return Ok(());
			}
			let entry = entry?;
			collect_files(
				files,
				&entry.path(),
				&format!("{archive_name}/{}", entry.file_name().to_string_lossy()),
				excluded_paths,
				cancel,
			)?;
		}
	} else if metadata.is_file()
		&& !std::fs::canonicalize(path).is_ok_and(|path| excluded_paths.contains(&path))
	{
		files.push(ExportFile {
			path: path.to_path_buf(),
			name: archive_name.to_string(),
			length: metadata.len(),
		});
	}
	Ok(())
}
