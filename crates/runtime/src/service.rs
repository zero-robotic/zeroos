// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;

use async_trait::async_trait;
use zos_msg::Message;

use crate::codec;
use crate::node::Node;
use crate::{Runnable, RuntimeError};

pub type ServiceHandlerFuture<Resp> =
    Pin<Box<dyn Future<Output = Result<Resp, RuntimeError>> + Send>>;
pub type ServiceHandler<Req, Resp> =
    dyn Fn(Req) -> ServiceHandlerFuture<Resp> + Send + Sync;

/// Service: handles requests via Zenoh queryable (ROS 2 `rclcpp::Service`).
pub struct Service<Req, Resp> {
    _session: zenoh::Session,
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
    pub fn new<F, Fut>(
        session: zenoh::Session,
        topic: impl Into<String>,
        handler: F,
    ) -> Self
    where
        F: Fn(Req) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Resp, RuntimeError>> + Send + 'static,
    {
        Self {
            _session: session,
            topic: topic.into(),
            handler: std::sync::Arc::new(move |req| Box::pin(handler(req))),
        }
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
    pub fn handler<F, Fut>(self, handler: F) -> Service<Req, Resp>
    where
        F: Fn(Req) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Resp, RuntimeError>> + Send + 'static,
    {
        Service::new(self.node.session().clone(), self.name, handler)
    }

    /// Build a service and append it to [`Node::runnables`] for the executor.
    pub fn register<F, Fut>(self, handler: F)
    where
        F: Fn(Req) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Resp, RuntimeError>> + Send + 'static,
    {
        let service = Service::new(self.node.session().clone(), self.name, handler);
        self.node.add_runnable(Box::new(service));
    }
}

#[async_trait]
impl<Req, Resp> Runnable for Service<Req, Resp>
where
    Req: Message,
    Resp: Message,
{
    async fn run(&mut self) -> Result<(), RuntimeError> {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let handler = std::sync::Arc::clone(&self.handler);
        let topic = self.topic.clone();

        self._session
            .declare_queryable(&topic)
            .complete(true)
            .callback(move |query| {
                let _ = tx.send(query);
            })
            .background()
            .await
            .map_err(|e| RuntimeError::from(e.to_string()))?;

        while let Some(query) = rx.recv().await {
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
    query: zenoh::query::Query,
) -> Result<(), RuntimeError>
where
    Req: Message,
    Resp: Message,
{
    let request = match query.payload() {
        Some(payload) => codec::decode::<Req>(&payload.to_bytes())?,
        None => {
            return Err(RuntimeError::from("service request payload is empty"));
        }
    };

    let response = handler(request).await?;
    let payload = codec::encode(&response)?;

    query
        .reply(query.key_expr(), payload)
        .await
        .map_err(|e| RuntimeError::from(e.to_string()))?;

    Ok(())
}
