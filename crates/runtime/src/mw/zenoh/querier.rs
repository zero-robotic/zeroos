// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use zenoh::query::QueryTarget;

use crate::mw::{MwError, Querier};

use super::session::ZenohSession;

pub(crate) struct ZenohQuerier {
    topic: String,
    querier: zenoh::query::Querier<'static>,
}

impl ZenohQuerier {
    pub(crate) async fn declare(session: &ZenohSession, topic: String) -> Result<Self, MwError> {
        let querier = session
            .inner()
            .declare_querier(topic.clone())
            .target(QueryTarget::BestMatching)
            .await
            .map_err(|e| MwError::from(e.to_string()))?;

        Ok(Self { topic, querier })
    }
}

#[async_trait]
impl Querier for ZenohQuerier {
    fn topic(&self) -> &str {
        &self.topic
    }

    async fn get(&self, payload: Vec<u8>) -> Result<Vec<u8>, MwError> {
        let replies = self
            .querier
            .get()
            .payload(payload)
            .await
            .map_err(|e| MwError::from(e.to_string()))?;

        let reply = replies.recv_async().await.map_err(|e| {
            let detail = e.to_string();
            if detail.contains("closed channel") {
                MwError::from(format!(
                    "no response from service `{}` (server not running or topic mismatch): {detail}",
                    self.topic
                ))
            } else {
                MwError::from(detail)
            }
        })?;

        match reply.result() {
            Ok(sample) => Ok(sample.payload().to_bytes().to_vec()),
            Err(err) => {
                let msg = if err.payload().is_empty() {
                    "service returned an error".to_owned()
                } else {
                    format!("service error: {:?}", err.payload())
                };
                Err(MwError::from(msg))
            }
        }
    }
}
