// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use async_trait::async_trait;

use crate::node::Node;
use crate::{Runnable, RuntimeError};

pub type TimerCallbackFuture = Pin<Box<dyn Future<Output = ()> + Send>>;
pub type TimerCallback = dyn Fn() -> TimerCallbackFuture + Send + Sync;

/// Periodic or one-shot timer driven by Tokio.
pub struct Timer {
    period: Duration,
    one_shot: bool,
    callback: Box<TimerCallback>,
}

/// Builder for [`Timer`] created from a [`Node`].
pub struct TimerBuilder<'a> {
    node: &'a mut Node,
    period: Duration,
    one_shot: bool,
}

impl<'a> TimerBuilder<'a> {
    pub(crate) fn new(node: &'a mut Node, period: Duration) -> Self {
        Self {
            node,
            period,
            one_shot: false,
        }
    }
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

impl<'a> TimerBuilder<'a> {
    /// Fire once after `period`, then stop.
    pub fn one_shot(mut self) -> Self {
        self.one_shot = true;
        self
    }

    /// Build a timer without registering it on the node.
    pub fn callback<F, Fut>(self, callback: F) -> Timer
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        if self.one_shot {
            Timer::one_shot(self.period, callback)
        } else {
            Timer::periodic(self.period, callback)
        }
    }

    /// Build a timer and append it to [`Node::runnables`] for the executor.
    pub fn register<F, Fut>(self, callback: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let timer = if self.one_shot {
            Timer::one_shot(self.period, callback)
        } else {
            Timer::periodic(self.period, callback)
        };
        self.node.add_runnable(Box::new(timer));
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
