// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared state for the ZeroOS message visualizer.
//!
//! Sensor streams (`scan`, camera) will extend [`VizState`] once `zos-msg` types exist.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Latest `cmd_vel` sample and receive metadata.
#[derive(Debug, Clone)]
pub struct CmdVelSample {
    pub linear: f64,
    pub angular: f64,
    received_at: Option<Instant>,
}

impl Default for CmdVelSample {
    fn default() -> Self {
        Self {
            linear: 0.0,
            angular: 0.0,
            received_at: None,
        }
    }
}

impl CmdVelSample {
    pub fn age(&self) -> Option<Duration> {
        self.received_at.map(|t| t.elapsed())
    }

    pub fn has_data(&self) -> bool {
        self.received_at.is_some()
    }
}

/// Thread-safe snapshot of subscribed topics for the UI.
#[derive(Debug, Default, Clone)]
pub struct VizState {
    pub cmd_vel: CmdVelSample,
    pub cmd_vel_count: u64,
}

impl VizState {
    pub fn update_cmd_vel(&mut self, linear: f64, angular: f64) {
        self.cmd_vel = CmdVelSample {
            linear,
            angular,
            received_at: Some(Instant::now()),
        };
        self.cmd_vel_count += 1;
    }
}

pub type SharedVizState = Arc<Mutex<VizState>>;

pub fn new_shared_state() -> SharedVizState {
    Arc::new(Mutex::new(VizState::default()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_cmd_vel_increments_count() {
        let mut state = VizState::default();
        state.update_cmd_vel(0.8, 0.2);
        assert_eq!(state.cmd_vel_count, 1);
        assert!((state.cmd_vel.linear - 0.8).abs() < f64::EPSILON);
        assert!((state.cmd_vel.angular - 0.2).abs() < f64::EPSILON);
        assert!(state.cmd_vel.has_data());
    }
}
