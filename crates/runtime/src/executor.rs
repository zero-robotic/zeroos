// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use crate::{Node, Runnable, RuntimeError};

/// Drives registered [`Runnable`] components (e.g. subscribers) concurrently on a Tokio runtime.
pub struct Executor {
    runnables: Vec<Box<dyn Runnable + Send>>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            runnables: Vec::new(),
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

    /// Spawn one task per runnable and wait until all complete or one returns an error.
    ///
    /// Requires a Tokio runtime (e.g. `#[tokio::main]`).
    pub async fn spin(self) -> Result<(), RuntimeError> {
        if self.runnables.is_empty() {
            return Ok(());
        }

        let mut handles = Vec::with_capacity(self.runnables.len());
        for mut runnable in self.runnables {
            handles.push(tokio::spawn(async move { runnable.run().await }));
        }

        for handle in handles {
            match handle.await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(RuntimeError::from(format!("executor task panicked: {e}"))),
            }
        }

        Ok(())
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}
