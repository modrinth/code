use sha2::{Digest, Sha512};
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncWrite, BufReader, copy_buf};
use tokio_util::io::InspectReader;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FileHashes {
    pub sha512: String,
    pub size: u64,
}

#[derive(Default)]
pub(crate) struct ContentHasher {
    sha512: Sha512,
}

impl ContentHasher {
    pub(crate) fn update(&mut self, bytes: &[u8]) {
        self.sha512.update(bytes);
    }

	pub(crate) fn finish(self, size: u64) -> FileHashes {
		FileHashes {
			sha512: format!("{:x}", self.sha512.finalize()),
			size,
		}
	}

}

pub(crate) async fn temporary_file(
    directory: Option<&Path>,
) -> crate::Result<(File, tempfile::TempPath)> {
    let directory = directory.map(Path::to_path_buf);
    let (file, path) = tokio::task::spawn_blocking(move || {
        match directory {
            Some(directory) => tempfile::NamedTempFile::new_in(directory),
            None => tempfile::NamedTempFile::new(),
        }
        .map(tempfile::NamedTempFile::into_parts)
    })
    .await??;
    Ok((File::from_std(file), path))
}

pub(crate) async fn hash_file(path: &Path) -> crate::Result<FileHashes> {
    hash_file_with_progress(path, &|_| {}).await
}

pub(crate) async fn hash_file_with_progress(
    path: &Path,
    on_read: &(dyn Fn(u64) + Send + Sync),
) -> crate::Result<FileHashes> {
    let file = File::open(path).await?;
	copy_and_hash(file, &mut tokio::io::sink(), on_read).await
}

pub(crate) async fn copy_and_hash(
    input: impl AsyncRead + Unpin,
    output: &mut (impl AsyncWrite + Unpin),
    on_read: &(dyn Fn(u64) + Send + Sync),
) -> crate::Result<FileHashes> {
    let mut hashes = ContentHasher::default();
    let size = {
        let reader = InspectReader::new(input, |bytes| {
            hashes.update(bytes);
            on_read(bytes.len() as u64);
        });
        let mut reader = BufReader::with_capacity(256 * 1024, reader);
        copy_buf(&mut reader, output).await?
    };
    Ok(hashes.finish(size))
}
