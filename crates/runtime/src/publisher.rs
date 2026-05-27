// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use std::marker::PhantomData;

use zos_msg::Message;

use crate::codec;
use crate::node::Node;
use crate::qos::Qos;
use crate::RuntimeError;

pub struct Publisher<T> {
    _session: zenoh::Session,
    publisher: zenoh::pubsub::Publisher<'static>,
    topic: String,
    _marker: PhantomData<T>,
}

/// Builder for [`Publisher`] created from a [`Node`].
pub struct PublisherBuilder<'a, T> {
    node: &'a Node,
    topic: String,
    qos: Qos,
    _marker: PhantomData<T>,
}

impl<'a, T> PublisherBuilder<'a, T> {
    pub(crate) fn new(node: &'a Node, topic: String) -> Self {
        Self {
            node,
            topic,
            qos: Qos::default(),
            _marker: PhantomData,
        }
    }
}

impl<T> Publisher<T>
where
    T: Message,
{
    /// Construct a publisher for `topic` on the given Zenoh session.
    ///
    /// Prefer [`Node::create_publisher`](crate::Node::create_publisher) in application code.
    pub async fn new(session: zenoh::Session, topic: impl Into<String>) -> Result<Self, RuntimeError> {
        Self::new_with_qos(session, topic, Qos::default()).await
    }

    /// Create a publisher with an explicit QoS profile.
    pub async fn new_with_qos(
        session: zenoh::Session,
        topic: impl Into<String>,
        qos: Qos,
    ) -> Result<Self, RuntimeError> {
        let topic = topic.into();
        let builder = session.declare_publisher(topic.clone());
        let publisher = qos
            .publish
            .apply(builder)
            .await
            .map_err(|e| RuntimeError::from(e.to_string()))?;

        Ok(Self {
            _session: session,
            publisher,
            topic,
            _marker: PhantomData,
        })
    }

    /// Serialize and publish a message to the topic.
    pub async fn publish(&self, message: &T) -> Result<(), RuntimeError> {
        let payload = codec::encode(message)?;

        self.publisher
            .put(payload)
            .await
            .map_err(|e| {
                RuntimeError::from(format!(
                    "Zenoh put failed on topic {}: {e}",
                    self.topic
                ))
            })?;

        Ok(())
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }
}

impl<'a, T> PublisherBuilder<'a, T>
where
    T: Message,
{
    pub fn qos(mut self, qos: Qos) -> Self {
        self.qos = qos;
        self
    }

    pub async fn build(self) -> Result<Publisher<T>, RuntimeError> {
        Publisher::new_with_qos(self.node.session().clone(), self.topic, self.qos).await
    }
}
