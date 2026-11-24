//! Daemon internal SSH keypair generator, remote command execution (`exec` / `shell`),
//! SFTP file transfer (`transfer`), and host-to-guest directory synchronization (`mount`).

use anyhow::{bail, Context, Result};
use base64::Engine;
use ring::rand::SecureRandom;
use ring::signature::KeyPair;
use std::path::{Path, PathBuf};
use tokio::process::Command;

use crate::defaults::{timeouts, LOOPBACK};

/// File name of the daemon's SSH private key (the public key adds `.pub`).
pub const KEY_NAME: &str = "id_ed25519_ec2pass";

/// Key names written by earlier releases, reused if present so existing VMs stay reachable.
pub const LEGACY_KEY_NAMES: &[&str] = &["id_ed25519_mp", "id_ed25519_alpass"];

/// Comment embedded in generated public keys.
pub const KEY_COMMENT: &str = "ec2pass@al2023-daemon";

/// `ssh-keygen` locations tried in order (bare name = `PATH` lookup).
#[cfg(windows)]
const SSH_KEYGEN_CANDIDATES: &[&str] = &[
    "ssh-keygen",
    r"C:\Windows\System32\OpenSSH\ssh-keygen.exe",
    r"C:\Program Files\Git\usr\bin\ssh-keygen.exe",
];
#[cfg(not(windows))]
const SSH_KEYGEN_CANDIDATES: &[&str] = &[
    "ssh-keygen",
    "/usr/bin/ssh-keygen",
    "/usr/local/bin/ssh-keygen",
];

pub struct SshManager {
    _key_dir: PathBuf,
    private_key_path: PathBuf,
    public_key_path: PathBuf,
}

impl SshManager {
    pub fn new(key_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&key_dir)?;
        let key_name = if key_dir.join(KEY_NAME).exists() {
            KEY_NAME
        } else {
            LEGACY_KEY_NAMES
                .iter()
                .copied()
                .find(|name| key_dir.join(name).exists())
                .unwrap_or(KEY_NAME)
        };
        let private_key_path = key_dir.join(key_name);
        let public_key_path = key_dir.join(format!("{}.pub", key_name));

