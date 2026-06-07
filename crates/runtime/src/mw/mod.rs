// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

//! Middleware abstraction; concrete backends are private and selected in [`crate::context`].

pub mod error;
pub mod qos;
pub mod traits;

pub(crate) mod zenoh;

pub use error::MwError;
pub use qos::{
    MessagePriority, PublishQos, Qos, Reliability, SubscribeQos, SubscriptionOrigin,
};
pub use traits::{
    IncomingQuery, Publisher, Querier, QueryReply, Queryable, Session, Subscriber,
};
