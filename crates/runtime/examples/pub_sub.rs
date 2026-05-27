// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

//! Publish and subscribe to `Twist` on topic `cmd_vel`.
//!
//! Run: `cargo run -p zos-runtime --example pub_sub`

use std::time::Duration;

use zos_msg::Twist;
use zos_runtime::{Node, RuntimeError};

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    let mut node = Node::open_default().await?;

    node.create_subscriber_builder::<Twist>("cmd_vel")
        .register(|msg| async move {
            println!("recv cmd_vel: linear={:.2}, angular={:.2}", msg.linear, msg.angular);
        });

    let publisher = node.create_publisher::<Twist>("cmd_vel").build().await?;

    tokio::spawn(async move {
        let mut n = 0.0f64;
        loop {
            n += 0.1;
            if let Err(e) = publisher.publish(&Twist { linear: n, angular: 0.0 }).await {
                eprintln!("publish error: {e}");
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    println!("spinning (Ctrl+C to stop)...");
    node.spin().await
}
