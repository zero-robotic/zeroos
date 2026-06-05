// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

//! [`Executor`] 两种线程模型：默认 Tokio 池 vs 专用 worker 线程池。
//!
//! ```bash
//! # 当前 runtime 线程池（由 #[tokio::main] 决定）
//! cargo run -p zos-runtime --example executor
//!
//! # 专用 2 线程池
//! cargo run -p zos-runtime --example executor -- --dedicated
//! ```

use std::env;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use zos_runtime::{init, Executor, ExecutorOptions, InitOptions, Node, NodeOptions, RuntimeError};

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    init(InitOptions::new()).await?;
    let dedicated = env::args().any(|a| a == "--dedicated");

    let executor_opts = if dedicated {
        println!("executor: dedicated pool (worker_threads = 2)");
        ExecutorOptions::new().worker_threads(2)
    } else {
        println!("executor: default (use #[tokio::main] worker_threads)");
        ExecutorOptions::default()
    };

    let mut node = Node::new(NodeOptions::new());
    register_timer(&mut node);

    println!("spinning via Executor::spin_node_with (Ctrl+C to stop)...");
    Executor::spin_node_with(&mut node, executor_opts).await
}

fn register_timer(node: &mut Node) {
    let tick = Arc::new(AtomicU64::new(0));
    node.create_timer_builder(Duration::from_millis(500)).register({
        let tick = Arc::clone(&tick);
        move || {
            let n = tick.fetch_add(1, Ordering::Relaxed);
            async move {
                println!("timer tick {n}");
            }
        }
    });
}
