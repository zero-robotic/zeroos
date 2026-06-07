// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

//! Zenoh middleware backend (private; wired from [`crate::context`] only).

mod open;
mod publisher;
mod querier;
mod queryable;
mod qos;
mod session;
mod subscriber;

pub(crate) use open::{open_default, open_from_file};
