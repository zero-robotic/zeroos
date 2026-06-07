// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

//! Embed MuJoCo framework rpath into the simulation binary on macOS.
//!
//! pkg-config passes rpath when linking `mujoco-rs`, but that does not propagate
//! to the final `zos-simulation` executable.

fn main() {
    #[cfg(target_os = "macos")]
    embed_mujoco_rpath();
}

#[cfg(target_os = "macos")]
fn embed_mujoco_rpath() {
    if let Some(frameworks) = frameworks_from_pkg_config()
        .or_else(frameworks_from_brew)
    {
        let rpath = frameworks.display();
        println!("cargo:rustc-link-arg=-Wl,-rpath,{rpath}");
        println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
        return;
    }

    println!(
        "cargo:warning=MuJoCo framework rpath not embedded. If runtime fails with \
         'Library not loaded: @rpath/mujoco.framework', install via \
         'brew install kcking/tap/mujoco@3.7', set \
         PKG_CONFIG_PATH=\"$(brew --prefix mujoco@3.7)/lib/pkgconfig\", and rebuild."
    );
}

#[cfg(target_os = "macos")]
fn frameworks_from_pkg_config() -> Option<std::path::PathBuf> {
    let pkg_config_path = std::env::var_os("PKG_CONFIG_PATH")?;
    for entry in std::env::split_paths(&pkg_config_path) {
        if let Some(frameworks) = entry
            .parent()
            .and_then(|lib| lib.parent())
            .map(|prefix| prefix.join("Frameworks"))
            .filter(|path| path.join("mujoco.framework").is_dir())
        {
            return Some(frameworks);
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn frameworks_from_brew() -> Option<std::path::PathBuf> {
    let output = std::process::Command::new("brew")
        .args(["--prefix", "mujoco@3.7"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let prefix = String::from_utf8(output.stdout).ok()?.trim().to_string();
    let frameworks = std::path::PathBuf::from(prefix).join("Frameworks");
    frameworks.join("mujoco.framework").is_dir().then_some(frameworks)
}
