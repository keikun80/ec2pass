//! Common QEMU hypervisor implementation and process / QMP socket manager.

use super::guide::get_hypervisor_install_guide;
use crate::state::InstanceRecord;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::process::Command;

/// Platform-agnostic QEMU Hypervisor Driver.
pub struct QemuDriver {
    pub qemu_bin: String,
    pub qemu_img_bin: String,
}

impl QemuDriver {
    /// Detects QEMU binaries in PATH or known installation locations.
    /// Returns None if QEMU binaries cannot be found.
    pub fn detect_optional() -> Option<Self> {
        let base_bin = if cfg!(target_arch = "aarch64") {
            "qemu-system-aarch64"
        } else {
            "qemu-system-x86_64"
        };
        let base_img = "qemu-img";

        let candidate_dirs = [
            PathBuf::from(""),
            PathBuf::from(r"C:\Program Files\qemu"),
            PathBuf::from(r"C:\Program Files (x86)\qemu"),
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/opt/homebrew/bin"),
            PathBuf::from("/usr/bin"),
        ];

        for dir in &candidate_dirs {
            let bin_path = if dir.as_os_str().is_empty() {
                PathBuf::from(base_bin)
            } else if cfg!(windows) {
                dir.join(format!("{}.exe", base_bin))
            } else {
                dir.join(base_bin)
            };

            let img_path = if dir.as_os_str().is_empty() {
                PathBuf::from(base_img)
            } else if cfg!(windows) {
                dir.join(format!("{}.exe", base_img))
            } else {
                dir.join(base_img)
            };

            if let Ok(s) = std::process::Command::new(&bin_path)
                .arg("--version")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
            {
                if s.success() {
                    return Some(Self {
                        qemu_bin: bin_path.to_string_lossy().to_string(),
                        qemu_img_bin: img_path.to_string_lossy().to_string(),
                    });
                }
            }
        }
        None
    }

    /// Strict constructor that fails with the OS-specific guide if QEMU is absent.
    pub fn try_new() -> Result<Self> {
        match Self::detect_optional() {
            Some(driver) => Ok(driver),
            None => {
                if std::env::var("EC2PASS_ALLOW_UNACCELERATED").is_ok()
                    || std::env::var("EC2PASS_SKIP_HYPERVISOR_CHECK").is_ok()
                {
                    Ok(Self {
                        qemu_bin: "qemu-system-x86_64".to_string(),
                        qemu_img_bin: "qemu-img".to_string(),
                    })
                } else {
                    bail!("{}", get_hypervisor_install_guide())
                }
            }
        }
    }

    /// Sends a raw QMP command payload, negotiates greeting/caps, and waits for `return` or `error` reply.
    pub async fn send_qmp_execute(&self, qmp_port: u16, payload: &str) -> Result<String> {
        let addr = format!("127.0.0.1:{}", qmp_port);
        let mut stream = None;
        for i in 0..10 {
            match TcpStream::connect(&addr).await {
                Ok(s) => {
                    stream = Some(s);
                    break;
                }
                Err(_) if i < 9 => {
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                }
                Err(e) => {
                    return Err(e).with_context(|| format!("Failed to connect to QMP socket at {}", addr));
                }
            }
        }
        let stream = stream.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        // 1. Read QMP greeting banner
        let _greeting = lines.next_line().await?;

        // 2. Negotiate qmp_capabilities
        writer
            .write_all(b"{\"execute\":\"qmp_capabilities\"}\r\n")
            .await?;
        let _cap_reply = lines.next_line().await?;

        // 3. Send actual command
        writer.write_all(payload.as_bytes()).await?;
        writer.write_all(b"\r\n").await?;

        // 4. Drain until a return or error response (ignoring asynchronous STOP/RESUME events)
        while let Some(line) = lines.next_line().await? {
            if line.contains("\"return\"") {
                return Ok(line);
            }
            if line.contains("\"error\"") {
                bail!("QMP command returned error: {}", line);
            }
        }
        bail!("QMP connection closed unexpectedly");
    }

