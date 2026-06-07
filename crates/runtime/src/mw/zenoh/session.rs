// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;

use crate::mw::{
    MwError, Publisher, PublishQos, Querier, Queryable, Session, SubscribeQos, Subscriber,
};

use super::publisher::ZenohPublisher;
use super::querier::ZenohQuerier;
use super::queryable::ZenohQueryable;
use super::subscriber::ZenohSubscriber;

/// Zenoh-backed [`Session`].
#[derive(Clone)]
pub struct ZenohSession {
    session: zenoh::Session,
}

impl ZenohSession {
    pub(crate) fn new(session: zenoh::Session) -> Self {
        Self { session }
    }

    pub(crate) fn inner(&self) -> &zenoh::Session {
        &self.session
    }
}

#[async_trait]
impl Session for ZenohSession {
    async fn declare_publisher(
        &self,
        topic: String,
        qos: PublishQos,
    ) -> Result<Box<dyn Publisher>, MwError> {
        Ok(Box::new(
            ZenohPublisher::declare(self, topic, qos).await?,
        ))
    }

    async fn declare_subscriber(
        &self,
        topic: String,
        qos: SubscribeQos,
    ) -> Result<Box<dyn Subscriber>, MwError> {
        Ok(Box::new(
            ZenohSubscriber::declare(self, topic, qos).await?,
        ))
    }

    async fn declare_querier(&self, topic: String) -> Result<Box<dyn Querier>, MwError> {
        Ok(Box::new(ZenohQuerier::declare(self, topic).await?))
    }

    async fn declare_queryable(&self, topic: String) -> Result<Box<dyn Queryable>, MwError> {
        Ok(Box::new(ZenohQueryable::declare(self, topic).await?))
    }
}
