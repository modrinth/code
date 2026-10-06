use std::sync::Arc;

use actix_web::web;
use async_trait::async_trait;
use bytes::Bytes;
use eyre::{Result, WrapErr, eyre};
use rdkafka::{producer::FutureRecord, util::Timeout};
use serde::Serialize;

use super::{
    DeleteFileData, FileHost, FileHostPublicity, FileHostingError,
    UploadFileData,
};
use crate::util::kafka::{
    KAFKA_OPERATION_INTERVAL, KafkaClientState, KafkaEvent,
};

pub const FILE_UPLOADED_TOPIC: &str = "public.labrinth.file-uploaded.v1";

#[derive(Debug, Serialize)]
struct FileUploadedEvent<'a> {
    file_path: &'a str,
    file_publicity: FileHostPublicity,
    content_length: u32,
    content_sha1: &'a str,
    content_sha512: &'a str,
}

/// Wraps a [`FileHost`] and publishes a [`FILE_UPLOADED_TOPIC`] event after
/// every successful upload.
pub struct KafkaFileHost {
    inner: Arc<dyn FileHost>,
    kafka_client: web::Data<KafkaClientState>,
}

impl KafkaFileHost {
    pub fn new(
        inner: Arc<dyn FileHost>,
        kafka_client: web::Data<KafkaClientState>,
    ) -> Self {
        Self {
            inner,
            kafka_client,
        }
    }

    async fn publish_upload(&self, data: &UploadFileData) -> Result<()> {
        let event = KafkaEvent::new(
            FILE_UPLOADED_TOPIC,
            FileUploadedEvent {
                file_path: &data.file_name,
                file_publicity: data.file_publicity,
                content_length: data.content_length,
                content_sha1: &data.content_sha1,
                content_sha512: &data.content_sha512,
            },
        );
        let payload = serde_json::to_vec(&event)
            .wrap_err("serializing file uploaded event")?;
        let record = FutureRecord::to(FILE_UPLOADED_TOPIC)
            .key(&data.file_name)
            .payload(&payload);

        self.kafka_client
            .client
            .send(record, Timeout::After(KAFKA_OPERATION_INTERVAL))
            .await
            .map_err(|(err, _)| eyre!(err))
            .wrap_err("publishing file uploaded event")?;

        Ok(())
    }
}

#[async_trait]
impl FileHost for KafkaFileHost {
    async fn upload_file(
        &self,
        content_type: &str,
        file_name: &str,
        file_publicity: FileHostPublicity,
        file_bytes: Bytes,
    ) -> Result<UploadFileData, FileHostingError> {
        let data = self
            .inner
            .upload_file(content_type, file_name, file_publicity, file_bytes)
            .await?;

        // The file is already stored, so failing the upload here would only
        // orphan it.
        if let Err(err) = self.publish_upload(&data).await {
            tracing::error!(file_name, "{err:#}");
        }

        Ok(data)
    }

    async fn get_url_for_private_file(
        &self,
        file_name: &str,
        expiry_secs: u32,
    ) -> Result<String, FileHostingError> {
        self.inner
            .get_url_for_private_file(file_name, expiry_secs)
            .await
    }

    async fn delete_file(
        &self,
        file_name: &str,
        file_publicity: FileHostPublicity,
    ) -> Result<DeleteFileData, FileHostingError> {
        self.inner.delete_file(file_name, file_publicity).await
    }

    async fn read_file(
        &self,
        file_name: &str,
        file_publicity: FileHostPublicity,
    ) -> Result<Bytes, FileHostingError> {
        self.inner.read_file(file_name, file_publicity).await
    }
}
