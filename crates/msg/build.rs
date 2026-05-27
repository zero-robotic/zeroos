// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

use std::io::Result;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=proto/");

    prost_build::compile_protos(
        &[
            "proto/geometry/twist.proto",
            "proto/geometry/vector2.proto",
        ],
        &["proto"],
    )?;

    Ok(())
}
