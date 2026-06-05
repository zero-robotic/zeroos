// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::OnceLock;

use crate::RuntimeError;

static SESSION: OnceLock<zenoh::Session> = OnceLock::new();

/// Initialize the global middleware session with default configuration.
pub async fn init() -> Result<(), RuntimeError> {
    open_session(zenoh::Config::default()).await
}

/// Initialize from a JSON5 config file (Zenoh `ZENOH_CONFIG` layout).
///
/// For the `ZENOH_CONFIG` environment variable:  
/// `init_from_file(std::env::var("ZENOH_CONFIG")?)`.
pub async fn init_from_file(path: impl AsRef<std::path::Path>) -> Result<(), RuntimeError> {
    let config = zenoh::Config::from_file(path).map_err(|e| RuntimeError::from(e.to_string()))?;
    open_session(config).await
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

async fn open_session(config: zenoh::Config) -> Result<(), RuntimeError> {
    if SESSION.get().is_some() {
        return Err(RuntimeError::from("runtime already initialized"));
    }

    let session = zenoh::open(config)
        .await
        .map_err(|e| RuntimeError::from(e.to_string()))?;

    SESSION
        .set(session)
        .map_err(|_| RuntimeError::from("runtime already initialized"))?;

    Ok(())
}
