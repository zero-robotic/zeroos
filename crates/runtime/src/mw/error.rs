// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MwError {
    #[error("{0}")]
    Message(String),
}

impl From<String> for MwError {
    fn from(value: String) -> Self {
        Self::Message(value)
    }
}

impl From<&str> for MwError {
    fn from(value: &str) -> Self {
        Self::Message(value.to_owned())
    }
}
