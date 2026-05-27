// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

//! ZeroOS runtime built on Zenoh.
//!
//! Planned: `Parameter`, `Logger`, `Lifecycle`.

pub mod codec;
pub mod error;
pub mod executor;
pub mod node;
pub mod publisher;
pub mod qos;
pub mod runnable;
pub mod client;
pub mod service;
pub mod subscriber;
pub mod timer;

pub use codec::{decode, encode};
pub use error::RuntimeError;
pub use executor::Executor;
pub use client::Client;
pub use node::{normalize_namespace, resolve_name, Node, NodeOptions};
pub use publisher::{Publisher, PublisherBuilder};
pub use service::{Service, ServiceBuilder};
pub use subscriber::{Subscriber, SubscriberBuilder};
pub use timer::{Timer, TimerBuilder};
pub use qos::{MessagePriority, PublishQos, Qos, Reliability, SubscribeQos, SubscriptionOrigin};
pub use runnable::Runnable;
