// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use futures_util::StreamExt;
use zos_msg::Message;

use crate::codec;
use crate::qos::Qos;
use crate::{Runnable, RuntimeError};

pub type CallbackFuture = Pin<Box<dyn Future<Output = ()> + Send>>;
pub type SubscriberCallback<T> = dyn Fn(T) -> CallbackFuture + Send + Sync;

pub struct Subscriber<T> {
    session: zenoh::Session,
    topic: String,
    qos: Qos,
    callback: Box<SubscriberCallback<T>>,
}

impl<T> Subscriber<T>
where
    T: Message,
{
    /// Prefer [`Node::create_subscriber`](crate::Node::create_subscriber) in application code.
    pub fn new<F, Fut>(session: zenoh::Session, topic: impl Into<String>, callback: F) -> Self
    where
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self::new_with_qos(session, topic, Qos::default(), callback)
    }

    /// Create a subscriber with an explicit QoS profile.
    pub fn new_with_qos<F, Fut>(
        session: zenoh::Session,
        topic: impl Into<String>,
        qos: Qos,
        callback: F,
    ) -> Self
    where
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self {
            session,
            topic: topic.into(),
            qos,
            callback: Box::new(move |msg| Box::pin(callback(msg))),
        }
    }
}

#[async_trait]
impl<T> Runnable for Subscriber<T>
where
    T: Message,
{
    async fn run(&mut self) -> Result<(), RuntimeError> {
        let builder = self.session.declare_subscriber(self.topic.clone());
        let subscriber = self
            .qos
            .subscribe
            .apply(builder)
            .await
            .map_err(|e| RuntimeError::from(e.to_string()))?;

        let mut stream = subscriber.stream();
        while let Some(sample) = stream.next().await {
            if let Ok(payload) = codec::decode::<T>(&sample.payload().to_bytes()) {
                (self.callback)(payload).await;
            }
        }

        Ok(())
    }
}
