// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use async_trait::async_trait;

use crate::{Runnable, RuntimeError};

pub type TimerCallbackFuture = Pin<Box<dyn Future<Output = ()> + Send>>;
pub type TimerCallback = dyn Fn() -> TimerCallbackFuture + Send + Sync;

/// Periodic or one-shot timer driven by Tokio.
pub struct Timer {
    period: Duration,
    one_shot: bool,
    callback: Box<TimerCallback>,
}

impl Timer {
    pub fn periodic<F, Fut>(period: Duration, callback: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self::new(period, false, callback)
    }

    pub fn one_shot<F, Fut>(period: Duration, callback: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self::new(period, true, callback)
    }

    fn new<F, Fut>(period: Duration, one_shot: bool, callback: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self {
            period,
            one_shot,
            callback: Box::new(move || Box::pin(callback())),
        }
    }

    pub fn period(&self) -> Duration {
        self.period
    }

    pub fn is_one_shot(&self) -> bool {
        self.one_shot
    }
}

#[async_trait]
impl Runnable for Timer {
    async fn run(&mut self) -> Result<(), RuntimeError> {
        if self.one_shot {
            tokio::time::sleep(self.period).await;
            (self.callback)().await;
            return Ok(());
        }

        let mut interval = tokio::time::interval(self.period);
        loop {
            interval.tick().await;
            (self.callback)().await;
        }
    }
}
