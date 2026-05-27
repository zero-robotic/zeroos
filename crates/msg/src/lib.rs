// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

//! Protobuf message types for ZeroOS (geometry, sensor, navigation, etc.).
//!
//! `.proto` files live under `crates/msg/proto/`. Run `cargo build -p zos-msg` to regenerate Rust types.

pub trait Message: prost::Message + Default + Send + Sync + 'static {}

impl<T> Message for T where T: prost::Message + Default + Send + Sync + 'static {}

/// Geometry messages (`zos.geometry` package).
pub mod geometry {
    include!(concat!(env!("OUT_DIR"), "/zos.geometry.rs"));
}

pub use geometry::{Twist, Vector2};
