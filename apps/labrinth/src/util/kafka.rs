use std::{ops::Deref, sync::Mutex};

use crate::{env::ENV, util::error::Context};
use kafka::client::KafkaClient;

pub const SEARCH_PROJECT_INDEX_QUEUE_TOPIC: &str =
    "public.labrinth.search_project_index_queue.v1";

#[derive(Debug)]
pub struct KafkaClientState {
    client: Mutex<KafkaClient>,
}

impl KafkaClientState {
    pub fn new() -> eyre::Result<Self> {
        let mut client =
            KafkaClient::new(ENV.KAFKA_BOOTSTRAP_SERVERS.0.clone());
        client.set_client_id(ENV.KAFKA_CLIENT_ID.clone());
        client
            .load_metadata(&[SEARCH_PROJECT_INDEX_QUEUE_TOPIC])
            .wrap_err("failed to load Kafka metadata")?;

        let topic_names = client
            .topics()
            .names()
            .map(str::to_string)
            .collect::<Vec<_>>();

        tracing::info!(
            kafka.bootstrap_servers = ?client.hosts(),
            kafka.client_id = %client.client_id(),
            kafka.topic_count = topic_names.len(),
            kafka.topics = ?topic_names,
            "Connected to Kafka"
        );

        Ok(Self {
            client: Mutex::new(client),
        })
    }
}

impl Deref for KafkaClientState {
    type Target = Mutex<KafkaClient>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}
