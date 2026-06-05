// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

//! ZeroOS facade crate.
//!
//! Re-exports the internal crates so applications can depend on a single package.

pub use zos_map as map;
pub use zos_msg as msg;
pub use zos_runtime as runtime;
