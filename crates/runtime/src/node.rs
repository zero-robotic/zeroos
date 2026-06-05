// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use zos_msg::Message;

use crate::context;
use crate::executor::{Executor, ExecutorOptions};
use crate::publisher::PublisherBuilder;
use crate::service::ServiceBuilder;
use crate::subscriber::SubscriberBuilder;
use crate::timer::TimerBuilder;
use crate::{Client, Runnable, RuntimeError, Subscriber, Timer};

/// A ZeroOS node: owns the Zenoh [`Session`] and creates publishers/subscribers.
///
/// Topic/service names are resolved like ROS 2: relative names are prefixed with
/// [`namespace`](Self::namespace) (default `/`); names starting with `/` are global.
pub struct Node {
    /// Node name (identity only; not prepended to topics).
    pub name: String,
    /// Normalized namespace without leading `/`; empty means root `/`.
    pub namespace: String,
    executor_options: ExecutorOptions,
    session: zenoh::Session,
    /// Runnable components (subscribers, timers, …) for the [`Executor`](crate::Executor) to drive.
    pub runnables: Vec<Box<dyn Runnable + Send>>,
}

/// Normalize a ROS 2-style namespace (`/`, `/robot`, `robot` → internal form).
pub fn normalize_namespace(namespace: &str) -> String {
    let namespace = namespace.trim();
    if namespace.is_empty() || namespace == "/" {
        String::new()
    } else {
        namespace.trim_matches('/').to_owned()
    }
}

/// Resolve a topic/service name under `namespace` (ROS 2 rules).
pub fn resolve_name(namespace: &str, name: &str) -> String {
    let name = name.trim();
    if name.starts_with('/') {
        return name.trim_start_matches('/').to_owned();
    }
    let name = name.trim_start_matches('/');
    let namespace = normalize_namespace(namespace);
    if namespace.is_empty() {
        name.to_owned()
    } else if name.is_empty() {
        namespace
    } else {
        format!("{namespace}/{name}")
    }
}

/// Options for [`Node::new`] (node name, namespace, executor). Zenoh config is set via [`crate::init`].
#[derive(Debug, Clone)]
pub struct NodeOptions {
    /// Node name (identity only).
    pub name: String,
    /// ROS 2 namespace (default `/`).
    pub namespace: String,
    pub executor: ExecutorOptions,
}

impl Default for NodeOptions {
    fn default() -> Self {
        Self {
            name: String::new(),
            namespace: "/".to_owned(),
            executor: ExecutorOptions::default(),
        }
    }
}

impl NodeOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = namespace.into();
        self
    }

    pub fn executor(mut self, executor: ExecutorOptions) -> Self {
        self.executor = executor;
        self
    }

    pub fn executor_worker_threads(mut self, n: usize) -> Self {
        self.executor.worker_threads = Some(n);
        self
    }
}

impl Node {
    /// Create a node using the global session from [`crate::init`].
    pub async fn new(options: NodeOptions) -> Result<Self, RuntimeError> {
        let session = context::session()?;

        Ok(Self {
            name: options.name,
            namespace: normalize_namespace(&options.namespace),
            executor_options: options.executor,
            session,
            runnables: Vec::new(),
        })
    }

    /// Fully qualified namespace string (`/` when root).
    pub fn fq_namespace(&self) -> String {
        if self.namespace.is_empty() {
            "/".to_owned()
        } else {
            format!("/{}", self.namespace)
        }
    }

    /// Register a runnable component to be driven by an executor.
    pub fn add_runnable(&mut self, runnable: Box<dyn Runnable + Send>) {
        self.runnables.push(runnable);
    }

    /// Run registered runnables (executor settings from [`NodeOptions::executor`] at creation).
    pub async fn spin(&mut self) -> Result<(), RuntimeError> {
        let mut executor = Executor::new(self.executor_options.clone());
        executor.add_node(self);
        executor.spin().await
    }

    /// Access the underlying Zenoh session (e.g. for advanced Zenoh APIs).
    pub fn session(&self) -> &zenoh::Session {
        &self.session
    }

    /// Start building a publisher on `topic` (resolved under [`namespace`](Self::namespace)).
    pub fn create_publisher<T: Message>(&self, topic: impl AsRef<str>) -> PublisherBuilder<'_, T> {
        PublisherBuilder::new(self, self.resolve_name(topic.as_ref()))
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
        SubscriberBuilder::new(self, self.resolve_name(topic.as_ref()))
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
        TimerBuilder::new(self, period)
    }

    /// Create a service client; `name` is relative to namespace or absolute if it starts with `/`.
    pub async fn create_client<Req, Resp>(
        &self,
        name: impl AsRef<str>,
    ) -> Result<Client<Req, Resp>, RuntimeError>
    where
        Req: Message,
        Resp: Message,
    {
        Client::new(self.session.clone(), self.resolve_name(name.as_ref())).await
    }

    /// Start building a service on `name`; resolved like other endpoints.
    pub fn create_service_builder<Req, Resp>(
        &mut self,
        name: impl AsRef<str>,
    ) -> ServiceBuilder<'_, Req, Resp> {
        ServiceBuilder::new(self, self.resolve_name(name.as_ref()))
    }

    pub(crate) fn resolve_name(&self, name: &str) -> String {
        resolve_name(&self.namespace, name)
    }
}

#[cfg(test)]
mod tests {
    use super::{normalize_namespace, resolve_name};

    #[test]
    fn normalize_namespace_root() {
        assert_eq!(normalize_namespace(""), "");
        assert_eq!(normalize_namespace("/"), "");
        assert_eq!(normalize_namespace("  /  "), "");
    }

    #[test]
    fn normalize_namespace_nested() {
        assert_eq!(normalize_namespace("/robot"), "robot");
        assert_eq!(normalize_namespace("robot/arm"), "robot/arm");
    }

    #[test]
    fn resolve_relative_in_root() {
        assert_eq!(resolve_name("/", "cmd_vel"), "cmd_vel");
        assert_eq!(resolve_name("", "scale"), "scale");
    }

    #[test]
    fn resolve_relative_in_namespace() {
        assert_eq!(resolve_name("/demo", "scale"), "demo/scale");
        assert_eq!(resolve_name("demo", "scale"), "demo/scale");
    }

    #[test]
    fn resolve_absolute_ignores_namespace() {
        assert_eq!(resolve_name("/demo", "/scale"), "scale");
        assert_eq!(resolve_name("robot", "/demo/scale"), "demo/scale");
    }
}
