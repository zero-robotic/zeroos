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
use crate::mw::Session;
use crate::node::Node;
use crate::{Runnable, RuntimeError};

pub type ServiceHandlerFuture<Resp> =
    Pin<Box<dyn Future<Output = Result<Resp, RuntimeError>> + Send>>;
pub type ServiceHandler<Req, Resp> =
    dyn Fn(Req) -> ServiceHandlerFuture<Resp> + Send + Sync;

/// Service: handles requests via middleware queryable (ROS 2 `rclcpp::Service`).
pub struct Service<Req, Resp> {
    session: Arc<dyn Session>,
    topic: String,
    handler: std::sync::Arc<ServiceHandler<Req, Resp>>,
}

/// Builder for [`Service`] created from a [`Node`].
pub struct ServiceBuilder<'a, Req, Resp> {
    node: &'a mut Node,
    name: String,
    _marker: PhantomData<(Req, Resp)>,
}

impl<'a, Req, Resp> ServiceBuilder<'a, Req, Resp> {
    pub(crate) fn new(node: &'a mut Node, name: String) -> Self {
        Self {
            node,
            name,
            _marker: PhantomData,
        }
    }
}

impl<Req, Resp> Service<Req, Resp>
where
    Req: Message,
    Resp: Message,
{
    pub fn new<F, Fut>(topic: impl Into<String>, handler: F) -> Result<Self, RuntimeError>
    where
        F: Fn(Req) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Resp, RuntimeError>> + Send + 'static,
    {
        let session = context::session()?;
        Ok(Self {
            session,
            topic: topic.into(),
            handler: std::sync::Arc::new(move |req| Box::pin(handler(req))),
        })
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }
}

impl<'a, Req, Resp> ServiceBuilder<'a, Req, Resp>
where
    Req: Message,
    Resp: Message,
{
    /// Build a service without registering it on the node.
    pub fn handler<F, Fut>(self, handler: F) -> Result<Service<Req, Resp>, RuntimeError>
    where
        F: Fn(Req) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Resp, RuntimeError>> + Send + 'static,
    {
        Service::new(self.name, handler)
    }

    /// Build a service and append it to [`Node::runnables`] for the executor.
    pub fn register<F, Fut>(self, handler: F) -> Result<(), RuntimeError>
    where
        F: Fn(Req) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Resp, RuntimeError>> + Send + 'static,
    {
        let service = Service::new(self.name, handler)?;
        self.node.add_runnable(Box::new(service));
        Ok(())
    }
}

#[async_trait]
impl<Req, Resp> Runnable for Service<Req, Resp>
where
    Req: Message,
    Resp: Message,
{
    async fn run(&mut self) -> Result<(), RuntimeError> {
        let handler = std::sync::Arc::clone(&self.handler);
        let topic = self.topic.clone();
        let mut queryable = self.session.declare_queryable(topic.clone()).await?;

        while let Some(query) = queryable.recv().await? {
            let handler = std::sync::Arc::clone(&handler);
            let topic_log = topic.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_query::<Req, Resp>(handler, query).await {
                    eprintln!("service {topic_log} handler error: {e}");
                }
            });
        }

        Ok(())
    }
}

async fn handle_query<Req, Resp>(
    handler: std::sync::Arc<ServiceHandler<Req, Resp>>,
    query: crate::mw::IncomingQuery,
) -> Result<(), RuntimeError>
where
    Req: Message,
    Resp: Message,
{
    if query.payload.is_empty() {
        return Err(RuntimeError::from("service request payload is empty"));
    }

    let request = codec::decode::<Req>(&query.payload)?;
    let response = handler(request).await?;
    let payload = codec::encode(&response)?;
    query.respond(payload).await?;
    Ok(())
}
