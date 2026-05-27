// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

//! ZeroOS runtime built on Zenoh.
//!
//! Planned: `Service`, `Parameter`, `Logger`, `Lifecycle`.

pub mod codec;
pub mod error;
pub mod executor;
pub mod node;
pub mod publisher;
pub mod qos;
pub mod runnable;
pub mod subscriber;
pub mod timer;

pub use codec::{decode, encode};
pub use error::RuntimeError;
pub use executor::Executor;
pub use node::{Node, PublisherBuilder, SubscriberBuilder, TimerBuilder};
pub use publisher::Publisher;
pub use qos::{MessagePriority, PublishQos, Qos, Reliability, SubscribeQos, SubscriptionOrigin};
pub use runnable::Runnable;
pub use subscriber::Subscriber;
pub use timer::Timer;
