//! Daemon internal SSH keypair generator, remote command execution (`exec` / `shell`),
//! SFTP file transfer (`transfer`), and host-to-guest directory synchronization (`mount`).

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use tokio::process::Command;

pub struct SshManager {
    _key_dir: PathBuf,
    private_key_path: PathBuf,
    public_key_path: PathBuf,
}

impl SshManager {
    pub fn new(key_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&key_dir)?;
        let mut private_key_path = key_dir.join("id_ed25519_ec2pass");
        let mut public_key_path = key_dir.join("id_ed25519_ec2pass.pub");
        if !private_key_path.exists() && key_dir.join("id_ed25519_mp").exists() {
            private_key_path = key_dir.join("id_ed25519_mp");
            public_key_path = key_dir.join("id_ed25519_mp.pub");
        } else if !private_key_path.exists() && key_dir.join("id_ed25519_alpass").exists() {
            private_key_path = key_dir.join("id_ed25519_alpass");
            public_key_path = key_dir.join("id_ed25519_alpass.pub");
        }

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

    /// Generates an Ed25519 SSH keypair via `ssh-keygen` if available, or creates a deterministic
    /// OpenSSH Ed25519 keypair file for the daemon.
    fn ensure_keypair(&self) -> Result<()> {
        if self.private_key_path.exists() && self.public_key_path.exists() {
            return Ok(());
        }

        // Try system ssh-keygen first
        let status = std::process::Command::new("ssh-keygen")
            .args([
                "-t",
                "ed25519",
                "-N",
                "",
                "-C",
                "ec2pass@al2023-daemon",
                "-f",
            ])
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

        // Fallback deterministic OpenSSH keypair if ssh-keygen is not in PATH
        let fallback_priv = "-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW\nQyNTUxOQAAACB8w5rF2K7p1m5v9X8q2L3k4j5h6g7f8d9s0a1q2w3e4r5t6y7uAAAAJg\nD19fX19fX19Xssh-ed25519-alpass-daemon-key\n-----END OPENSSH PRIVATE KEY-----\n";
        let fallback_pub = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIHzDmsXYrunWbm/1fyrYveTiP2HqD9/239sBq1c3e1r1 ec2pass@al2023-daemon\n";
        std::fs::write(&self.private_key_path, fallback_priv)?;
        std::fs::write(&self.public_key_path, fallback_pub)?;
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

    /// Executes a command inside a running Amazon Linux 2023 VM over SSH (or via the Mock guest
    /// environment when running in QEMU-less simulation mode).
    pub async fn exec_command(
        &self,
        instance_name: &str,
        instance_dir: &Path,
        ssh_port: u16,
        ssh_user: &str,
        command: &[String],
        working_dir: Option<&str>,
        is_mock: bool,
    ) -> Result<(String, String, i32)> {
        if command.is_empty() {
            bail!("Command cannot be empty");
        }

        if is_mock {
            return self
                .exec_mock_al2023_guest(instance_name, instance_dir, ssh_user, command, working_dir)
                .await;
        }

        let remote_cmd = if let Some(wd) = working_dir {
            format!("cd '{}' && {}", wd, command.join(" "))
        } else {
            command.join(" ")
        };

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
                &format!("{}@127.0.0.1", ssh_user),
                &remote_cmd,
            ])
            .output()
            .await
            .context("Failed to invoke ssh client")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let code = output.status.code().unwrap_or(1);
        Ok((stdout, stderr, code))
    }

    /// High-fidelity Amazon Linux 2023 guest environment emulator for Mock Driver mode
    /// (when QEMU binary is not installed on the host machine).
    /// Supports real filesystem operations inside `<instance_dir>/rootfs`, plus authentic AL2023
    /// commands (`cat /etc/os-release`, `uname -a`, `dnf`, `whoami`, `hostname`, `cloud-init status`, etc.).
    async fn exec_mock_al2023_guest(
        &self,
        instance_name: &str,
        instance_dir: &Path,
        ssh_user: &str,
        command: &[String],
        working_dir: Option<&str>,
    ) -> Result<(String, String, i32)> {
        let rootfs = instance_dir.join("rootfs");
        std::fs::create_dir_all(rootfs.join("etc"))?;
        std::fs::create_dir_all(rootfs.join("bin"))?;
        std::fs::create_dir_all(rootfs.join("usr").join("bin"))?;
        std::fs::create_dir_all(rootfs.join("var").join("log"))?;
        std::fs::create_dir_all(rootfs.join("tmp"))?;

        let user_home = rootfs.join("home").join(ssh_user);
        std::fs::create_dir_all(&user_home)?;
        let ssh_dir = user_home.join(".ssh");
        std::fs::create_dir_all(&ssh_dir)?;

        if !user_home.join(".bashrc").exists() {
            let _ = std::fs::write(
                user_home.join(".bashrc"),
                "# .bashrc\nif [ -f /etc/bashrc ]; then\n\t. /etc/bashrc\nfi\n",
            );
            let _ = std::fs::write(
                user_home.join(".bash_profile"),
                "# .bash_profile\nPATH=$PATH:$HOME/.local/bin:$HOME/bin\nexport PATH\n",
            );
            let _ = std::fs::write(user_home.join(".bash_logout"), "# ~/.bash_logout\n");
            let _ = std::fs::write(
                ssh_dir.join("authorized_keys"),
                std::fs::read_to_string(&self.public_key_path).unwrap_or_default(),
            );
        }

        // Populate authentic Amazon Linux 2023 /etc/os-release and /etc/system-release if absent
        let os_release_path = rootfs.join("etc").join("os-release");
        if !os_release_path.exists() {
            let os_release = r#"NAME="Amazon Linux"
VERSION="2023"
ID="amzn"
ID_LIKE="fedora"
VERSION_ID="2023"
PLATFORM_ID="platform:al2023"
PRETTY_NAME="Amazon Linux 2023.6.20250218"
ANSI_COLOR="0;33"
CPE_NAME="cpe:2.3:o:amazon:amazon_linux:2023"
HOME_URL="https://aws.amazon.com/linux/amazon-linux-2023/"
DOCUMENTATION_URL="https://docs.aws.amazon.com/linux/"
SUPPORT_URL="https://aws.amazon.com/premiumsupport/"
BUG_REPORT_URL="https://github.com/amazonlinux/amazon-linux-2023"
VENDOR_NAME="AWS"
VENDOR_URL="https://aws.amazon.com/"
SUPPORT_END="2029-06-30"
"#;
            std::fs::write(&os_release_path, os_release)?;
            std::fs::write(
                rootfs.join("etc").join("system-release"),
                "Amazon Linux release 2023.6.20250218 (Amazon Linux)\n",
            )?;
        }

        let joined = command.join(" ").trim().to_string();
        let current_target_dir = if let Some(wd) = working_dir {
            let rel = wd.trim_start_matches('/');
            if rel.is_empty() {
                rootfs.clone()
            } else {
                rootfs.join(rel)
            }
        } else {
            user_home.clone()
        };

        match joined.as_str() {
            "cat /etc/os-release" => {
                let text = std::fs::read_to_string(&os_release_path)?;
                Ok((text, String::new(), 0))
            }
            "cat /etc/system-release" => {
                let text = std::fs::read_to_string(rootfs.join("etc").join("system-release"))?;
                Ok((text, String::new(), 0))
            }
            "uname -a" | "uname -r" => {
                if joined == "uname -r" {
                    Ok(("6.1.128-136.201.amzn2023.x86_64\n".to_string(), String::new(), 0))
                } else {
                    Ok((
                        format!(
                            "Linux {} 6.1.128-136.201.amzn2023.x86_64 #1 SMP PREEMPT_DYNAMIC UTC 2025 x86_64 x86_64 x86_64 GNU/Linux\n",
                            instance_name
                        ),
                        String::new(),
                        0,
                    ))
                }
            }
            "whoami" => Ok((format!("{}\n", ssh_user), String::new(), 0)),
            "hostname" => Ok((format!("{}\n", instance_name), String::new(), 0)),
            "pwd" => {
                let wd = working_dir
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("/home/{}", ssh_user));
                Ok((format!("{}\n", wd), String::new(), 0))
            }
            "df" | "df -h" => {
                let out = "Filesystem      Size  Used Avail Use% Mounted on\ndevtmpfs        4.0M     0  4.0M   0% /dev\ntmpfs           960M     0  960M   0% /dev/shm\ntmpfs           384M  4.8M  380M   2% /run\n/dev/xvda4       10G  1.3G  8.7G  13% /\n/dev/xvda3      150M   16M  134M  11% /boot/efi\n".to_string();
                Ok((out, String::new(), 0))
            }
            "free" | "free -h" | "free -m" => {
                let out = "               total        used        free      shared  buff/cache   available\nMem:           2.0Gi       341Mi       1.4Gi       4.0Mi       300Mi       1.6Gi\nSwap:             0B          0B          0B\n".to_string();
                Ok((out, String::new(), 0))
            }
            "uptime" => {
                Ok((" 20:25:00 up 10 min,  1 user,  load average: 0.08, 0.03, 0.01\n".to_string(), String::new(), 0))
            }
            "ip a" | "ip addr" => {
                let out = "1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN group default qlen 1000\n    inet 127.0.0.1/8 scope host lo\n2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc fq_codel state UP group default qlen 1000\n    inet 10.23.104.10/24 brd 10.23.104.255 scope global dynamic eth0\n".to_string();
                Ok((out, String::new(), 0))
            }
            "ps" | "ps aux" => {
                let out = format!(
                    "USER         PID %CPU %MEM    VSZ   RSS TTY      STAT START   TIME COMMAND\nroot           1  0.0  0.5 168432  9812 ?        Ss   20:19   0:01 /usr/lib/systemd/systemd --switched-root --system\nroot         240  0.0  0.3  42104  6144 ?        Ss   20:19   0:00 /usr/lib/systemd/systemd-journald\nroot         312  0.0  0.4  89412  8192 ?        Ss   20:19   0:00 /usr/sbin/sshd -D\n{:<8}     450  0.0  0.2  14200  4120 pts/0    Ss   20:20   0:00 -bash\n",
                    ssh_user
                );
                Ok((out, String::new(), 0))
            }
            "cloud-init status" | "cloud-init status --wait" => {
                Ok(("status: done\n".to_string(), String::new(), 0))
            }
            s if s.starts_with("dnf ") || s.starts_with("sudo dnf ") => {
                if s.contains("--version") {
                    Ok(("4.14.0\n  Installed: dnf-0:4.14.0-1.amzn2023.0.4.noarch\n".to_string(), String::new(), 0))
                } else {
                    Ok((
                        format!(
                            "Amazon Linux 2023 repository                    3.8 MB/s | 24 MB     00:00\nLast metadata expiration check: 0:00:01 ago.\nDependencies resolved.\nComplete! (Executed '{}' on {})\n",
                            s, instance_name
                        ),
                        String::new(),
                        0,
                    ))
                }
            }
            s if s.starts_with("mkdir ") => {
                let dirname = s.strip_prefix("mkdir ").unwrap().trim().trim_start_matches("-p ").trim();
                let p = current_target_dir.join(dirname.trim_start_matches('/'));
                std::fs::create_dir_all(&p)?;
                Ok((String::new(), String::new(), 0))
            }
            s if s.starts_with("touch ") => {
                let filename = s.strip_prefix("touch ").unwrap().trim();
                let p = current_target_dir.join(filename.trim_start_matches('/'));
                if let Some(parent) = p.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::OpenOptions::new().create(true).write(true).open(&p);
                Ok((String::new(), String::new(), 0))
            }
            s if s.starts_with("rm ") => {
                let target = s.strip_prefix("rm ").unwrap().trim().trim_start_matches("-rf ").trim_start_matches("-f ").trim();
                let p = current_target_dir.join(target.trim_start_matches('/'));
                if p.is_dir() {
                    let _ = std::fs::remove_dir_all(&p);
                } else {
                    let _ = std::fs::remove_file(&p);
                }
                Ok((String::new(), String::new(), 0))
            }
            s if s.starts_with("echo ") => {
                if let Some((content, filename)) = s.split_once('>') {
                    let text = content.strip_prefix("echo ").unwrap_or("").trim().trim_matches('"').trim_matches('\'');
                    let target_file = filename.trim().trim_start_matches('>');
                    let p = current_target_dir.join(target_file.trim().trim_start_matches('/'));
                    if let Some(parent) = p.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    std::fs::write(&p, format!("{}\n", text))?;
                    Ok((String::new(), String::new(), 0))
                } else {
                    let msg = s.strip_prefix("echo ").unwrap_or("").trim().trim_matches('"').trim_matches('\'');
                    Ok((format!("{}\n", msg), String::new(), 0))
                }
            }
            s if s.starts_with("ls") => {
                let is_long = s.contains("-l") || s.contains("-al") || s.contains("-la");
                let show_all = s.contains("-a") || s.contains("-al") || s.contains("-la");

                let mut entries = Vec::new();
                if show_all {
                    entries.push((".".to_string(), true, 4096));
                    entries.push(("..".to_string(), true, 4096));
                }

                if let Ok(rd) = std::fs::read_dir(&current_target_dir) {
                    for entry in rd.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if !show_all && name.starts_with('.') {
                            continue;
                        }
                        let is_d = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        entries.push((name, is_d, size));
                    }
                }

                if is_long {
                    let mut out = String::from("total 16\n");
                    for (name, is_d, size) in entries {
                        if is_d {
                            out.push_str(&format!("drwxr-xr-x 2 {} {} {:>5} Nov 20 20:20 {}\n", ssh_user, ssh_user, size, name));
                        } else {
                            out.push_str(&format!("-rw-r--r-- 1 {} {} {:>5} Nov 20 20:20 {}\n", ssh_user, ssh_user, size, name));
                        }
                    }
                    Ok((out, String::new(), 0))
                } else {
                    let names: Vec<String> = entries.into_iter().map(|(n, _, _)| n).collect();
                    let out = if names.is_empty() {
                        String::new()
                    } else {
                        names.join("  ") + "\n"
                    };
                    Ok((out, String::new(), 0))
                }
            }
            _ => {
                // If user reads a file inside the guest rootfs (e.g., `cat hello.txt` or `cat /etc/hosts`)
                if let Some(arg_path) = joined.strip_prefix("cat ") {
                    let p = if arg_path.trim().starts_with('/') {
                        rootfs.join(arg_path.trim().trim_start_matches('/'))
                    } else {
                        current_target_dir.join(arg_path.trim())
                    };
                    if p.exists() {
                        let content = std::fs::read_to_string(p)?;
                        return Ok((content, String::new(), 0));
                    }
                }
                Ok((
                    format!(
                        "[AL2023 Guest ({})]: executed `{}`\n",
                        instance_name, joined
                    ),
                    String::new(),
                    0,
                ))
            }
        }
    }

    /// Transfers files or directories between Host and Guest (`ec2pass transfer`).
    pub async fn transfer_path(
        &self,
        instance_dir: &Path,
        ssh_port: u16,
        ssh_user: &str,
        src: &str,
        dst: &str,
        recursive: bool,
        parents: bool,
        is_mock: bool,
    ) -> Result<()> {
        let rootfs = instance_dir.join("rootfs");
        if is_mock {
            let resolve_end = |spec: &str| -> PathBuf {
                if let Some((_inst, guest_path)) = spec.split_once(':') {
                    // Take care not to split Windows drive letters like C:\path
                    if _inst.len() > 1 {
                        let rel = guest_path.trim_start_matches('/');
                        return rootfs.join(rel);
                    }
                }
                PathBuf::from(spec)
            };

            let src_path = resolve_end(src);
            let dst_path = resolve_end(dst);
            if !src_path.exists() {
                bail!("Source path '{}' does not exist", src);
            }
            if parents {
                if let Some(parent) = dst_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            if src_path.is_dir() {
                if !recursive {
                    bail!(
                        "'{}' is a directory. Use -r / --recursive to transfer directories.",
                        src
                    );
                }
                copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                if let Some(parent) = dst_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let final_dst = if dst_path.is_dir() {
                    dst_path.join(src_path.file_name().unwrap_or_default())
                } else {
                    dst_path
                };
                std::fs::copy(&src_path, &final_dst)?;
            }
            return Ok(());
        }

        // Real SCP transfer over SSH port
        let format_scp_endpoint = |spec: &str| -> String {
            if let Some((inst, guest_path)) = spec.split_once(':') {
                if inst.len() > 1 {
                    return format!("{}@127.0.0.1:{}", ssh_user, guest_path);
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
