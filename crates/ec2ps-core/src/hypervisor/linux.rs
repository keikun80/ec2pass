//! Linux KVM hypervisor prerequisite checking and driver configuration.

use super::guide::LINUX_GUIDE;
use super::qemu_common::QemuDriver;
use anyhow::{bail, Result};
use std::path::Path;

/// Verifies that Linux KVM requirements are satisfied:
/// 1. `/dev/kvm` exists and is readable/writable.
/// 2. QEMU binaries (`qemu-system-*` and `qemu-img`) exist.
pub fn check_linux_prerequisites() -> Result<()> {
    let allow_unaccelerated = std::env::var("EC2PASS_ALLOW_UNACCELERATED").is_ok()
        || std::env::var("EC2PASS_SKIP_HYPERVISOR_CHECK").is_ok();

    let kvm_dev = Path::new("/dev/kvm");
    if !kvm_dev.exists() && !allow_unaccelerated {
        bail!(
            "Linux KVM device (/dev/kvm) was not found.\n{}",
            LINUX_GUIDE
        );
    }

    // Verify QEMU binaries
    if QemuDriver::detect_optional().is_none() && !allow_unaccelerated {
        bail!(
            "QEMU virtualization binary (qemu-system-x86_64 or qemu-img) was not found.\n{}",
            LINUX_GUIDE
        );
    }

    Ok(())
}
