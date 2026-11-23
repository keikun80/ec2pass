//! Platform-specific hypervisor abstraction, prerequisite checks, and guides.

pub mod guide;
pub mod linux;
pub mod macos;
pub mod qemu_common;
pub mod windows;

pub use guide::get_hypervisor_install_guide;
pub use qemu_common::QemuDriver;
pub use windows::HypervDriver;

use crate::state::InstanceRecord;
use anyhow::{bail, Result};
use async_trait::async_trait;
use std::path::Path;

#[async_trait]
pub trait VirtualMachineDriver: Send + Sync {
    /// Name of the active driver (`"qemu"` or `"hyperv"`).
    fn driver_name(&self) -> &str;

    /// Creates a Copy-on-Write (`qcow2` or `vhd`) instance overlay disk backed by `base_image`.
    async fn create_overlay_disk(
        &self,
        base_image: &Path,
        overlay_path: &Path,
        disk_bytes: u64,
    ) -> Result<()>;

    /// Resizes the root/overlay disk to the new byte size.
    async fn resize_disk(&self, disk_path: &Path, new_bytes: u64) -> Result<()>;

    /// Boots the virtual machine process and returns its host PID.
    async fn start_vm(&self, rec: &InstanceRecord) -> Result<u32>;

    /// Gracefully powers down or forces off the virtual machine.
    async fn stop_vm(&self, rec: &InstanceRecord, force: bool) -> Result<()>;

    /// Suspends the VM execution state.
    async fn suspend_vm(&self, rec: &InstanceRecord) -> Result<()>;

    /// Resumes a suspended VM.
    async fn resume_vm(&self, rec: &InstanceRecord) -> Result<()>;

    /// Creates an internal disk snapshot.
    async fn take_disk_snapshot(&self, disk_path: &Path, snapshot_name: &str) -> Result<()>;

    /// Restores an internal disk snapshot.
    async fn restore_disk_snapshot(&self, disk_path: &Path, snapshot_name: &str) -> Result<()>;

    /// Creates a live VM snapshot (memory + disk).
    async fn take_live_snapshot(
        &self,
        _qmp_port: u16,
        disk_path: &Path,
        snapshot_name: &str,
    ) -> Result<()> {
        self.take_disk_snapshot(disk_path, snapshot_name).await
    }

    /// Restores a live VM snapshot (memory + disk).
    async fn restore_live_snapshot(
        &self,
        _qmp_port: u16,
        disk_path: &Path,
        snapshot_name: &str,
    ) -> Result<()> {
        self.restore_disk_snapshot(disk_path, snapshot_name).await
    }
}

/// Strictly checks that the platform's required hypervisor is installed and operational.
/// Fails with an actionable, user-friendly installation guide if missing.
pub fn check_hypervisor_prerequisites() -> Result<()> {
    if cfg!(target_os = "linux") {
        linux::check_linux_prerequisites()
    } else if cfg!(target_os = "macos") {
        macos::check_macos_prerequisites()
    } else if cfg!(target_os = "windows") {
        windows::check_windows_prerequisites()
    } else {
        bail!("{}", guide::GENERIC_GUIDE);
    }
}

/// Creates the appropriate driver for the current host OS.
/// Fails immediately if the hypervisor is absent.
pub fn create_default_driver(preferred: Option<&str>) -> Result<Box<dyn VirtualMachineDriver>> {
    // 1. Perform prerequisite check
    check_hypervisor_prerequisites()?;

    // 2. Select driver: QEMU is the primary default engine across all platforms (Windows, Linux, macOS).
    // Hyper-V is an optional secondary fallback only if explicitly requested.
    if let Some("hyperv") = preferred {
        let hv = HypervDriver::detect();
        if hv.available {
            return Ok(Box::new(hv));
        }
    }

    Ok(Box::new(QemuDriver::try_new()?))
}
