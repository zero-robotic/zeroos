// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::OnceLock;

use crate::RuntimeError;

static SESSION: OnceLock<zenoh::Session> = OnceLock::new();

/// Options for [`init`]: middleware session is opened once per process.
#[derive(Debug, Clone)]
pub struct InitOptions {
    config: zenoh::Config,
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

    /// Load configuration from a JSON5 file (e.g. Zenoh `ZENOH_CONFIG` layout).
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, RuntimeError> {
        zenoh::Config::from_file(path)
            .map(|config| Self { config })
            .map_err(|e| RuntimeError::from(e.to_string()))
    }

    /// Load configuration from a JSON5 string.
    pub fn from_json5(input: &str) -> Result<Self, RuntimeError> {
        zenoh::Config::from_json5(input)
            .map(|config| Self { config })
            .map_err(|e| RuntimeError::from(e.to_string()))
    }

    /// Load configuration from the path in the `ZENOH_CONFIG` environment variable.
    pub fn from_env() -> Result<Self, RuntimeError> {
        zenoh::Config::from_env()
            .map(|config| Self { config })
            .map_err(|e| RuntimeError::from(e.to_string()))
    }

    pub fn config(mut self, config: zenoh::Config) -> Self {
        self.config = config;
        self
    }
}

/// Initialize the global middleware session. Call once before [`crate::Node::new`].
///
/// Use [`InitOptions::new`] (or [`Default::default`]) for default configuration,
/// or [`InitOptions::from_file`] / [`InitOptions::from_json5`] for custom settings.
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

/// Global session opened by [`init`]. Clone per endpoint as needed.
pub fn session() -> Result<zenoh::Session, RuntimeError> {
    SESSION
        .get()
        .cloned()
        .ok_or_else(|| RuntimeError::from("call init() before creating endpoints"))
}
