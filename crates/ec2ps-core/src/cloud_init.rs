//! Amazon Linux 2023 NoCloud `cloud-init` metadata, vendor-data, and user-data generator.
//!
//! ec2pass injects a `cidata` ISO (`seed.iso`) into every VM containing:
//! 1. `meta-data`: instance-id and local-hostname.
//! 2. `vendor-data`: daemon-managed SSH public key (`ec2-user` + `ubuntu` alias), passwordless sudo,
//!    `growpart` / `xfs_growfs` for root disk resizing, and SELinux context restoration.
//! 3. `user-data`: user-supplied `--cloud-init` YAML configuration (validated prior to boot).

use anyhow::{bail, Context, Result};
use ec2ps_iso9660::{write_cidata_iso, IsoFile};
use std::path::Path;

pub struct CloudInitConfig<'a> {
    pub instance_id: &'a str,
    pub hostname: &'a str,
    pub default_user: &'a str,
    pub ssh_public_key: &'a str,
    pub user_data_yaml: Option<&'a str>,
}

/// How cloud-init will interpret a user-data payload, decided by its first line
/// (mirrors EC2 user-data handling).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserDataKind {
    /// Starts with `#cloud-config`: YAML mapping, validated before boot.
    CloudConfig,
    /// Shell script, MIME multipart, `#include`, boothook, jinja template, part-handler,
    /// cloud-config-archive: handed to cloud-init unchanged.
    Passthrough,
    /// No recognized header: treated as cloud-config YAML for backward compatibility.
    Headerless,
}

const PASSTHROUGH_HEADERS: &[&str] = &[
    "#!",
    "#include",
    "#cloud-boothook",
    "#part-handler",
    "#cloud-config-archive",
    "## template:",
];

/// Removes a leading UTF-8 byte-order mark (added by some Windows editors), which would
/// otherwise hide the format header from cloud-init.
fn strip_bom(s: &str) -> &str {
    s.strip_prefix('\u{feff}').unwrap_or(s)
}

/// Classifies user-data by its first non-blank line.
pub fn classify_user_data(user_data: &str) -> UserDataKind {
    let head = strip_bom(user_data).trim_start();
    if PASSTHROUGH_HEADERS.iter().any(|h| head.starts_with(h))
        || head.to_ascii_lowercase().starts_with("content-type:")
        || head.to_ascii_lowercase().starts_with("mime-version:")
    {
        UserDataKind::Passthrough
    } else if head.starts_with("#cloud-config") {
        UserDataKind::CloudConfig
    } else {
        UserDataKind::Headerless
    }
}

/// Validates user-supplied `--cloud-init` user-data before launching the instance.
/// Cloud-config (and headerless) payloads must be a YAML mapping; other EC2 user-data
/// formats (shell scripts, MIME multipart, ...) are passed through without validation.
pub fn validate_cloud_init_yaml(yaml_str: &str) -> Result<()> {
    let trimmed = strip_bom(yaml_str).trim();
    if trimmed.is_empty() || classify_user_data(trimmed) == UserDataKind::Passthrough {
        return Ok(());
    }
    let val: serde_yaml::Value = serde_yaml::from_str(trimmed)
        .context("Invalid cloud-init YAML syntax provided to --cloud-init")?;
    if !val.is_mapping() && !val.is_null() {
        bail!("cloud-init configuration must be a valid YAML mapping (dictionary)");
    }
    Ok(())
}

