//! Linux KVM hypervisor prerequisite checking and driver configuration.

use super::guide::LINUX_GUIDE;
use super::qemu_common::QemuDriver;
use anyhow::{bail, Result};
use std::path::Path;

/// Verifies that Linux KVM requirements are satisfied:
/// 1. `/dev/kvm` exists and is readable/writable.
/// 2. QEMU binaries (`qemu-system-*` and `qemu-img`) exist.
pub fn check_linux_prerequisites() -> Result<()> {
    let kvm_dev = Path::new("/dev/kvm");
    if !kvm_dev.exists() {
        bail!(
            "Linux KVM device (/dev/kvm) was not found.\n{}",
            LINUX_GUIDE
        );
    }

    // Verify QEMU binaries
    if QemuDriver::detect_optional().is_none() {
        bail!(
            "QEMU virtualization binary (qemu-system-x86_64 or qemu-img) was not found.\n{}",
            LINUX_GUIDE
        );
    }

    Ok(())
}
