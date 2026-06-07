// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use crate::mw::MwError;

use super::session::ZenohSession;

/// Open a Zenoh session with default configuration.
pub(crate) async fn open_default() -> Result<Arc<ZenohSession>, MwError> {
    open_config(zenoh::Config::default()).await
}

/// Open a Zenoh session from a JSON5 config file.
pub(crate) async fn open_from_file(path: impl AsRef<std::path::Path>) -> Result<Arc<ZenohSession>, MwError> {
    let config = zenoh::Config::from_file(path).map_err(|e| MwError::from(e.to_string()))?;
    open_config(config).await
}

async fn open_config(config: zenoh::Config) -> Result<Arc<ZenohSession>, MwError> {
    let session = zenoh::open(config)
        .await
        .map_err(|e| MwError::from(e.to_string()))?;
    Ok(Arc::new(ZenohSession::new(session)))
}