/// Builds the `seed.iso` (`cidata` volume label) for an Amazon Linux 2023 instance.
pub fn generate_al2023_seed_iso(output_iso_path: &Path, cfg: &CloudInitConfig<'_>) -> Result<()> {
    // 1. meta-data
    let meta_data = format!(
        "instance-id: {}\nlocal-hostname: {}\n",
        cfg.instance_id, cfg.hostname
    );

    // 2. vendor-data tailored for multi-distro cloud-init (Amazon Linux, Rocky, Fedora, Ubuntu, Debian)
    let vendor_data = format!(
        r#"#cloud-config
growpart:
  mode: auto
  devices: ['/']
  ignore_growroot_disabled: false
resize_rootfs: true
users:
  - default
  - name: {user}
    gecos: ec2pass Cloud User
    sudo: ALL=(ALL) NOPASSWD:ALL
    groups: [wheel, sudo, adm, systemd-journal]
    shell: /bin/bash
    lock_passwd: true
    ssh_authorized_keys:
      - {pub_key}
ssh_pwauth: false
package_update: false
runcmd:
  - [ sh, -c, "restorecon -R /home/{user}/.ssh 2>/dev/null || true" ]
"#,
        user = cfg.default_user,
        pub_key = cfg.ssh_public_key.trim()
    );

    // 3. user-data (either user's custom cloud-config or a default header that also authorizes the SSH key)
    let user_data = match cfg.user_data_yaml {
        Some(custom) if !custom.trim().is_empty() => {
            validate_cloud_init_yaml(custom)?;
            // cloud-init only recognizes the format header on the very first line.
            let body = strip_bom(custom).trim_start();
            match classify_user_data(body) {
                UserDataKind::Passthrough => body.to_string(),
                UserDataKind::CloudConfig => format!("{}\n", body.trim_end()),
                UserDataKind::Headerless => format!("#cloud-config\n{}\n", body.trim_end()),
            }
        }
        _ => format!(
            r#"#cloud-config
ssh_authorized_keys:
  - {}
"#,
            cfg.ssh_public_key.trim()
        ),
    };

    let files = vec![
        IsoFile::new("meta-data", meta_data.into_bytes()),
        IsoFile::new("vendor-data", vendor_data.into_bytes()),
        IsoFile::new("user-data", user_data.into_bytes()),
    ];

    write_cidata_iso(output_iso_path, "cidata", &files).with_context(|| {
        format!(
            "Failed to write cloud-init seed ISO at {:?}",
            output_iso_path
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_cloud_init_yaml() {
        let valid = r#"#cloud-config
packages:
  - git
  - htop
runcmd:
  - echo "Hello AL2023"
"#;
        assert!(validate_cloud_init_yaml(valid).is_ok());

        let invalid = "packages: [unclosed_list";
        assert!(validate_cloud_init_yaml(invalid).is_err());
    }

    #[test]
    fn test_classify_user_data() {
        assert_eq!(
            classify_user_data("#cloud-config\npackages: [git]"),
            UserDataKind::CloudConfig
        );
        assert_eq!(
            classify_user_data("\n#!/bin/bash\necho hi"),
            UserDataKind::Passthrough
        );
        assert_eq!(
            classify_user_data("Content-Type: multipart/mixed; boundary=\"//\""),
            UserDataKind::Passthrough
        );
        assert_eq!(
            classify_user_data("#include\nhttps://example.com/a"),
            UserDataKind::Passthrough
        );
        assert_eq!(
            classify_user_data("#cloud-boothook\n#!/bin/sh"),
            UserDataKind::Passthrough
        );
        assert_eq!(
            classify_user_data("## template: jinja\n#cloud-config"),
            UserDataKind::Passthrough
        );
        assert_eq!(
            classify_user_data("#cloud-config-archive\n- type: text/x-shellscript"),
            UserDataKind::Passthrough
        );
        assert_eq!(
            classify_user_data("packages: [git]"),
            UserDataKind::Headerless
        );
    }

    #[test]
    fn test_shell_and_mime_user_data_accepted() {
        let script = "#!/bin/bash\ndnf install -y nginx\nsystemctl enable --now nginx\n";
        assert!(validate_cloud_init_yaml(script).is_ok());
        let mime = "Content-Type: multipart/mixed; boundary=\"//\"\nMIME-Version: 1.0\n\n--//\n";
        assert!(validate_cloud_init_yaml(mime).is_ok());
        // A UTF-8 BOM from Windows editors must not hide the header.
        assert_eq!(
            classify_user_data("\u{feff}#!/bin/bash\necho hi"),
            UserDataKind::Passthrough
        );
        assert!(validate_cloud_init_yaml("\u{feff}#cloud-config\npackages: [git]").is_ok());
        // Headerless non-mapping text is still rejected.
        assert!(validate_cloud_init_yaml("just some text").is_err());
    }

    #[test]
    fn test_shell_script_user_data_written_verbatim() {
        let tmp_iso = std::env::temp_dir().join(format!("seed_sh_{}.iso", rand::random::<u32>()));
        let script = "\n#!/bin/bash\necho EC2PASS_MARKER > /tmp/marker\n";
        let cfg = CloudInitConfig {
            instance_id: "i-test",
            hostname: "test-host",
            default_user: crate::defaults::DEFAULT_SSH_USER,
            ssh_public_key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAITestKey",
            user_data_yaml: Some(script),
        };
        generate_al2023_seed_iso(&tmp_iso, &cfg).unwrap();
        let iso_str = String::from_utf8_lossy(&std::fs::read(&tmp_iso).unwrap()).to_string();
        assert!(iso_str.contains("#!/bin/bash\necho EC2PASS_MARKER"));
        assert!(!iso_str.contains("#cloud-config\n#!/bin/bash"));
        let _ = std::fs::remove_file(&tmp_iso);
    }

    #[test]
    fn test_cloud_init_vendor_data_has_multi_distro_groups() {
        let tmp_iso = std::env::temp_dir().join(format!("seed_test_{}.iso", rand::random::<u32>()));
        let cfg = CloudInitConfig {
            instance_id: "i-test",
            hostname: "test-host",
            default_user: crate::defaults::DEFAULT_SSH_USER,
            ssh_public_key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAITestKey",
            user_data_yaml: None,
        };
        assert!(generate_al2023_seed_iso(&tmp_iso, &cfg).is_ok());

        let iso_bytes = std::fs::read(&tmp_iso).unwrap();
        let iso_str = String::from_utf8_lossy(&iso_bytes);
        assert!(iso_str.contains("wheel"));
        assert!(iso_str.contains("sudo"));

        let _ = std::fs::remove_file(&tmp_iso);
    }
}
