// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;

use crate::RuntimeError;

#[async_trait]
pub trait Runnable: Send + Sync {
    // Core run entry for long-running components.
    async fn run(&mut self) -> Result<(), RuntimeError>;
}
