// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

//! Pluggable middleware session and endpoint traits.

use async_trait::async_trait;

use super::error::MwError;
use super::qos::{PublishQos, SubscribeQos};

/// Process-wide middleware connection (pub/sub and services).
///
/// Runtime holds [`std::sync::Arc`]`<dyn Session>` from [`crate::context::session`].
#[async_trait]
pub trait Session: Send + Sync + 'static {
    /// Open a publication endpoint on `topic`.
    async fn declare_publisher(
        &self,
        topic: String,
        qos: PublishQos,
    ) -> Result<Box<dyn Publisher>, MwError>;

    /// Open a subscription endpoint on `topic`.
    async fn declare_subscriber(
        &self,
        topic: String,
        qos: SubscribeQos,
    ) -> Result<Box<dyn Subscriber>, MwError>;

    /// Open a service client endpoint on `topic`.
    async fn declare_querier(&self, topic: String) -> Result<Box<dyn Querier>, MwError>;

    /// Open a service server endpoint on `topic`.
    async fn declare_queryable(&self, topic: String) -> Result<Box<dyn Queryable>, MwError>;
}

/// Topic publisher handle.
#[async_trait]
pub trait Publisher: Send + Sync {
    fn topic(&self) -> &str;

    async fn put(&self, payload: Vec<u8>) -> Result<(), MwError>;
}

/// Topic subscriber handle.
#[async_trait]
pub trait Subscriber: Send {
    async fn recv(&mut self) -> Result<Option<Vec<u8>>, MwError>;
}

/// Service client handle.
#[async_trait]
pub trait Querier: Send + Sync {
    fn topic(&self) -> &str;

    async fn get(&self, payload: Vec<u8>) -> Result<Vec<u8>, MwError>;
}

/// One incoming service request.
pub struct IncomingQuery {
    pub payload: Vec<u8>,
    reply: Box<dyn QueryReply + Send + Sync>,
}

impl IncomingQuery {
    pub fn new(payload: Vec<u8>, reply: Box<dyn QueryReply + Send + Sync>) -> Self {
        Self { payload, reply }
    }

    pub async fn respond(&self, payload: Vec<u8>) -> Result<(), MwError> {
        self.reply.respond(payload).await
    }
}

/// Sends a service response for one [`IncomingQuery`].
#[async_trait]
pub trait QueryReply: Send + Sync {
    async fn respond(&self, payload: Vec<u8>) -> Result<(), MwError>;
}

/// Service server handle.
#[async_trait]
pub trait Queryable: Send {
    async fn recv(&mut self) -> Result<Option<IncomingQuery>, MwError>;
}
