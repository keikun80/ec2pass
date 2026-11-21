//! Windows Hyper-V and WHPX hypervisor prerequisite checking and driver implementation.

use super::guide::WINDOWS_GUIDE;
use super::qemu_common::QemuDriver;
use super::VirtualMachineDriver;
use crate::state::InstanceRecord;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use std::path::Path;
use tokio::process::Command;

/// Verifies that Windows virtualization requirements are satisfied:
/// 1. Hyper-V PowerShell module responds, OR
/// 2. QEMU with WHPX support is installed.
pub fn check_windows_prerequisites() -> Result<()> {
    // Check Hyper-V availability
    if HypervDriver::detect().available {
        return Ok(());
    }

    // Fallback: check QEMU on Windows
    if QemuDriver::detect_optional().is_some() {
        return Ok(());
    }

    bail!(
        "Neither Hyper-V nor QEMU was found on this Windows system.\n{}",
        WINDOWS_GUIDE
    );
}

/// Windows Hyper-V native driver.
pub struct HypervDriver {
    pub available: bool,
}

impl HypervDriver {
    pub fn detect() -> Self {
        if !cfg!(windows) {
            return Self { available: false };
        }
        let out = std::process::Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Get-Command Get-VM -ErrorAction SilentlyContinue",
            ])
            .output();

        let available = match out {
            Ok(o) => o.status.success() && !o.stdout.is_empty(),
            Err(_) => false,
        };
        Self { available }
    }

    async fn run_ps(cmd: &str) -> Result<String> {
        let out = Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", cmd])
            .output()
            .await
            .with_context(|| format!("Failed to run powershell command: {}", cmd))?;

        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            bail!("PowerShell command failed: {}\nStderr: {}", cmd, err);
        }
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }
}

#[async_trait]
impl VirtualMachineDriver for HypervDriver {
    fn driver_name(&self) -> &str {
        "hyperv"
    }

    fn is_mock(&self) -> bool {
        false
    }

    async fn create_overlay_disk(
        &self,
        base_image: &Path,
        overlay_path: &Path,
        disk_bytes: u64,
    ) -> Result<()> {
        if let Some(parent) = overlay_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let size_bytes = disk_bytes.max(2 * 1024 * 1024 * 1024);
        let cmd = format!(
            "New-VHD -Path '{}' -ParentPath '{}' -SizeBytes {} -Differencing -ErrorAction Stop",
            overlay_path.to_string_lossy(),
            base_image.to_string_lossy(),
            size_bytes
        );
        Self::run_ps(&cmd).await?;
        Ok(())
    }

    async fn start_vm(&self, rec: &InstanceRecord) -> Result<u32> {
        let name = &rec.details.name;
        let cpus = rec.details.cpus;
        let mem = rec.details.memory_total_bytes;
        let disk = rec.disk_path.to_string_lossy();

        let cmd = format!(
            r#"
            $vm = Get-VM -Name '{name}' -ErrorAction SilentlyContinue
            if (-not $vm) {{
                New-VM -Name '{name}' -MemoryStartupBytes {mem} -Generation 2 -NoVHD | Out-Null
                Set-VMProcessor -VMName '{name}' -Count {cpus} | Out-Null
                Add-VMHardDiskDrive -VMName '{name}' -Path '{disk}' | Out-Null
                Set-VMFirmware -VMName '{name}' -EnableSecureBoot Off | Out-Null
            }}
            Start-VM -Name '{name}' | Out-Null
            (Get-Process -Name vmwp -ErrorAction SilentlyContinue | Select-Object -First 1).Id
            "#
        );
        let out = Self::run_ps(&cmd).await?;
        let pid = out
            .lines()
            .last()
            .and_then(|l| l.trim().parse::<u32>().ok())
            .unwrap_or(1000);
        Ok(pid)
    }

    async fn stop_vm(&self, rec: &InstanceRecord, force: bool) -> Result<()> {
        let name = &rec.details.name;
        let flag = if force { "-TurnOff" } else { "-Force" };
        let cmd = format!(
            "Stop-VM -Name '{}' {} -ErrorAction SilentlyContinue",
            name, flag
        );
        let _ = Self::run_ps(&cmd).await;
        Ok(())
    }

    async fn suspend_vm(&self, rec: &InstanceRecord) -> Result<()> {
        let name = &rec.details.name;
        let cmd = format!("Suspend-VM -Name '{}' -ErrorAction SilentlyContinue", name);
        let _ = Self::run_ps(&cmd).await;
        Ok(())
    }

    async fn resume_vm(&self, rec: &InstanceRecord) -> Result<()> {
        let name = &rec.details.name;
        let cmd = format!("Resume-VM -Name '{}' -ErrorAction SilentlyContinue", name);
        let _ = Self::run_ps(&cmd).await;
        Ok(())
    }

    async fn take_disk_snapshot(&self, _disk_path: &Path, snapshot_name: &str) -> Result<()> {
        let cmd = format!(
            "Checkpoint-VM -SnapshotName '{}' -ErrorAction SilentlyContinue",
            snapshot_name
        );
        let _ = Self::run_ps(&cmd).await;
        Ok(())
    }

    async fn restore_disk_snapshot(&self, _disk_path: &Path, snapshot_name: &str) -> Result<()> {
        let cmd = format!(
            "Restore-VMCheckpoint -Name '{}' -Confirm:$false -ErrorAction SilentlyContinue",
            snapshot_name
        );
        let _ = Self::run_ps(&cmd).await;
        Ok(())
    }
}
