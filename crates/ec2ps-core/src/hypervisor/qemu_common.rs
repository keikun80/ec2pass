//! Common QEMU hypervisor implementation and process / QMP socket manager.

use super::guide::get_hypervisor_install_guide;
use crate::defaults::{timeouts, LOOPBACK, MIN_MEMORY_BYTES};
use crate::state::InstanceRecord;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::process::Command;

/// Directories searched for QEMU binaries (empty string = `PATH` lookup).
const QEMU_SEARCH_DIRS: &[&str] = &[
    "",
    r"C:\Program Files\qemu",
    r"C:\Program Files (x86)\qemu",
    "/usr/local/bin",
    "/opt/homebrew/bin",
    "/usr/bin",
    "/usr/libexec",
];

/// Well-known aarch64 UEFI firmware locations across macOS, Linux and Windows.
const AARCH64_UEFI_FIRMWARE_PATHS: &[&str] = &[
    // Homebrew macOS (Apple Silicon & Intel)
    "/opt/homebrew/share/qemu/edk2-aarch64-code.fd",
    "/usr/local/share/qemu/edk2-aarch64-code.fd",
    // Linux RHEL / Rocky / Fedora
    "/usr/share/edk2/aarch64/QEMU_EFI.fd",
    "/usr/share/edk2/aarch64/QEMU_EFI.silent.fd",
    // Linux Debian / Ubuntu
    "/usr/share/qemu-efi-aarch64/QEMU_EFI.fd",
    "/usr/share/qemu-efi/QEMU_EFI.fd",
    // Linux generic / Arch / openSUSE / Alpine
    "/usr/share/qemu/edk2-aarch64-code.fd",
    "/usr/share/qemu/qemu-uefi-aarch64.bin",
    // Windows
    r"C:\Program Files\qemu\share\edk2-aarch64-code.fd",
];

/// File in the instance directory that receives QEMU's stderr.
pub const QEMU_LOG_NAME: &str = "qemu.log";

/// Last lines of an instance's qemu.log, for error messages.
pub fn read_qemu_log_tail(instance_dir: &std::path::Path) -> String {
    let text = std::fs::read_to_string(instance_dir.join(QEMU_LOG_NAME)).unwrap_or_default();
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        "no output in qemu.log".to_string()
    } else {
        lines[lines.len().saturating_sub(10)..].join("\n")
    }
}

/// Platform-agnostic QEMU Hypervisor Driver.
pub struct QemuDriver {
    pub qemu_bin: String,
    pub qemu_img_bin: String,
}

