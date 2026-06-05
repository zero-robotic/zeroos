// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::OnceLock;

use crate::RuntimeError;

static SESSION: OnceLock<zenoh::Session> = OnceLock::new();

/// Options for [`init`]: Zenoh session is opened once per process.
#[derive(Debug, Clone)]
pub struct InitOptions {
    pub config: zenoh::Config,
}

impl Default for InitOptions {
    fn default() -> Self {
        Self {
            config: zenoh::Config::default(),
        }
    }
}

impl InitOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn config(mut self, config: zenoh::Config) -> Self {
        self.config = config;
        self
    }
}

/// Initialize the global Zenoh session. Call once before [`crate::Node::new`].
///
/// Use [`InitOptions::new`] (or [`Default::default`]) for the default Zenoh configuration.
pub async fn init(options: InitOptions) -> Result<(), RuntimeError> {
    if SESSION.get().is_some() {
        return Err(RuntimeError::from("runtime already initialized"));
    }

    let session = zenoh::open(options.config)
        .await
        .map_err(|e| RuntimeError::from(e.to_string()))?;

    SESSION
        .set(session)
        .map_err(|_| RuntimeError::from("runtime already initialized"))?;

    Ok(())
}

/// Whether [`init`] has completed successfully in this process.
pub fn is_initialized() -> bool {
    SESSION.get().is_some()
}

pub(crate) fn session() -> Result<zenoh::Session, RuntimeError> {
    SESSION
        .get()
        .cloned()
        .ok_or_else(|| RuntimeError::from("call init() before creating nodes"))
}
