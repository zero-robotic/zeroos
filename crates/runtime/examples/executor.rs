// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

//! [`Executor`] 两种配置方式：默认 Tokio 池 vs 专用 worker 线程池。
//!
//! 通过 `NodeOptions`（常用）或直接 `Executor::new` 构造。
//!
//! ```bash
//! # 方式 A：NodeOptions + 当前 runtime 线程池（由 #[tokio::main] 决定）
//! cargo run -p zos-runtime --example executor
//!
//! # 方式 A + 专用 2 线程池
//! cargo run -p zos-runtime --example executor -- --dedicated
//!
//! # 方式 B：直接 Executor::new(opts)
//! cargo run -p zos-runtime --example executor -- --direct
//!
//! # 方式 B + 专用线程池
//! cargo run -p zos-runtime --example executor -- --direct --dedicated
//! ```

use std::env;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use zos_runtime::{Executor, ExecutorOptions, Node, NodeOptions, RuntimeError};

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    let direct = env::args().any(|a| a == "--direct");
    let dedicated = env::args().any(|a| a == "--dedicated");

    let executor_opts = if dedicated {
        println!("executor: dedicated pool (worker_threads = 2)");
        ExecutorOptions::new().worker_threads(2)
    } else {
        println!("executor: default (use #[tokio::main] worker_threads)");
        ExecutorOptions::default()
    };

    if direct {
        println!("construct: Executor::new(executor_opts)");
        run_direct(executor_opts).await
    } else {
        println!("construct: NodeOptions::executor(executor_opts)");
        run_via_node(executor_opts).await
    }
}

async fn run_via_node(executor_opts: ExecutorOptions) -> Result<(), RuntimeError> {
    let mut node = Node::new(NodeOptions::new().executor(executor_opts)).await?;
    register_timer(&mut node);
    println!("spinning via node.spin() (Ctrl+C to stop)...");
    node.spin().await
}

async fn run_direct(executor_opts: ExecutorOptions) -> Result<(), RuntimeError> {
    let mut node = Node::new(NodeOptions::new()).await?;
    register_timer(&mut node);

    let mut executor = Executor::new(executor_opts);
    executor.add_node(&mut node);
    println!("spinning via Executor::new(...).spin() (Ctrl+C to stop)...");
    executor.spin().await
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
