// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;

use crate::mw::{MwError, SubscribeQos, Subscriber};

use super::qos::apply_subscribe_qos;
use super::session::ZenohSession;

type ZenohFifoSubscriber =
    zenoh::pubsub::Subscriber<zenoh::handlers::FifoChannelHandler<zenoh::sample::Sample>>;

pub(crate) struct ZenohSubscriber {
    subscriber: ZenohFifoSubscriber,
}

impl ZenohSubscriber {
    pub(crate) async fn declare(
        session: &ZenohSession,
        topic: String,
        qos: SubscribeQos,
    ) -> Result<Self, MwError> {
        let builder = session.inner().declare_subscriber(topic);
        let subscriber = apply_subscribe_qos(qos, builder)
            .await
            .map_err(|e| MwError::from(e.to_string()))?;

        Ok(Self { subscriber })
    }
}

#[async_trait]
impl Subscriber for ZenohSubscriber {
    async fn recv(&mut self) -> Result<Option<Vec<u8>>, MwError> {
        match self.subscriber.recv_async().await {
            Ok(sample) => Ok(Some(sample.payload().to_bytes().to_vec())),
            Err(e) => Err(MwError::from(e.to_string())),
        }
    }
}
