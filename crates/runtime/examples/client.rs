// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

//! [`Client`]: request/response over Zenoh querier.
//!
//! Run (starts an in-process service for demo):
//!   `cargo run -p zos-runtime --example client`
//!
//! Run against an existing [`service`] example:
//!   `cargo run -p zos-runtime --example client -- --remote`

use std::env;

use zos_msg::{Twist, Vector2};
use zos_runtime::{init, Executor, Node, NodeOptions, RuntimeError};

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    init().await?;
    let remote = env::args().any(|a| a == "--remote");

    let server_task = if remote {
        None
    } else {
        let mut server = Node::new(NodeOptions::new().name("server").namespace("/demo"));
        server
            .create_service_builder::<Twist, Vector2>("scale")
            .register(|req| async move {
                Ok(Vector2 {
                    x: req.linear,
                    y: req.angular,
                })
            })
            .expect("register service");
        Some(tokio::spawn(async move {
            Executor::spin_node(&mut server)
                .await
                .expect("service spin failed");
        }))
    };

    if !remote {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    let client_node = Node::new(NodeOptions::new().name("caller"));
    // Absolute name (ROS 2): ignores caller's root namespace → `/demo/scale`.
    let client = client_node
        .create_client::<Twist, Vector2>("/demo/scale")
        .await?;

    for i in 0..5 {
        let resp = client
            .call(&Twist {
                linear: 3.0 + i as f64,
                angular: 0.5,
            })
            .await?;
        println!("client [{i}]: response x={:.2}, y={:.2}", resp.x, resp.y);
    }

    if let Some(task) = server_task {
        task.abort();
    }

    Ok(())
}
