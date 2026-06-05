// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

//! Periodic timer registered on a node and driven by the executor.
//!
//! Run: `cargo run -p zos-runtime --example timer`

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use zos_runtime::{init, Executor, InitOptions, Node, NodeOptions, RuntimeError};

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    init(InitOptions::new()).await?;
    let mut node = Node::new(NodeOptions::new());

    let tick = Arc::new(AtomicU64::new(0));
    node.create_timer_builder(Duration::from_millis(500))
        .register({
            let tick = Arc::clone(&tick);
            move || {
                let n = tick.fetch_add(1, Ordering::Relaxed);
                async move {
                    println!("timer tick {n}");
                }
            }
        });

    println!("spinning (Ctrl+C to stop)...");
    Executor::spin_node(&mut node).await
}
