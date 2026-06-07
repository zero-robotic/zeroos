// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

//! Middleware-neutral QoS profiles for publishers and subscribers.

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

/// Publisher-side QoS.
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

/// Subscriber-side QoS.
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

/// Message priority.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MessagePriority {
    Low,
    #[default]
    Normal,
    High,
}

/// Delivery behavior under congestion.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Reliability {
    /// Drop when queues are full.
    #[default]
    BestEffort,
    /// Block until the message can be sent.
    Reliable,
}

/// Where matching publications may originate.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SubscriptionOrigin {
    SessionLocal,
    Remote,
    #[default]
    Any,
}
