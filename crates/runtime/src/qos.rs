// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

//! QoS profiles for publishers and subscribers.
//!
//! Defaults match Zenoh's built-in defaults. Most callers should use [`Node::create_publisher`](crate::Node::create_publisher)
//! / [`Node::create_subscriber`](crate::Node::create_subscriber) and only set QoS when needed.

use zenoh::qos::CongestionControl;
use zenoh::qos::Priority;
use zenoh::sample::Locality;

/// Combined QoS for publishing and subscribing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Qos {
    pub publish: PublishQos,
    pub subscribe: SubscribeQos,
}

impl Default for Qos {
    fn default() -> Self {
        Self {
            publish: PublishQos::default(),
            subscribe: SubscribeQos::default(),
        }
    }
}

/// Publisher-side QoS (mapped to Zenoh `declare_publisher` options).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublishQos {
    pub priority: MessagePriority,
    pub reliability: Reliability,
    /// When `true`, messages are not batched (lower latency, more overhead).
    pub express: bool,
}

impl Default for PublishQos {
    fn default() -> Self {
        Self {
            priority: MessagePriority::Normal,
            reliability: Reliability::BestEffort,
            express: false,
        }
    }
}

impl PublishQos {
    pub(crate) fn apply<'a, 'b>(
        self,
        builder: zenoh::pubsub::PublisherBuilder<'a, 'b>,
    ) -> zenoh::pubsub::PublisherBuilder<'a, 'b> {
        builder
            .priority(self.priority.into())
            .congestion_control(self.reliability.into())
            .express(self.express)
    }
}

/// Subscriber-side QoS (mapped to Zenoh `declare_subscriber` options).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubscribeQos {
    /// Which publishers this subscriber accepts (local session, remote, or both).
    pub origin: SubscriptionOrigin,
}

impl Default for SubscribeQos {
    fn default() -> Self {
        Self {
            origin: SubscriptionOrigin::Any,
        }
    }
}

impl SubscribeQos {
    pub(crate) fn apply<'a, 'b>(
        self,
        builder: zenoh::pubsub::SubscriberBuilder<'a, 'b, zenoh::handlers::DefaultHandler>,
    ) -> zenoh::pubsub::SubscriberBuilder<'a, 'b, zenoh::handlers::DefaultHandler> {
        builder.allowed_origin(self.origin.into())
    }
}

/// Message priority (simplified view of Zenoh priorities).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MessagePriority {
    Low,
    #[default]
    Normal,
    High,
}

impl From<MessagePriority> for Priority {
    fn from(value: MessagePriority) -> Self {
        match value {
            MessagePriority::Low => Priority::Background,
            MessagePriority::Normal => Priority::Data,
            MessagePriority::High => Priority::RealTime,
        }
    }
}

/// Delivery behavior under congestion.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Reliability {
    /// Drop when queues are full (Zenoh default).
    #[default]
    BestEffort,
    /// Block until the message can be sent.
    Reliable,
}

impl From<Reliability> for CongestionControl {
    fn from(value: Reliability) -> Self {
        match value {
            Reliability::BestEffort => CongestionControl::Drop,
            Reliability::Reliable => CongestionControl::Block,
        }
    }
}

/// Where matching publications may originate.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SubscriptionOrigin {
    SessionLocal,
    Remote,
    #[default]
    Any,
}

impl From<SubscriptionOrigin> for Locality {
    fn from(value: SubscriptionOrigin) -> Self {
        match value {
            SubscriptionOrigin::SessionLocal => Locality::SessionLocal,
            SubscriptionOrigin::Remote => Locality::Remote,
            SubscriptionOrigin::Any => Locality::Any,
        }
    }
}
