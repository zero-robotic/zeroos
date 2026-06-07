// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;

use crate::mw::{MwError, Publisher, PublishQos};

use super::qos::apply_publish_qos;
use super::session::ZenohSession;

pub(crate) struct ZenohPublisher {
    topic: String,
    publisher: zenoh::pubsub::Publisher<'static>,
}

impl ZenohPublisher {
    pub(crate) async fn declare(
        session: &ZenohSession,
        topic: String,
        qos: PublishQos,
    ) -> Result<Self, MwError> {
        let builder = session.inner().declare_publisher(topic.clone());
        let publisher = apply_publish_qos(qos, builder)
            .await
            .map_err(|e| MwError::from(e.to_string()))?;

        Ok(Self { topic, publisher })
    }
}

#[async_trait]
impl Publisher for ZenohPublisher {
    fn topic(&self) -> &str {
        &self.topic
    }

    async fn put(&self, payload: Vec<u8>) -> Result<(), MwError> {
        self.publisher
            .put(payload)
            .await
            .map_err(|e| MwError::from(format!("put failed on {}: {e}", self.topic)))
    }
}
