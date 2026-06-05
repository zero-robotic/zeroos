// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use crate::{Node, Runnable, RuntimeError};

/// Executor configuration (passed to [`Executor::new`]).
#[derive(Debug, Clone, Default)]
pub struct ExecutorOptions {
    /// `None`: run on the current Tokio runtime ([`Executor::spin`].await inside `#[tokio::main]`).
    /// `Some(n)`: create a dedicated runtime with `n` worker threads.
    pub worker_threads: Option<usize>,
}

impl ExecutorOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn worker_threads(mut self, n: usize) -> Self {
        self.worker_threads = Some(n);
        self
    }
}

/// Drives registered [`Runnable`] components concurrently on a Tokio runtime.
pub struct Executor {
    runnables: Vec<Box<dyn Runnable + Send>>,
    options: ExecutorOptions,
}

impl Executor {
    pub fn new(options: ExecutorOptions) -> Self {
        Self {
            runnables: Vec::new(),
            options,
        }
    }

    pub fn add(&mut self, runnable: Box<dyn Runnable + Send>) {
        self.runnables.push(runnable);
    }

    pub fn is_empty(&self) -> bool {
        self.runnables.is_empty()
    }

    pub fn len(&self) -> usize {
        self.runnables.len()
    }

    /// Take all runnables registered on `node` and append them to this executor.
    pub fn add_node(&mut self, node: &mut Node) {
        self.runnables.append(&mut std::mem::take(&mut node.runnables));
    }

    /// Run all runnables using [`ExecutorOptions`] from construction.
    ///
    /// - `worker_threads: None` — spawn on the current runtime (`#[tokio::main(worker_threads = N)]`).
    /// - `worker_threads: Some(n)` — dedicated `n`-thread pool (blocking thread hosts the runtime).
    pub async fn spin(self) -> Result<(), RuntimeError> {
        match self.options.worker_threads {
            None => {
                Self::spawn_and_wait(tokio::runtime::Handle::current(), self.runnables).await
            }
            Some(workers) => {
                let runnables = self.runnables;
                tokio::task::spawn_blocking(move || Self::run_on_dedicated(runnables, workers))
                    .await
                    .map_err(|e| RuntimeError::from(format!("executor task join failed: {e}")))?
            }
        }
    }

    fn run_on_dedicated(
        runnables: Vec<Box<dyn Runnable + Send>>,
        workers: usize,
    ) -> Result<(), RuntimeError> {
        let workers = workers.max(1);
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(workers)
            .enable_all()
            .build()
            .map_err(|e| RuntimeError::from(e.to_string()))?;

        runtime.block_on(Self::spawn_and_wait(runtime.handle().clone(), runnables))
    }

    async fn spawn_and_wait(
        handle: tokio::runtime::Handle,
        runnables: Vec<Box<dyn Runnable + Send>>,
    ) -> Result<(), RuntimeError> {
        if runnables.is_empty() {
            return Ok(());
        }

        let mut handles = Vec::with_capacity(runnables.len());
        for mut runnable in runnables {
            handles.push(handle.spawn(async move { runnable.run().await }));
        }

        for join in handles {
            match join.await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => return Err(e),
                Err(e) => {
                    return Err(RuntimeError::from(format!("executor task panicked: {e}")));
                }
            }
        }

        Ok(())
    }
}

/// Default worker count when using a dedicated pool with `worker_threads(0)` is not used;
/// prefer explicit `ExecutorOptions::worker_threads(n)`.
pub fn default_worker_threads() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

impl Default for Executor {
    fn default() -> Self {
        Self::new(ExecutorOptions::default())
    }
}
