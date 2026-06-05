// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("{0}")]
    Message(String),
    #[error("protobuf decode error: {0}")]
    Decode(#[from] prost::DecodeError),
    #[error("protobuf encode error: {0}")]
    Encode(#[from] prost::EncodeError),
}

impl From<String> for RuntimeError {
    fn from(value: String) -> Self {
        Self::Message(value)
    }
}

impl From<&str> for RuntimeError {
    fn from(value: &str) -> Self {
        Self::Message(value.to_owned())
    }
}
