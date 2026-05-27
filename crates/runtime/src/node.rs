// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use std::marker::PhantomData;

use zos_msg::Message;

use crate::executor::Executor;
use crate::qos::Qos;
use std::time::Duration;

use crate::{Publisher, Runnable, RuntimeError, Subscriber, Timer};

/// A ZeroOS node: owns the Zenoh [`Session`] and creates publishers/subscribers.
pub struct Node {
    pub name: String,
    session: zenoh::Session,
    /// Runnable components (subscribers, timers, …) for the [`Executor`](crate::Executor) to drive.
    pub runnables: Vec<Box<dyn Runnable + Send>>,
}

/// Builder for [`Publisher`] created from a [`Node`].
pub struct PublisherBuilder<'a, T> {
    node: &'a Node,
    topic: String,
    qos: Qos,
    _marker: PhantomData<T>,
}

/// Builder for [`Subscriber`] created from a [`Node`].
pub struct SubscriberBuilder<'a, T> {
    node: &'a mut Node,
    topic: String,
    qos: Qos,
    _marker: PhantomData<T>,
}

impl Node {
    /// Open a node with the default Zenoh configuration.
    pub async fn open_default() -> Result<Self, RuntimeError> {
        Self::open(zenoh::Config::default()).await
    }

    /// Open a node with the given Zenoh configuration (unnamed).
    pub async fn open(config: zenoh::Config) -> Result<Self, RuntimeError> {
        Self::open_named("", config).await
    }

    /// Open a named node. Topics are prefixed with `name/` when `name` is non-empty.
    pub async fn open_named(name: impl Into<String>, config: zenoh::Config) -> Result<Self, RuntimeError> {
        let session = zenoh::open(config)
            .await
            .map_err(|e| RuntimeError::from(e.to_string()))?;

        Ok(Self {
            name: name.into(),
            session,
            runnables: Vec::new(),
        })
    }

    /// Register a runnable component to be driven by an executor.
    pub fn add_runnable(&mut self, runnable: Box<dyn Runnable + Send>) {
        self.runnables.push(runnable);
    }

    /// Take registered runnables and run them concurrently until completion or error.
    ///
    /// Requires a Tokio runtime (e.g. `#[tokio::main]`).
    pub async fn spin(&mut self) -> Result<(), RuntimeError> {
        let mut executor = Executor::new();
        executor.add_node(self);
        executor.spin().await
    }

    /// Access the underlying Zenoh session (e.g. for advanced Zenoh APIs).
    pub fn session(&self) -> &zenoh::Session {
        &self.session
    }

    /// Start building a publisher on `topic` (default QoS).
    pub fn create_publisher<T: Message>(&self, topic: impl AsRef<str>) -> PublisherBuilder<'_, T> {
        PublisherBuilder {
            node: self,
            topic: self.resolve_topic(topic.as_ref()),
            qos: Qos::default(),
            _marker: PhantomData,
        }
    }

    /// Create a subscriber with default QoS (does not register with the node).
    pub fn create_subscriber<T, F, Fut>(
        &mut self,
        topic: impl AsRef<str>,
        callback: F,
    ) -> Subscriber<T>
    where
        T: Message,
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.create_subscriber_builder::<T>(topic).callback(callback)
    }

    /// Start building a subscriber on `topic` (default QoS; set callback on the builder).
    pub fn create_subscriber_builder<T: Message>(
        &mut self,
        topic: impl AsRef<str>,
    ) -> SubscriberBuilder<'_, T> {
        let topic = self.resolve_topic(topic.as_ref());
        SubscriberBuilder {
            node: self,
            topic,
            qos: Qos::default(),
            _marker: PhantomData,
        }
    }

    /// Create a periodic timer (does not register with the node).
    pub fn create_timer<F, Fut>(&mut self, period: Duration, callback: F) -> Timer
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.create_timer_builder(period).callback(callback)
    }

    /// Start building a timer (`period` between ticks; default: periodic).
    pub fn create_timer_builder(&mut self, period: Duration) -> TimerBuilder<'_> {
        TimerBuilder {
            node: self,
            period,
            one_shot: false,
        }
    }

    fn resolve_topic(&self, topic: &str) -> String {
        let topic = topic.trim_start_matches('/');
        if self.name.is_empty() {
            topic.to_owned()
        } else {
            format!("{}/{}", self.name, topic)
        }
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
        Publisher::new_with_qos(self.node.session.clone(), self.topic, self.qos).await
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
    pub fn callback<F, Fut>(self, callback: F) -> Subscriber<T>
    where
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        Subscriber::new_with_qos(
            self.node.session.clone(),
            self.topic,
            self.qos,
            callback,
        )
    }

    /// Build a subscriber and append it to [`Node::runnables`] for the executor.
    pub fn register<F, Fut>(self, callback: F)
    where
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let subscriber = Subscriber::new_with_qos(
            self.node.session.clone(),
            self.topic,
            self.qos,
            callback,
        );
        self.node.runnables.push(Box::new(subscriber));
    }
}

/// Builder for [`Timer`] created from a [`Node`].
pub struct TimerBuilder<'a> {
    node: &'a mut Node,
    period: Duration,
    one_shot: bool,
}

impl<'a> TimerBuilder<'a> {
    /// Fire once after `period`, then stop.
    pub fn one_shot(mut self) -> Self {
        self.one_shot = true;
        self
    }

    /// Build a timer without registering it on the node.
    pub fn callback<F, Fut>(self, callback: F) -> Timer
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        if self.one_shot {
            Timer::one_shot(self.period, callback)
        } else {
            Timer::periodic(self.period, callback)
        }
    }

    /// Build a timer and append it to [`Node::runnables`] for the executor.
    pub fn register<F, Fut>(self, callback: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let timer = if self.one_shot {
            Timer::one_shot(self.period, callback)
        } else {
            Timer::periodic(self.period, callback)
        };
        self.node.runnables.push(Box::new(timer));
    }
}
