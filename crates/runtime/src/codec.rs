// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use zos_msg::Message as ZosMessage;

use crate::RuntimeError;

pub fn decode<T: ZosMessage>(bytes: &[u8]) -> Result<T, RuntimeError> {
    T::decode(bytes).map_err(RuntimeError::from)
}

pub fn encode<T: ZosMessage>(message: &T) -> Result<Vec<u8>, RuntimeError> {
    Ok(message.encode_to_vec())
}
