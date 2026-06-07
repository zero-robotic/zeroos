// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

//! ZeroOS runtime: ROS-like nodes on a pluggable middleware stack.
//!
//! Call [`init`](context::init) or [`init_with`](context::init_with) once, then use [`Node`](node::Node).
//! Transport is abstracted in [`mw`](crate::mw); the active backend is chosen in [`context`](crate::context).
//!
//! Planned: `Parameter`, `Logger`, `Lifecycle`.

pub mod codec;
pub mod context;
pub mod error;
pub mod executor;
pub mod mw;
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
pub use executor::{default_worker_threads, Executor, ExecutorOptions};
pub use client::Client;
pub use context::{
    backend, init, init_from_file, init_with, init_with_options, is_initialized, session,
    InitOptions, MiddlewareBackend,
};
pub use node::{normalize_namespace, resolve_name, Node, NodeOptions};
pub use publisher::{Publisher, PublisherBuilder};
pub use service::{Service, ServiceBuilder};
pub use subscriber::{Subscriber, SubscriberBuilder};
pub use timer::{Timer, TimerBuilder};
pub use qos::{MessagePriority, PublishQos, Qos, Reliability, SubscribeQos, SubscriptionOrigin};
pub use runnable::Runnable;

pub use mw::{MwError, Session as MiddlewareSession};
