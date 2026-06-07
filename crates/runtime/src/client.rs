// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use std::marker::PhantomData;

use zos_msg::Message;
use crate::mw::Querier;

use crate::codec;
use crate::context;
use crate::RuntimeError;

/// Client: sends service requests via the middleware querier (ROS 2 `rclcpp::Client`).
pub struct Client<Req, Resp> {
    querier: Box<dyn Querier>,
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
        let querier = session.declare_querier(topic.clone()).await?;

        Ok(Self {
            querier,
            topic,
            _marker: PhantomData,
        })
    }

    pub async fn call(&self, request: &Req) -> Result<Resp, RuntimeError> {
        let payload = codec::encode(request)?;
        let response = self.querier.get(payload).await?;
        codec::decode::<Resp>(&response)
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }
}
