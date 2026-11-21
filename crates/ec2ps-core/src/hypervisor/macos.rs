//! macOS Hypervisor.framework (HVF) prerequisite checking and driver configuration.

use super::guide::MACOS_GUIDE;
use super::qemu_common::QemuDriver;
use anyhow::{bail, Result};

/// Verifies that macOS HVF requirements are satisfied:
/// 1. CPU hardware virtualization (kern.hv_support) is available.
/// 2. QEMU binaries (`qemu-system-*` and `qemu-img`) are installed in PATH.
pub fn check_macos_prerequisites() -> Result<()> {
    // Check kern.hv_support via sysctl
    let output = std::process::Command::new("sysctl")
        .args(["-n", "kern.hv_support"])
        .output();

    if let Ok(out) = output {
        let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if val != "1" {
            bail!(
                "macOS Hypervisor.framework is not supported on this hardware (kern.hv_support={}).\n{}",
                val,
                MACOS_GUIDE
            );
        }
    }

    // Check QEMU binaries
    if QemuDriver::detect_optional().is_none() {
        bail!(
            "macOS QEMU engine was not found in PATH or Homebrew directories.\n{}",
            MACOS_GUIDE
        );
    }

    Ok(())
}