    /// Sends a Human Monitor command through QMP `human-monitor-command`.
    pub async fn send_hmp_command(&self, qmp_port: u16, hmp_cmd: &str) -> Result<String> {
        let escaped = hmp_cmd.replace('\\', "\\\\").replace('"', "\\\"");
        let payload = format!(
            "{{\"execute\":\"human-monitor-command\",\"arguments\":{{\"command-line\":\"{}\"}}}}",
            escaped
        );
        self.send_qmp_execute(qmp_port, &payload).await
    }

    /// Sends a JSON QMP command (`system_powerdown`, `stop`, `cont`, `quit`) to the VM's QMP TCP socket.
    pub async fn send_qmp_command(&self, qmp_port: u16, execute_cmd: &str) -> Result<()> {
        let payload = format!("{{\"execute\":\"{}\"}}", execute_cmd);
        self.send_qmp_execute(qmp_port, &payload).await?;
        Ok(())
    }

    /// Builds the full command line argument list for an Amazon Linux 2023 VM.
    pub fn build_qemu_args(&self, rec: &InstanceRecord) -> Vec<String> {
        let mem_mib = (rec.details.memory_total_bytes / (1024 * 1024)).max(256);

        let mut args = vec![
            "-name".to_string(),
            rec.details.name.clone(),
            "-m".to_string(),
            format!("{}M", mem_mib),
            "-smp".to_string(),
            rec.details.cpus.to_string(),
            "-display".to_string(),
            "none".to_string(),
            "-serial".to_string(),
            format!("file:{}/console.log", rec.instance_dir.to_string_lossy()),
            "-monitor".to_string(),
            "none".to_string(),
            // QMP Control Socket for lifecycle management
            "-qmp".to_string(),
            format!("tcp:127.0.0.1:{},server,nowait", rec.qmp_port),
            // Primary VirtIO Root Disk (QCOW2 overlay backed by AL2023 base image)
            "-drive".to_string(),
            format!(
                "file={},if=virtio,format=qcow2,cache=writeback,discard=unmap",
                rec.disk_path.to_string_lossy()
            ),
            // NoCloud cidata seed.iso attached as read-only CD-ROM
            "-drive".to_string(),
            format!(
                "file={},media=cdrom,readonly=on",
                rec.seed_iso_path.to_string_lossy()
            ),
            // VirtIO Network with user-mode SSH port forwarding
            "-netdev".to_string(),
            format!(
                "user,id=net0,hostfwd=tcp:127.0.0.1:{}-:22",
                rec.details.ssh_port
            ),
            "-device".to_string(),
            format!("virtio-net-pci,netdev=net0,mac={}", rec.mac_address),
            "-device".to_string(),
            "virtio-rng-pci".to_string(),
        ];

        // Platform-optimal machine type and hardware acceleration
        if cfg!(target_arch = "aarch64") {
            args.extend(["-M".to_string(), "virt,highmem=on".to_string()]);
        }

        if cfg!(target_os = "linux") {
            if std::path::Path::new("/dev/kvm").exists() {
                args.extend(["-enable-kvm".to_string(), "-cpu".to_string(), "host".to_string()]);
            } else {
                args.extend(["-accel".to_string(), "tcg".to_string(), "-cpu".to_string(), "max".to_string()]);
            }
        } else if cfg!(target_os = "macos") {
            args.extend(["-accel".to_string(), "hvf".to_string(), "-cpu".to_string(), "host".to_string()]);
        } else if cfg!(target_os = "windows") {
            args.extend(["-accel".to_string(), "whpx,kernel-irqchip=off".to_string(), "-accel".to_string(), "tcg".to_string()]);
        }

        args
    }
}

