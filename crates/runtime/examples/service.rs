// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

//! [`Service`] server: registers a Zenoh queryable and handles requests via [`Executor::spin`].
//!
//! Run: `cargo run -p zos-runtime --example service`
//!
//! In another terminal, call it with: `cargo run -p zos-runtime --example client -- --remote`

use zos_msg::{Twist, Vector2};
use zos_runtime::{init, Executor, InitOptions, Node, NodeOptions, RuntimeError};

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    init(InitOptions::new()).await?;
    let mut node = Node::new(NodeOptions::new().name("server").namespace("/demo"));

    node.create_service_builder::<Twist, Vector2>("scale")
        .register(|req| async move {
            println!(
                "service: request linear={:.2}, angular={:.2}",
                req.linear, req.angular
            );
            Ok(Vector2 {
                x: req.linear,
                y: req.angular,
            })
        })?;

    println!("service ready on /demo/scale");
    println!("call from another terminal: cargo run -p zos-runtime --example client -- --remote");
    println!("spinning (Ctrl+C to stop)...");
    Executor::spin_node(&mut node).await
}
