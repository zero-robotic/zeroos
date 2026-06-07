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

/// Standard messages (`zos.std_msgs` package).
pub mod std_msgs {
    include!(concat!(env!("OUT_DIR"), "/zos.std_msgs.rs"));
}

/// Sensor messages (`zos.sensor` package).
pub mod sensor {
    include!(concat!(env!("OUT_DIR"), "/zos.sensor.rs"));
}

/// Navigation messages (`zos.nav` package).
pub mod nav {
    include!(concat!(env!("OUT_DIR"), "/zos.nav.rs"));
}

pub use geometry::{Point, Point32, Pose, Pose2D, PoseStamped, PoseWithCovariance, Quaternion, Twist, TwistStamped, TwistWithCovariance, Vector2, Vector3};
pub use nav::Odometry;
pub use sensor::{CompressedImage, Image, Imu, PointCloud2, PointField};
pub use std_msgs::{Header, Time};
