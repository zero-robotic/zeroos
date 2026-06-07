// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use std::io::Result;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=proto/");

    prost_build::compile_protos(
        &[
            "proto/geometry_msgs/point.proto",
            "proto/geometry_msgs/point32.proto",
            "proto/geometry_msgs/quaternion.proto",
            "proto/geometry_msgs/pose.proto",
            "proto/geometry_msgs/pose2d.proto",
            "proto/geometry_msgs/pose_stamped.proto",
            "proto/geometry_msgs/pose_with_covariance.proto",
            "proto/geometry_msgs/twist.proto",
            "proto/geometry_msgs/twist_stamped.proto",
            "proto/geometry_msgs/twist_with_covariance.proto",
            "proto/geometry_msgs/vector2.proto",
            "proto/geometry_msgs/vector3.proto",
            "proto/std_msgs/time.proto",
            "proto/std_msgs/header.proto",
            "proto/sensor_msgs/imu.proto",
            "proto/sensor_msgs/image.proto",
            "proto/sensor_msgs/compressed_image.proto",
            "proto/sensor_msgs/point_field.proto",
            "proto/sensor_msgs/pointcloud2.proto",
            "proto/nav_msgs/odometry.proto",
        ],
        &["proto"],
    )?;

    Ok(())
}
