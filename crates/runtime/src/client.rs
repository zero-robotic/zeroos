// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use std::marker::PhantomData;

use zenoh::query::QueryTarget;
use zos_msg::Message;

use crate::codec;
use crate::context;
use crate::RuntimeError;

/// Client: sends service requests via Zenoh querier (ROS 2 `rclcpp::Client`).
pub struct Client<Req, Resp> {
    _session: zenoh::Session,
    querier: zenoh::query::Querier<'static>,
    topic: String,
    _marker: PhantomData<(Req, Resp)>,
}

impl<Req, Resp> Client<Req, Resp>
where
    Req: Message,
    Resp: Message,
{
    pub async fn new(topic: impl Into<String>) -> Result<Self, RuntimeError> {
        let session = context::session()?;
        let topic = topic.into();
        let querier = session
            .declare_querier(topic.clone())
            .target(QueryTarget::BestMatching)
            .await
            .map_err(|e| RuntimeError::from(e.to_string()))?;

        Ok(Self {
            _session: session,
            querier,
            topic,
            _marker: PhantomData,
        })
    }

    pub async fn call(&self, request: &Req) -> Result<Resp, RuntimeError> {
        let payload = codec::encode(request)?;

        let replies = self
            .querier
            .get()
            .payload(payload)
            .await
            .map_err(|e| RuntimeError::from(e.to_string()))?;

        let reply = replies.recv_async().await.map_err(|e| {
            let detail = e.to_string();
            if detail.contains("closed channel") {
                RuntimeError::from(format!(
                    "no response from service `{}` (server not running or topic mismatch): {detail}",
                    self.topic
                ))
            } else {
                RuntimeError::from(detail)
            }
        })?;

        match reply.result() {
            Ok(sample) => codec::decode::<Resp>(&sample.payload().to_bytes()),
            Err(err) => {
                let msg = if err.payload().is_empty() {
                    "service returned an error".to_owned()
                } else {
                    format!("service error: {:?}", err.payload())
                };
                Err(RuntimeError::from(msg))
            }
        }
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }
}