impl QemuDriver {
    /// Detects QEMU binaries in PATH or known installation locations.
    /// Returns None if QEMU binaries cannot be found.
    pub fn detect_optional() -> Option<Self> {
        let candidate_bins = if cfg!(target_arch = "aarch64") {
            vec!["qemu-system-aarch64"]
        } else {
            vec!["qemu-system-x86_64", "qemu-kvm"]
        };

        let candidate_dirs: Vec<PathBuf> = QEMU_SEARCH_DIRS.iter().map(PathBuf::from).collect();

        let mut found_img = None;
        for dir in &candidate_dirs {
            let img_path = if dir.as_os_str().is_empty() {
                PathBuf::from("qemu-img")
            } else if cfg!(windows) {
                dir.join("qemu-img.exe")
            } else {
                dir.join("qemu-img")
            };

            if let Ok(s) = std::process::Command::new(&img_path)
                .arg("--version")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
            {
                if s.success() {
                    found_img = Some(img_path.to_string_lossy().to_string());
                    break;
                }
            }
        }
        let found_img = found_img?;

        for base_bin in &candidate_bins {
            for dir in &candidate_dirs {
                let bin_path = if dir.as_os_str().is_empty() {
                    PathBuf::from(base_bin)
                } else if cfg!(windows) {
                    dir.join(format!("{}.exe", base_bin))
                } else {
                    dir.join(base_bin)
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
                            qemu_img_bin: found_img,
                        });
                    }
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
        let addr = format!("{}:{}", LOOPBACK, qmp_port);
        let mut stream = None;
        for i in 0..timeouts::QMP_CONNECT_ATTEMPTS {
            match TcpStream::connect(&addr).await {
                Ok(s) => {
                    stream = Some(s);
                    break;
                }
                Err(_) if i + 1 < timeouts::QMP_CONNECT_ATTEMPTS => {
                    tokio::time::sleep(timeouts::QMP_POLL).await;
                }
                Err(e) => {
                    return Err(e)
                        .with_context(|| format!("Failed to connect to QMP socket at {}", addr));
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
        let resp = self.send_qmp_execute(qmp_port, &payload).await?;
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&resp) {
            if let Some(ret) = v.get("return").and_then(|r| r.as_str()) {
                let trimmed = ret.trim();
                if trimmed.starts_with("Error")
                    || trimmed.starts_with("error:")
                    || trimmed.contains("could not open")
                    || trimmed.contains("does not support snapshots")
                {
                    bail!("HMP '{}' failed: {}", hmp_cmd, trimmed);
                }
                return Ok(ret.to_string());
            }
        }
        Ok(resp)
    }

    /// Sends a JSON QMP command (`system_powerdown`, `stop`, `cont`, `quit`) to the VM's QMP TCP socket.
    pub async fn send_qmp_command(&self, qmp_port: u16, execute_cmd: &str) -> Result<()> {
        let payload = format!("{{\"execute\":\"{}\"}}", execute_cmd);
        self.send_qmp_execute(qmp_port, &payload).await?;
        Ok(())
    }

    /// Discovers the edk2 UEFI firmware binary for aarch64 VM boot if available.
    pub fn find_aarch64_uefi_firmware(&self) -> Option<PathBuf> {
        // 1. Check relative to discovered QEMU binary directory
        if let Some(bin_parent) = Path::new(&self.qemu_bin).parent() {
            let share_candidates = [
                bin_parent.join("share/qemu/edk2-aarch64-code.fd"),
                bin_parent.join("../share/qemu/edk2-aarch64-code.fd"),
                bin_parent.join("edk2-aarch64-code.fd"),
            ];
            for cand in &share_candidates {
                if cand.exists() {
                    return Some(cand.clone());
                }
            }
        }

        // 2. Standard system locations across macOS, Linux, and Windows

        for p in AARCH64_UEFI_FIRMWARE_PATHS {
            let path = PathBuf::from(p);
            if path.exists() {
                return Some(path);
            }
        }
        None
    }

    /// Builds the full command line argument list for an Amazon Linux 2023 VM.
    pub fn build_qemu_args(&self, rec: &InstanceRecord) -> Vec<String> {
        let mem_mib = rec.details.memory_total_bytes.max(MIN_MEMORY_BYTES) / (1024 * 1024);

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
            format!("tcp:{}:{},server,nowait", LOOPBACK, rec.qmp_port),
            // Primary VirtIO Root Disk (QCOW2 overlay backed by AL2023 base image)
            "-drive".to_string(),
            format!(
                "file={},if=virtio,format=qcow2,cache=writeback,discard=unmap",
                rec.disk_path.to_string_lossy()
            ),
            // NoCloud cidata seed.iso attached as read-only CD-ROM (snapshot=off prevents snapshot locking)
            "-drive".to_string(),
            format!(
                "file={},media=cdrom,readonly=on,snapshot=off",
                rec.seed_iso_path.to_string_lossy()
            ),
            // VirtIO Network with user-mode SSH port forwarding
            "-netdev".to_string(),
            format!(
                "user,id=net0,hostfwd=tcp:{}:{}-:22",
                LOOPBACK, rec.details.ssh_port
            ),
            "-device".to_string(),
            format!("virtio-net-pci,netdev=net0,mac={}", rec.mac_address),
            "-device".to_string(),
            "virtio-rng-pci".to_string(),
        ];

        // VirtIO 9p shared filesystem mounts
        for (idx, (_tgt, spec)) in rec.details.mounts.iter().enumerate() {
            let tag = format!("ec2ps_mount_{}", idx);
            args.extend([
                "-virtfs".to_string(),
                format!(
                    "local,path={},mount_tag={},security_model=none",
                    spec.source_path, tag
                ),
            ]);
        }

        // Additional user-requested network devices (--network <bridge>)
        for (idx, net) in rec.networks.iter().enumerate() {
            let net_id = format!("net{}", idx + 1);
            let mac = format!(
                "52:54:00:24:{:02x}:{:02x}",
                (idx + 1) & 0xff,
                (rec.qmp_port % 250) as u8
            );
            let br_name = if net == "bridge" || net == "bridged" || net.is_empty() {
                "br0"
            } else {
                net.as_str()
            };
            args.extend([
                "-netdev".to_string(),
                format!("bridge,id={},br={}", net_id, br_name),
                "-device".to_string(),
                format!("virtio-net-pci,netdev={},mac={}", net_id, mac),
            ]);
        }

        // Platform-optimal machine type and hardware acceleration
        if cfg!(target_arch = "aarch64") {
            args.extend(["-M".to_string(), "virt,highmem=on".to_string()]);
            if let Some(bios_path) = self.find_aarch64_uefi_firmware() {
                args.extend(["-bios".to_string(), bios_path.to_string_lossy().to_string()]);
            }
        }

        if cfg!(target_os = "linux") {
            if std::path::Path::new("/dev/kvm").exists() {
                args.extend([
                    "-enable-kvm".to_string(),
                    "-cpu".to_string(),
                    "host".to_string(),
                ]);
            } else {
                args.extend([
                    "-accel".to_string(),
                    "tcg".to_string(),
                    "-cpu".to_string(),
                    "max".to_string(),
                ]);
            }
        } else if cfg!(target_os = "macos") {
            args.extend([
                "-accel".to_string(),
                "hvf".to_string(),
                "-cpu".to_string(),
                "host".to_string(),
            ]);
        } else if cfg!(target_os = "windows") {
            args.extend([
                "-accel".to_string(),
                "whpx,kernel-irqchip=off".to_string(),
                "-accel".to_string(),
                "tcg".to_string(),
            ]);
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

        let output = cmd
            .stdin(std::process::Stdio::null())
            .output()
            .await
            .context("Failed to execute qemu-img create")?;

        if !output.status.success() {
            bail!(
                "qemu-img failed to create overlay disk {:?}: {}",
                overlay_path,
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        Ok(())
    }

    async fn resize_disk(&self, disk_path: &Path, new_bytes: u64) -> Result<()> {
        if !disk_path.exists() {
            bail!("Disk file {:?} does not exist", disk_path);
        }

        let mut cmd = Command::new(&self.qemu_img_bin);
        cmd.args([
            "resize",
            "-f",
            "qcow2",
            &disk_path.to_string_lossy(),
            &new_bytes.to_string(),
        ]);

        let output = cmd
            .output()
            .await
            .context("Failed to execute qemu-img resize")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("qemu-img resize failed: {}", stderr.trim());
        }
        Ok(())
    }

    async fn start_vm(&self, rec: &InstanceRecord) -> Result<u32> {
        let args = self.build_qemu_args(rec);
        // QEMU's own errors (bad options, missing accelerator, locked disk) go to qemu.log so a
        // failed start is reported instead of silently waiting for SSH until the timeout.
        let log_path = rec.instance_dir.join(QEMU_LOG_NAME);
        let log_file = std::fs::File::create(&log_path)
            .with_context(|| format!("Failed to create {}", log_path.display()))?;
        let mut child = Command::new(&self.qemu_bin)
            .args(&args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(log_file)
            .spawn()
            .with_context(|| {
                format!("Failed to spawn QEMU hypervisor process: {}", self.qemu_bin)
            })?;

        tokio::time::sleep(timeouts::QEMU_START_CHECK).await;
        if let Ok(Some(status)) = child.try_wait() {
            bail!(
                "QEMU exited immediately ({}): {}",
                status,
                read_qemu_log_tail(&rec.instance_dir)
            );
        }

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
            let _ = self
                .send_qmp_command(rec.qmp_port, "system_powerdown")
                .await;
        }

        // Wait for the QEMU process to cleanly exit and release disk locks
        let qmp_addr = format!("{}:{}", LOOPBACK, rec.qmp_port);
        let start = std::time::Instant::now();
        while start.elapsed() < timeouts::QMP_SHUTDOWN_WAIT {
            tokio::time::sleep(timeouts::QMP_POLL).await;
            if tokio::net::TcpStream::connect(&qmp_addr).await.is_err() {
                // Ensure OS file lock is completely released
                tokio::time::sleep(timeouts::QEMU_LOCK_RELEASE).await;
                return Ok(());
            }
        }

        // If still running after the grace period, force quit via QMP
        let _ = self.send_qmp_command(rec.qmp_port, "quit").await;
        tokio::time::sleep(timeouts::QEMU_FORCE_QUIT_SETTLE).await;
        Ok(())
    }

    async fn suspend_vm(&self, rec: &InstanceRecord) -> Result<()> {
        self.send_qmp_command(rec.qmp_port, "stop").await
    }

    async fn resume_vm(&self, rec: &InstanceRecord) -> Result<()> {
        self.send_qmp_command(rec.qmp_port, "cont").await
    }

    async fn take_disk_snapshot(&self, disk_path: &Path, snapshot_name: &str) -> Result<()> {
        for _ in 1..=timeouts::DISK_SNAPSHOT_MAX_ATTEMPTS {
            let status = Command::new(&self.qemu_img_bin)
                .args([
                    "snapshot",
                    "-c",
                    snapshot_name,
                    &disk_path.to_string_lossy(),
                ])
                .status()
                .await?;
            if status.success() {
                return Ok(());
            }
            tokio::time::sleep(timeouts::DISK_SNAPSHOT_RETRY_DELAY).await;
        }
        bail!("qemu-img snapshot -c failed for {:?}", disk_path);
    }

    async fn restore_disk_snapshot(&self, disk_path: &Path, snapshot_name: &str) -> Result<()> {
        let status = Command::new(&self.qemu_img_bin)
            .args([
                "snapshot",
                "-a",
                snapshot_name,
                &disk_path.to_string_lossy(),
            ])
            .status()
            .await?;
        if !status.success() {
            bail!("qemu-img snapshot -a failed for {:?}", disk_path);
        }
        Ok(())
    }

    async fn take_live_snapshot(
        &self,
        qmp_port: u16,
        disk_path: &Path,
        snapshot_name: &str,
    ) -> Result<()> {
        match self
            .send_hmp_command(qmp_port, &format!("savevm {}", snapshot_name))
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::warn!(
                    "QMP live snapshot failed ({:?}), falling back to offline disk snapshot",
                    e
                );
                self.take_disk_snapshot(disk_path, snapshot_name).await
            }
        }
    }

    async fn restore_live_snapshot(
        &self,
        qmp_port: u16,
        _disk_path: &Path,
        snapshot_name: &str,
    ) -> Result<()> {
        self.send_hmp_command(qmp_port, &format!("loadvm {}", snapshot_name))
            .await?;
        Ok(())
    }
}