#[async_trait]
impl super::VirtualMachineDriver for QemuDriver {
    fn driver_name(&self) -> &str {
        "qemu"
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

        let mut cmd = Command::new(&self.qemu_img_bin);
        cmd.args([
            "create",
            "-f",
            "qcow2",
            "-F",
            "qcow2",
            "-b",
            &base_image.to_string_lossy(),
            &overlay_path.to_string_lossy(),
        ]);

        // If the requested disk size exceeds the base image default (25 GiB), expand it
        let size_gib = disk_bytes / (1024 * 1024 * 1024);
        if size_gib > 25 {
            cmd.arg(format!("{}G", size_gib));
        }

        let status = cmd
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .await
            .context("Failed to execute qemu-img create")?;

        if !status.success() {
            bail!("qemu-img failed to create overlay disk {:?}", overlay_path);
        }
        Ok(())
    }

    async fn start_vm(&self, rec: &InstanceRecord) -> Result<u32> {
        let args = self.build_qemu_args(rec);
        let mut child = Command::new(&self.qemu_bin)
            .args(&args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .with_context(|| format!("Failed to spawn QEMU hypervisor process: {}", self.qemu_bin))?;

        let pid = child.id().unwrap_or(0);
        tokio::spawn(async move {
            let _ = child.wait().await;
        });
        Ok(pid)
    }

    async fn stop_vm(&self, rec: &InstanceRecord, force: bool) -> Result<()> {
        if force {
            let _ = self.send_qmp_command(rec.qmp_port, "quit").await;
        } else {
            let _ = self.send_qmp_command(rec.qmp_port, "system_powerdown").await;
        }

        // Wait for the QEMU process to cleanly exit and release disk locks (up to 5 seconds)
        let qmp_addr = format!("127.0.0.1:{}", rec.qmp_port);
        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_secs(5) {
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            if tokio::net::TcpStream::connect(&qmp_addr).await.is_err() {
                // Ensure OS file lock is completely released
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                return Ok(());
            }
        }

        // If still running after 5s, force quit via QMP
        let _ = self.send_qmp_command(rec.qmp_port, "quit").await;
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        Ok(())
    }

    async fn suspend_vm(&self, rec: &InstanceRecord) -> Result<()> {
        self.send_qmp_command(rec.qmp_port, "stop").await
    }

    async fn resume_vm(&self, rec: &InstanceRecord) -> Result<()> {
        self.send_qmp_command(rec.qmp_port, "cont").await
    }

    async fn take_disk_snapshot(&self, disk_path: &Path, snapshot_name: &str) -> Result<()> {
        for _ in 1..=6 {
            let status = Command::new(&self.qemu_img_bin)
                .args(["snapshot", "-c", snapshot_name, &disk_path.to_string_lossy()])
                .status()
                .await?;
            if status.success() {
                return Ok(());
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        bail!("qemu-img snapshot -c failed for {:?}", disk_path);
    }

    async fn restore_disk_snapshot(&self, disk_path: &Path, snapshot_name: &str) -> Result<()> {
        let status = Command::new(&self.qemu_img_bin)
            .args(["snapshot", "-a", snapshot_name, &disk_path.to_string_lossy()])
            .status()
            .await?;
        if !status.success() {
            bail!("qemu-img snapshot -a failed for {:?}", disk_path);
        }
        Ok(())
    }

    async fn take_live_snapshot(&self, qmp_port: u16, disk_path: &Path, snapshot_name: &str) -> Result<()> {
        match self.send_hmp_command(qmp_port, &format!("savevm {}", snapshot_name)).await {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::warn!("QMP live snapshot failed ({:?}), falling back to offline disk snapshot", e);
                self.take_disk_snapshot(disk_path, snapshot_name).await
            }
        }
    }

    async fn restore_live_snapshot(&self, qmp_port: u16, disk_path: &Path, snapshot_name: &str) -> Result<()> {
        match self.send_hmp_command(qmp_port, &format!("loadvm {}", snapshot_name)).await {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::warn!("QMP live restore failed ({:?}), falling back to offline disk restore", e);
                self.restore_disk_snapshot(disk_path, snapshot_name).await
            }
        }
    }
}