        let mgr = Self {
            _key_dir: key_dir,
            private_key_path,
            public_key_path,
        };
        mgr.ensure_keypair()?;
        Ok(mgr)
    }

    pub fn private_key_path(&self) -> &Path {
        &self.private_key_path
    }

    pub fn public_key_string(&self) -> Result<String> {
        Ok(std::fs::read_to_string(&self.public_key_path)?
            .trim()
            .to_string())
    }

    /// Generates an Ed25519 SSH keypair via `ssh-keygen` if available, or dynamically creates
    /// a cryptographically secure OpenSSH Ed25519 keypair in pure Rust via `ring`.
    fn ensure_keypair(&self) -> Result<()> {
        if self.private_key_path.exists() && self.public_key_path.exists() {
            return Ok(());
        }

        // Try system ssh-keygen first across standard locations
        for candidate in SSH_KEYGEN_CANDIDATES {
            let status = std::process::Command::new(candidate)
                .args(["-t", "ed25519", "-N", "", "-C", KEY_COMMENT, "-f"])
                .arg(&self.private_key_path)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();

            if let Ok(s) = status {
                if s.success() && self.public_key_path.exists() {
                    self.enforce_key_permissions();
                    return Ok(());
                }
            }
        }

        // Dynamic, cryptographically secure pure-Rust Ed25519 generation (no hardcoded keys)
        tracing::info!("ssh-keygen not available; generating dynamic Ed25519 keypair via ring");
        let (priv_pem, pub_str) = generate_dynamic_ed25519_keypair()?;
        std::fs::write(&self.private_key_path, priv_pem)?;
        std::fs::write(&self.public_key_path, pub_str)?;
        self.enforce_key_permissions();
        Ok(())
    }

    /// Enforces strict POSIX permissions (0600) on private SSH key for OpenSSH client compliance
    fn enforce_key_permissions(&self) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(
                &self.private_key_path,
                std::fs::Permissions::from_mode(0o600),
            );
        }
    }

    /// Executes a command inside a running Amazon Linux 2023 VM over SSH.
    pub async fn exec_command(
        &self,
        _instance_name: &str,
        _instance_dir: &Path,
        ssh_port: u16,
        ssh_user: &str,
        command: &[String],
        working_dir: Option<&str>,
    ) -> Result<(String, String, i32)> {
        if command.is_empty() {
            bail!("Command cannot be empty");
        }

        let remote_cmd = if let Some(wd) = working_dir {
            format!("cd '{}' && {}", wd, command.join(" "))
        } else {
            command.join(" ")
        };

        let mut last_res = (String::new(), String::new(), 1);
        for attempt in 1..=timeouts::SSH_MAX_ATTEMPTS {
            let output = Command::new("ssh")
                .args([
                    "-i",
                    &self.private_key_path.to_string_lossy(),
                    "-p",
                    &ssh_port.to_string(),
                    "-o",
                    "StrictHostKeyChecking=no",
                    "-o",
                    "UserKnownHostsFile=/dev/null",
                    "-o",
                    "LogLevel=ERROR",
                    "-o",
                    "ConnectTimeout=10",
                    &format!("{}@{}", ssh_user, LOOPBACK),
                    &remote_cmd,
                ])
                .output()
                .await
                .context("Failed to invoke ssh client")?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let code = output.status.code().unwrap_or(1);
            last_res = (stdout, stderr, code);

            if code == 255
                && attempt < timeouts::SSH_MAX_ATTEMPTS
                && (last_res.1.contains("banner exchange")
                    || last_res.1.contains("Connection refused")
                    || last_res.1.contains("Connection reset")
                    || last_res.1.contains("kex_exchange_identification")
                    || last_res.1.contains("closed by remote host"))
            {
                tokio::time::sleep(timeouts::SSH_RETRY_DELAY).await;
                continue;
            }
            break;
        }
        Ok(last_res)
    }

    /// Polls the instance SSH port until the VM responds and cloud-init finishes,
    /// or returns an error if timeout_secs is exceeded.
    pub async fn wait_until_ready(
        &self,
        instance_name: &str,
        instance_dir: &Path,
        ssh_port: u16,
        ssh_user: &str,
        timeout_secs: u64,
    ) -> Result<()> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);
        let mut attempts = 0;

        while start.elapsed() < timeout {
            attempts += 1;
            let elapsed = start.elapsed().as_secs();

            // 1. Check if SSH TCP port is open
            let addr = format!("{}:{}", LOOPBACK, ssh_port);
            if let Ok(Ok(_)) = tokio::time::timeout(
                timeouts::SSH_TCP_PROBE,
                tokio::net::TcpStream::connect(&addr),
            )
            .await
            {
                // 2. Check if SSH authentication succeeds and cloud-init status is ready
                let check_cmd = vec!["cloud-init status --wait || true".to_string()];
                if let Ok((stdout, _stderr, code)) = self
                    .exec_command(
                        instance_name,
                        instance_dir,
                        ssh_port,
                        ssh_user,
                        &check_cmd,
                        None,
                    )
                    .await
                {
                    if code == 0 || stdout.contains("done") || stdout.contains("status:") {
                        tracing::info!(
                            "Instance '{}' is ready (SSH port {} responded after {}s, {} attempts)",
                            instance_name,
                            ssh_port,
                            elapsed,
                            attempts
                        );
                        return Ok(());
                    }
                }
            }

            tokio::time::sleep(timeouts::SSH_READY_POLL).await;
        }

        bail!(
            "Timed out waiting for instance '{}' to become ready (SSH port {} not responding or cloud-init incomplete after {} seconds).",
            instance_name,
            ssh_port,
            timeout_secs
        )
    }

    /// Transfers files or directories between Host and Guest (`ec2pass transfer`).
    #[allow(clippy::too_many_arguments)]
    pub async fn transfer_path(
        &self,
        instance_dir: &Path,
        ssh_port: u16,
        ssh_user: &str,
        src: &str,
        dst: &str,
        recursive: bool,
        parents: bool,
    ) -> Result<()> {
        if parents {
            if let Some((inst, guest_path)) = dst.split_once(':') {
                if inst.len() > 1 && !inst.contains('/') && !inst.contains('\\') {
                    if let Some(parent) = Path::new(guest_path).parent() {
                        let p_str = parent.to_string_lossy();
                        if !p_str.is_empty() && p_str != "/" {
                            let _ = self
                                .exec_command(
                                    "instance",
                                    instance_dir,
                                    ssh_port,
                                    ssh_user,
                                    &[format!(
                                        "sudo mkdir -p '{}' && sudo chown -R {}:{} '{}'",
                                        p_str, ssh_user, ssh_user, p_str
                                    )],
                                    None,
                                )
                                .await;
                        }
                    }
                } else if let Some(parent) = Path::new(dst).parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
            } else if let Some(parent) = Path::new(dst).parent() {
                let _ = std::fs::create_dir_all(parent);
            }
        }

        let format_scp_endpoint = |spec: &str| -> String {
            if let Some((inst, guest_path)) = spec.split_once(':') {
                if inst.len() > 1 && !inst.contains('/') && !inst.contains('\\') {
                    return format!("{}@{}:{}", ssh_user, LOOPBACK, guest_path);
                }
            }
            spec.to_string()
        };

        let mut cmd = Command::new("scp");
        cmd.args([
            "-i",
            &self.private_key_path.to_string_lossy(),
            "-P",
            &ssh_port.to_string(),
            "-o",
            "StrictHostKeyChecking=no",
            "-o",
            "UserKnownHostsFile=/dev/null",
        ]);
        if recursive {
            cmd.arg("-r");
        }
        cmd.arg(format_scp_endpoint(src));
        cmd.arg(format_scp_endpoint(dst));

        let status = cmd.status().await.context("Failed to execute scp")?;
        if !status.success() {
            bail!("scp transfer failed with exit code {:?}", status.code());
        }
        Ok(())
    }

    /// Synchronizes or mounts a host directory into a guest directory mount path.
    /// Uses 9p (virtfs) if an active mount_tag is provided, falling back to scp transfer.
    pub async fn sync_mount(
        &self,
        instance_dir: &Path,
        ssh_port: u16,
        ssh_user: &str,
        host_src: &str,
        guest_target: &str,
        mount_tag: Option<&str>,
    ) -> Result<()> {
        let src_path = Path::new(host_src);
        if !src_path.exists() {
            bail!("Host source path '{}' does not exist", host_src);
        }

        // 1. Ensure target directory exists in guest with proper ownership and write permissions
        let mkdir_cmd = vec![format!(
            "sudo mkdir -p '{0}' && sudo chown -R {1}:{1} '{0}' && sudo chmod -R u+w '{0}'",
            guest_target, ssh_user
        )];
        self.exec_command(
            "instance",
            instance_dir,
            ssh_port,
            ssh_user,
            &mkdir_cmd,
            None,
        )
        .await
        .context("Failed to initialize target mount directory in guest")?;

        // 2. If a mount_tag is supplied, attempt kernel 9p mount
        let mut mounted_9p = false;
        if let Some(tag) = mount_tag {
            let mnt_cmd = vec![format!(
                "sudo modprobe 9pnet_virtio 2>/dev/null; sudo modprobe 9p 2>/dev/null; if ! mountpoint -q '{0}'; then sudo mount -t 9p -o trans=virtio,version=9p2000.L,access=any,msize=1048576 '{1}' '{0}' 2>/dev/null || true; fi; mountpoint -q '{0}'",
                guest_target, tag
            )];
            if let Ok((_out, _err, code)) = self
                .exec_command("instance", instance_dir, ssh_port, ssh_user, &mnt_cmd, None)
                .await
            {
                if code == 0 {
                    mounted_9p = true;
                }
            }
        }

        // 3. Fallback: synchronize files via scp if 9p mount is not active
        if !mounted_9p {
            let scp_src = if src_path.is_dir() {
                format!("{}/.", host_src.trim_end_matches('/'))
            } else {
                host_src.to_string()
            };

            let mut cmd = Command::new("scp");
            cmd.args([
                "-i",
                &self.private_key_path.to_string_lossy(),
                "-P",
                &ssh_port.to_string(),
                "-o",
                "StrictHostKeyChecking=no",
                "-o",
                "UserKnownHostsFile=/dev/null",
                "-o",
                "LogLevel=ERROR",
                "-r",
                &scp_src,
                &format!("{}@{}:{}", ssh_user, LOOPBACK, guest_target),
            ]);

            let status = cmd
                .status()
                .await
                .context("Failed to execute mount sync scp")?;
            if !status.success() {
                bail!(
                    "Mount synchronization failed with exit code {:?}",
                    status.code()
                );
            }
        }
        Ok(())
    }
}

pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

fn put_u32(buf: &mut Vec<u8>, val: u32) {
    buf.extend_from_slice(&val.to_be_bytes());
}

fn put_string(buf: &mut Vec<u8>, s: &str) {
    put_u32(buf, s.len() as u32);
    buf.extend_from_slice(s.as_bytes());
}

fn put_bytes(buf: &mut Vec<u8>, b: &[u8]) {
    put_u32(buf, b.len() as u32);
    buf.extend_from_slice(b);
}

/// Generates a fresh, dynamic OpenSSH Ed25519 keypair in pure Rust via `ring`.
///
/// Output:
/// - `private_key_pem`: OpenSSH v1 format (`-----BEGIN OPENSSH PRIVATE KEY-----`)
/// - `public_key_str`: OpenSSH public key format (`ssh-ed25519 AAAAC3NzaC1...`)
pub fn generate_dynamic_ed25519_keypair() -> Result<(String, String)> {
    let rng = ring::rand::SystemRandom::new();
    let pkcs8_doc = ring::signature::Ed25519KeyPair::generate_pkcs8(&rng)
        .map_err(|e| anyhow::anyhow!("Failed to generate PKCS#8 Ed25519 key: {}", e))?;
    let key_pair = ring::signature::Ed25519KeyPair::from_pkcs8(pkcs8_doc.as_ref())
        .map_err(|e| anyhow::anyhow!("Failed to parse generated Ed25519 keypair: {}", e))?;

    let pubkey_bytes = key_pair.public_key().as_ref();
    let pkcs8_bytes = pkcs8_doc.as_ref();
    if pkcs8_bytes.len() < 32 {
        bail!("Generated PKCS#8 document is too short");
    }
    let seed = &pkcs8_bytes[pkcs8_bytes.len() - 32..];

    // 1. Construct public key blob (SSH wire format)
    let mut pub_blob = Vec::with_capacity(64);
    put_string(&mut pub_blob, "ssh-ed25519");
    put_bytes(&mut pub_blob, pubkey_bytes);

    let pub_b64 = base64::engine::general_purpose::STANDARD.encode(&pub_blob);
    let public_key_str = format!("ssh-ed25519 {} ec2pass@al2023-daemon\n", pub_b64);

    // 2. Construct OpenSSH v1 private key blob
    let mut priv_blob = Vec::new();
    priv_blob.extend_from_slice(b"openssh-key-v1\0");

    put_string(&mut priv_blob, "none"); // cipher
    put_string(&mut priv_blob, "none"); // kdf
    put_string(&mut priv_blob, ""); // kdf options
    put_u32(&mut priv_blob, 1); // number of keys
    put_bytes(&mut priv_blob, &pub_blob); // public key list

    // Unencrypted private key payload
    let mut priv_payload = Vec::new();
    let mut check_bytes = [0u8; 4];
    let _ = rng.fill(&mut check_bytes);
    let checkint = u32::from_ne_bytes(check_bytes);
    put_u32(&mut priv_payload, checkint);
    put_u32(&mut priv_payload, checkint);

    put_string(&mut priv_payload, "ssh-ed25519");
    put_bytes(&mut priv_payload, pubkey_bytes);

    let mut priv_and_pub = Vec::with_capacity(64);
    priv_and_pub.extend_from_slice(seed);
    priv_and_pub.extend_from_slice(pubkey_bytes);
    put_bytes(&mut priv_payload, &priv_and_pub);

    put_string(&mut priv_payload, KEY_COMMENT);

    // Padding (multiple of 8 bytes for "none" cipher)
    let block_size = 8;
    let rem = priv_payload.len() % block_size;
    let pad_len = if rem == 0 { 0 } else { block_size - rem };
    for i in 1..=pad_len {
        priv_payload.push(i as u8);
    }

    put_bytes(&mut priv_blob, &priv_payload);

    let priv_b64 = base64::engine::general_purpose::STANDARD.encode(&priv_blob);
    let mut private_key_pem = String::from("-----BEGIN OPENSSH PRIVATE KEY-----\n");
    for chunk in priv_b64.as_bytes().chunks(70) {
        if let Ok(s) = std::str::from_utf8(chunk) {
            private_key_pem.push_str(s);
            private_key_pem.push('\n');
        }
    }
    private_key_pem.push_str("-----END OPENSSH PRIVATE KEY-----\n");

    Ok((private_key_pem, public_key_str))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_key_dir(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("ec2pass-sshtest-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_prefers_legacy_key_when_primary_missing() {
        let dir = scratch_key_dir("legacy");
        std::fs::write(dir.join("id_ed25519_alpass"), "priv").unwrap();
        std::fs::write(dir.join("id_ed25519_alpass.pub"), "ssh-ed25519 AAAA legacy").unwrap();

        let mgr = SshManager::new(dir.clone()).unwrap();
        assert_eq!(mgr.private_key_path(), dir.join("id_ed25519_alpass"));
        assert_eq!(mgr.public_key_string().unwrap(), "ssh-ed25519 AAAA legacy");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_primary_key_wins_over_legacy() {
        let dir = scratch_key_dir("primary");
        for name in [KEY_NAME, "id_ed25519_mp"] {
            std::fs::write(dir.join(name), "priv").unwrap();
            std::fs::write(dir.join(format!("{}.pub", name)), name).unwrap();
        }

        let mgr = SshManager::new(dir.clone()).unwrap();
        assert_eq!(mgr.private_key_path(), dir.join(KEY_NAME));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_generate_dynamic_ed25519_keypair() {
        let (priv_pem, pub_str) = generate_dynamic_ed25519_keypair().expect("generation failed");

        assert!(priv_pem.starts_with("-----BEGIN OPENSSH PRIVATE KEY-----\n"));
        assert!(priv_pem.ends_with("-----END OPENSSH PRIVATE KEY-----\n"));
        assert!(pub_str.starts_with("ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAI"));
        assert!(pub_str.ends_with("ec2pass@al2023-daemon\n"));

        // Verify randomness: subsequent calls produce distinct keys
        let (priv_pem2, pub_str2) = generate_dynamic_ed25519_keypair().expect("generation failed");
        assert_ne!(priv_pem, priv_pem2);
        assert_ne!(pub_str, pub_str2);
    }
}
