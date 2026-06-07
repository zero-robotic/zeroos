// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use zos_msg::Message;

use crate::codec;
use crate::context;
use crate::mw::{Session, SubscribeQos};
use crate::node::Node;
use crate::qos::Qos;
use crate::{Runnable, RuntimeError};

pub type CallbackFuture = Pin<Box<dyn Future<Output = ()> + Send>>;
pub type SubscriberCallback<T> = dyn Fn(T) -> CallbackFuture + Send + Sync;

pub struct Subscriber<T> {
    session: Arc<dyn Session>,
    topic: String,
    qos: SubscribeQos,
    callback: Box<SubscriberCallback<T>>,
}

/// Builder for [`Subscriber`] created from a [`Node`].
pub struct SubscriberBuilder<'a, T> {
    node: &'a mut Node,
    topic: String,
    qos: Qos,
    _marker: PhantomData<T>,
}

impl<'a, T> SubscriberBuilder<'a, T> {
    pub(crate) fn new(node: &'a mut Node, topic: String) -> Self {
        Self {
            node,
            topic,
            qos: Qos::default(),
            _marker: PhantomData,
        }
    }
}

impl<T> Subscriber<T>
where
    T: Message,
{
    /// Prefer [`Node::create_subscriber`](crate::Node::create_subscriber) in application code.
    pub fn new<F, Fut>(topic: impl Into<String>, callback: F) -> Result<Self, RuntimeError>
    where
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self::new_with_qos(topic, Qos::default(), callback)
    }

    pub fn new_with_qos<F, Fut>(
        topic: impl Into<String>,
        qos: Qos,
        callback: F,
    ) -> Result<Self, RuntimeError>
    where
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let session = context::session()?;
        Ok(Self {
            session,
            topic: topic.into(),
            qos: qos.subscribe,
            callback: Box::new(move |msg| Box::pin(callback(msg))),
        })
    }
}

impl<'a, T> SubscriberBuilder<'a, T>
where
    T: Message,
{
    pub fn qos(mut self, qos: Qos) -> Self {
        self.qos = qos;
        self
    }

    /// Build a subscriber without registering it on the node.
    pub fn callback<F, Fut>(self, callback: F) -> Result<Subscriber<T>, RuntimeError>
    where
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Subscriber::new_with_qos(self.topic, self.qos, callback)
    }

    /// Build a subscriber and append it to [`Node::runnables`] for the executor.
    pub fn register<F, Fut>(self, callback: F) -> Result<(), RuntimeError>
    where
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let subscriber = Subscriber::new_with_qos(self.topic, self.qos, callback)?;
        self.node.add_runnable(Box::new(subscriber));
        Ok(())
    }
}

#[async_trait]
impl<T> Runnable for Subscriber<T>
where
    T: Message,
{
    async fn run(&mut self) -> Result<(), RuntimeError> {
        let mut inner = self
            .session
            .declare_subscriber(self.topic.clone(), self.qos)
            .await?;

        while let Some(payload) = inner.recv().await? {
            if let Ok(message) = codec::decode::<T>(&payload) {
                (self.callback)(message).await;
            }
        }

        Ok(())
    }
}
