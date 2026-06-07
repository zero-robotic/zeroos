// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

//! Process-wide middleware initialization and session access.
//!
//! This is the only runtime module that selects a concrete backend implementation.

use std::path::Path;
use std::sync::{Arc, OnceLock};

use crate::mw::Session;
use crate::RuntimeError;

/// Active middleware implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiddlewareBackend {
    /// [Zenoh](https://zenoh.io/) (default).
    Zenoh,
}

impl Default for MiddlewareBackend {
    fn default() -> Self {
        Self::Zenoh
    }
}

/// Options for [`init_with`].
#[derive(Debug, Clone, Default)]
pub struct InitOptions {
    /// Backend-specific config file. For [`MiddlewareBackend::Zenoh`], a Zenoh JSON5 file.
    pub config_file: Option<std::path::PathBuf>,
}

static SESSION: OnceLock<Arc<dyn Session>> = OnceLock::new();
static BACKEND: OnceLock<MiddlewareBackend> = OnceLock::new();

/// Initialize the global middleware session with the default backend ([`MiddlewareBackend::Zenoh`]).
pub async fn init() -> Result<(), RuntimeError> {
    init_with(MiddlewareBackend::default()).await
}

/// Initialize with an explicit backend and default options.
pub async fn init_with(backend: MiddlewareBackend) -> Result<(), RuntimeError> {
    init_with_options(backend, InitOptions::default()).await
}

/// Initialize with the default backend and a Zenoh JSON5 config file.
pub async fn init_from_file(path: impl AsRef<Path>) -> Result<(), RuntimeError> {
    init_with_options(
        MiddlewareBackend::Zenoh,
        InitOptions {
            config_file: Some(path.as_ref().to_path_buf()),
        },
    )
    .await
}

/// Initialize with explicit backend and options.
pub async fn init_with_options(
    backend: MiddlewareBackend,
    options: InitOptions,
) -> Result<(), RuntimeError> {
    if SESSION.get().is_some() {
        return Err(RuntimeError::from("runtime already initialized"));
    }

    let session = open_backend(backend, &options).await?;
    SESSION
        .set(session)
        .map_err(|_| RuntimeError::from("runtime already initialized"))?;
    let _ = BACKEND.set(backend);
    Ok(())
}

/// Whether [`init`] has completed successfully in this process.
pub fn is_initialized() -> bool {
    SESSION.get().is_some()
}

/// Active backend after [`init`].
pub fn backend() -> Result<MiddlewareBackend, RuntimeError> {
    BACKEND
        .get()
        .copied()
        .ok_or_else(|| RuntimeError::from("call init() before querying backend"))
}

/// Global middleware session opened by [`init`].
pub fn session() -> Result<Arc<dyn Session>, RuntimeError> {
    SESSION
        .get()
        .cloned()
        .ok_or_else(|| RuntimeError::from("call init() before creating endpoints"))
}

async fn open_backend(
    backend: MiddlewareBackend,
    options: &InitOptions,
) -> Result<Arc<dyn Session>, RuntimeError> {
    match backend {
        MiddlewareBackend::Zenoh => {
            let zenoh = if let Some(path) = &options.config_file {
                crate::mw::zenoh::open_from_file(path).await?
            } else {
                crate::mw::zenoh::open_default().await?
            };
            Ok(zenoh as Arc<dyn Session>)
        }
    }
}
