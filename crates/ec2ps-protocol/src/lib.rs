//! Client-Daemon IPC protocol, RPC messages, instance state models, and
//! output formatters (`table`, `json`, `yaml`, `csv`).

use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Default local TCP port for the `ec2passd` daemon IPC server.
pub const DEFAULT_DAEMON_ADDR: &str = "127.0.0.1:50051";

/// Returns the platform-appropriate default IPC daemon address:
/// - Unix (Linux/macOS): Unix Domain Socket (`unix:/var/run/ec2pass.sock` or `unix:$XDG_RUNTIME_DIR/ec2pass.sock`)
/// - Windows: TCP Loopback (`127.0.0.1:50051`)
pub fn default_daemon_address() -> String {
    if let Ok(custom) = std::env::var("EC2PASS_DAEMON_ADDR")
        .or_else(|_| std::env::var("MP_DAEMON_ADDR"))
        .or_else(|_| std::env::var("ALPASS_DAEMON_ADDR"))
    {
        return custom;
    }
    #[cfg(unix)]
    {
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            return format!("unix:{}/ec2pass.sock", runtime_dir);
        }
        "unix:/var/run/ec2pass.sock".to_string()
    }
    #[cfg(not(unix))]
    {
        DEFAULT_DAEMON_ADDR.to_string()
    }
}

/// Output format supported by `list`, `info`, `find`, `networks`, `aliases`, `version`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
    Csv,
}

impl std::str::FromStr for OutputFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "table" => Ok(Self::Table),
            "json" => Ok(Self::Json),
            "yaml" => Ok(Self::Yaml),
            "csv" => Ok(Self::Csv),
            other => bail!(
                "Invalid format '{}'. Supported formats: table, json, yaml, csv",
                other
            ),
        }
    }
}

/// Lifecycle state of an instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstanceState {
    Running,
    Stopped,
    Deleted,
    Starting,
    Restarting,
    Suspending,
    Suspended,
    DelayedShutdown,
    Unknown,
}

impl InstanceState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::Stopped => "Stopped",
            Self::Deleted => "Deleted",
            Self::Starting => "Starting",
            Self::Restarting => "Restarting",
            Self::Suspending => "Suspending",
            Self::Suspended => "Suspended",
            Self::DelayedShutdown => "Delayed Shutdown",
            Self::Unknown => "Unknown",
        }
    }
}

impl std::fmt::Display for InstanceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Directory mount mapping between host and guest instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountSpec {
    pub source_path: String,
    pub target_path: String,
    /// `uid_mappings` in format `host_uid:guest_uid`
    pub uid_map: Vec<(i32, i32)>,
    /// `gid_mappings` in format `host_gid:guest_gid`
    pub gid_map: Vec<(i32, i32)>,
    pub mount_type: String,
}

/// Snapshot metadata for an instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotInfo {
    pub name: String,
    pub index: u32,
    pub parent: Option<String>,
    pub comment: String,
    pub created_at: DateTime<Utc>,
    pub cpus: u32,
    pub memory_bytes: u64,
    pub disk_bytes: u64,
}

/// Host command alias mapped to an instance command (`ec2pass alias`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandAlias {
    pub alias_name: String,
    pub instance: String,
    pub command: String,
    pub working_directory: String,
}

/// Detailed instance metadata returned by `info` and `list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceDetails {
    pub name: String,
    pub state: InstanceState,
    pub ipv4: Vec<String>,
    pub release: String,
    pub image_hash: String,
    pub image_release: String,
    pub cpus: u32,
    pub load: Vec<f64>,
    pub memory_usage_bytes: u64,
    pub memory_total_bytes: u64,
    pub disk_usage_bytes: u64,
    pub disk_total_bytes: u64,
    pub mounts: BTreeMap<String, MountSpec>,
    pub ssh_port: u16,
    pub ssh_user: String,
    pub ssh_key_path: String,
    pub snapshots: Vec<SnapshotInfo>,
    pub created_at: DateTime<Utc>,
}

/// Catalog entry for an Amazon Linux 2023 image returned by `find`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageCatalogEntry {
    pub os: String,
    pub release: String,
    pub remote: String,
    pub version: String,
    pub aliases: Vec<String>,
    pub arch: String,
    pub download_url: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub description: String,
}

/// Host network interface entry returned by `networks`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterfaceInfo {
    pub name: String,
    pub net_type: String,
    pub description: String,
}

/// Client-to-Daemon RPC request message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RpcRequest {
    Ping,
    Version,
    Authenticate {
        passphrase: String,
    },
    Launch {
        image: Option<String>,
        name: Option<String>,
        cpus: u32,
        memory_bytes: u64,
        disk_bytes: u64,
        cloud_init_yaml: Option<String>,
        networks: Vec<String>,
        mounts: Vec<(String, String)>,
        timeout_secs: u64,
    },
    Start {
        names: Vec<String>,
        all: bool,
    },
    Stop {
        names: Vec<String>,
        all: bool,
        force: bool,
        delay_mins: Option<u32>,
        cancel_shutdown: bool,
    },
    Restart {
        names: Vec<String>,
        all: bool,
    },
    Suspend {
        names: Vec<String>,
        all: bool,
    },
    Delete {
        names: Vec<String>,
        all: bool,
        purge: bool,
    },
    Recover {
        names: Vec<String>,
        all: bool,
    },
    Purge,
    List {
        snapshots: bool,
    },
    Info {
        names: Vec<String>,
        all: bool,
        snapshots: bool,
    },
    Find {
        search_string: Option<String>,
        show_unsupported: bool,
        force_update: bool,
    },
    Exec {
        name: String,
        command: Vec<String>,
        working_dir: Option<String>,
    },
    ShellInfo {
        name: Option<String>,
    },
    Mount {
        source_path: String,
        instance: String,
        target_path: String,
        uid_map: Vec<(i32, i32)>,
        gid_map: Vec<(i32, i32)>,
        mount_type: String,
    },
    Umount {
        instance: String,
        target_path: Option<String>,
    },
    Transfer {
        sources: Vec<String>,
        destination: String,
        recursive: bool,
        parents: bool,
    },
    Snapshot {
        instance: String,
        snapshot_name: Option<String>,
        comment: Option<String>,
    },
    Restore {
        instance: String,
        snapshot_name: String,
        destructive: bool,
    },
    Clone {
        source_name: String,
        destination_name: Option<String>,
    },
    Get {
        key: Option<String>,
        keys_only: bool,
    },
    Set {
        key: String,
        value: String,
    },
    Alias {
        instance: String,
        command: String,
        alias_name: Option<String>,
        working_directory: String,
    },
    Unalias {
        alias_names: Vec<String>,
        all: bool,
    },
    Aliases,
    Networks,
}

/// Daemon-to-Client streamed frame (allows real-time progress events followed by final response).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RpcStreamFrame {
    Progress {
        phase: String,
        message: String,
        percent: Option<u8>,
    },
    Response(RpcResponse),
}

/// Final Daemon-to-Client RPC response payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RpcResponse {
    Pong,
    Version {
        client_version: String,
        daemon_version: String,
        driver: String,
        default_image: String,
    },
    OkMessage(String),
    Launched(InstanceDetails),
    InstanceList(Vec<InstanceDetails>),
    CatalogList(Vec<ImageCatalogEntry>),
    ExecResult {
        stdout: String,
        stderr: String,
        exit_code: i32,
    },
    ShellConnection {
        instance: InstanceDetails,
        ssh_host: String,
        ssh_port: u16,
        ssh_user: String,
        ssh_key_path: String,
        is_mock: bool,
    },
    SnapshotCreated(SnapshotInfo),
    SettingsMap(BTreeMap<String, String>),
    AliasList(Vec<CommandAlias>),
    NetworkList(Vec<NetworkInterfaceInfo>),
    Error(String),
}

// ============================================================================
// Human-readable Size Parser & Formatter (`512M`, `2G`, `20GiB`, etc.)
// ============================================================================

/// Parses human-readable byte strings like `1G`, `1024M`, `20GiB`, `512MiB` into raw bytes.
pub fn parse_byte_size(input: &str) -> Result<u64> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        bail!("Byte size string cannot be empty");
    }

    let split_idx = trimmed
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(trimmed.len());
    let (num_str, unit_str) = trimmed.split_at(split_idx);
    let value: f64 = num_str
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid numeric size value: '{}'", input))?;

    let multiplier: f64 = match unit_str.trim().to_lowercase().as_str() {
        "" | "b" => 1.0,
        "k" | "kb" | "kib" => 1024.0,
        "m" | "mb" | "mib" => 1024.0 * 1024.0,
        "g" | "gb" | "gib" => 1024.0 * 1024.0 * 1024.0,
        "t" | "tb" | "tib" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        other => bail!("Unsupported size unit '{}' in '{}'", other, input),
    };

    Ok((value * multiplier) as u64)
}

/// Formats raw byte count into human-readable string (`1.2GiB`, `412.5MiB`).
pub fn format_bytes(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;

    let b = bytes as f64;
    if b >= GIB {
        format!("{:.1}GiB", b / GIB)
    } else if b >= MIB {
        format!("{:.1}MiB", b / MIB)
    } else if b >= KIB {
        format!("{:.1}KiB", b / KIB)
    } else {
        format!("{}B", bytes)
    }
}

// ============================================================================
// Output Formatters (Table / JSON / YAML / CSV)
// ============================================================================

pub fn format_instance_list(
    instances: &[InstanceDetails],
    snapshots: bool,
    format: OutputFormat,
) -> Result<String> {
    if snapshots {
        return format_snapshot_list(instances, format);
    }

    match format {
        OutputFormat::Table => {
            if instances.is_empty() {
                return Ok("No instances found.\n".to_string());
            }
            let headers = ["Name", "State", "IPv4", "Image"];
            let mut rows = Vec::with_capacity(instances.len());
            for inst in instances {
                let ip = if inst.ipv4.is_empty() || inst.state != InstanceState::Running {
                    "--".to_string()
                } else {
                    inst.ipv4.join(", ")
                };
                rows.push(vec![
                    inst.name.clone(),
                    inst.state.to_string(),
                    ip,
                    inst.release.clone(),
                ]);
            }
            Ok(render_ascii_table(&headers, &rows))
        }
        OutputFormat::Csv => {
            let mut out = String::from("Name,State,IPv4,AutoIPv4,Release\n");
            for inst in instances {
                let ip = inst.ipv4.first().cloned().unwrap_or_default();
                out.push_str(&format!(
                    "{},{},{},,{}\n",
                    inst.name, inst.state, ip, inst.release
                ));
            }
            Ok(out)
        }
        OutputFormat::Json => {
            #[derive(Serialize)]
            struct JsonListEntry<'a> {
                name: &'a str,
                state: &'a str,
                ipv4: &'a [String],
                release: &'a str,
            }
            #[derive(Serialize)]
            struct JsonList<'a> {
                list: Vec<JsonListEntry<'a>>,
            }
            let list = instances
                .iter()
                .map(|i| JsonListEntry {
                    name: &i.name,
                    state: i.state.as_str(),
                    ipv4: &i.ipv4,
                    release: &i.release,
                })
                .collect();
            Ok(serde_json::to_string_pretty(&JsonList { list })? + "\n")
        }
        OutputFormat::Yaml => {
            let mut map = BTreeMap::new();
            for inst in instances {
                map.insert(inst.name.clone(), inst.clone());
            }
            Ok(serde_yaml::to_string(&map)?)
        }
    }
}

fn format_snapshot_list(instances: &[InstanceDetails], format: OutputFormat) -> Result<String> {
    let mut rows = Vec::new();
    for inst in instances {
        for snap in &inst.snapshots {
            rows.push(vec![
                inst.name.clone(),
                snap.name.clone(),
                snap.parent.clone().unwrap_or_else(|| "--".to_string()),
                if snap.comment.is_empty() {
                    "--".to_string()
                } else {
                    snap.comment.clone()
                },
            ]);
        }
    }

    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                return Ok("No snapshots found.\n".to_string());
            }
            Ok(render_ascii_table(
                &["Instance", "Snapshot", "Parent", "Comment"],
                &rows,
            ))
        }
        OutputFormat::Csv => {
            let mut out = String::from("Instance,Snapshot,Parent,Comment\n");
            for r in rows {
                out.push_str(&format!("{},{},{},{}\n", r[0], r[1], r[2], r[3]));
            }
            Ok(out)
        }
        OutputFormat::Json => Ok(serde_json::to_string_pretty(&rows)? + "\n"),
        OutputFormat::Yaml => Ok(serde_yaml::to_string(&rows)?),
    }
}

pub fn format_instance_info(
    instances: &[InstanceDetails],
    format: OutputFormat,
) -> Result<String> {
    match format {
        OutputFormat::Table => {
            let mut blocks = Vec::with_capacity(instances.len());
            for inst in instances {
                let mut b = String::new();
                b.push_str(&format!("{:<16}{}\n", "Name:", inst.name));
                b.push_str(&format!("{:<16}{}\n", "State:", inst.state));
                b.push_str(&format!("{:<16}{}\n", "Snapshots:", inst.snapshots.len()));
                let ipv4 = if inst.ipv4.is_empty() || inst.state != InstanceState::Running {
                    "--".to_string()
                } else {
                    inst.ipv4.join("\n                ")
                };
                b.push_str(&format!("{:<16}{}\n", "IPv4:", ipv4));
                b.push_str(&format!("{:<16}{}\n", "Release:", inst.release));
                b.push_str(&format!(
                    "{:<16}{} ({})\n",
                    "Image hash:",
                    &inst.image_hash[..12.min(inst.image_hash.len())],
                    inst.image_release
                ));
                b.push_str(&format!("{:<16}{}\n", "CPU(s):", inst.cpus));
                if inst.state == InstanceState::Running && !inst.load.is_empty() {
                    let loads: Vec<String> =
                        inst.load.iter().map(|l| format!("{:.2}", l)).collect();
                    b.push_str(&format!("{:<16}{}\n", "Load:", loads.join(" ")));
                } else {
                    b.push_str(&format!("{:<16}--\n", "Load:"));
                }
                b.push_str(&format!(
                    "{:<16}{} out of {}\n",
                    "Disk usage:",
                    format_bytes(inst.disk_usage_bytes),
                    format_bytes(inst.disk_total_bytes)
                ));
                b.push_str(&format!(
                    "{:<16}{} out of {}\n",
                    "Memory usage:",
                    format_bytes(inst.memory_usage_bytes),
                    format_bytes(inst.memory_total_bytes)
                ));
                if inst.mounts.is_empty() {
                    b.push_str(&format!("{:<16}--\n", "Mounts:"));
                } else {
                    let mut first = true;
                    for (target, m) in &inst.mounts {
                        if first {
                            b.push_str(&format!(
                                "{:<16}{} => {}\n",
                                "Mounts:", m.source_path, target
                            ));
                            first = false;
                        } else {
                            b.push_str(&format!(
                                "{:<16}{} => {}\n",
                                "", m.source_path, target
                            ));
                        }
                    }
                }
                blocks.push(b);
            }
            Ok(blocks.join("\n"))
        }
        OutputFormat::Json => {
            let mut info_map = BTreeMap::new();
            for inst in instances {
                info_map.insert(inst.name.clone(), inst.clone());
            }
            #[derive(Serialize)]
            struct Wrapper {
                errors: Vec<String>,
                info: BTreeMap<String, InstanceDetails>,
            }
            Ok(serde_json::to_string_pretty(&Wrapper {
                errors: vec![],
                info: info_map,
            })? + "\n")
        }
        OutputFormat::Yaml => {
            let mut info_map = BTreeMap::new();
            for inst in instances {
                info_map.insert(inst.name.clone(), inst.clone());
            }
            Ok(serde_yaml::to_string(&info_map)?)
        }
        OutputFormat::Csv => {
            let mut out = String::from(
                "Name,State,Ipv4,Release,Image hash,CPU(s),Disk usage,Disk total,Memory usage,Memory total\n",
            );
            for inst in instances {
                out.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{},{}\n",
                    inst.name,
                    inst.state,
                    inst.ipv4.join(";"),
                    inst.release,
                    inst.image_hash,
                    inst.cpus,
                    inst.disk_usage_bytes,
                    inst.disk_total_bytes,
                    inst.memory_usage_bytes,
                    inst.memory_total_bytes
                ));
            }
            Ok(out)
        }
    }
}

pub fn format_catalog_find(
    entries: &[ImageCatalogEntry],
    format: OutputFormat,
) -> Result<String> {
    match format {
        OutputFormat::Table => {
            let headers = ["Image", "Aliases", "Version", "Description"];
            let mut rows = Vec::with_capacity(entries.len());
            for e in entries {
                let primary_alias = e
                    .aliases
                    .first()
                    .cloned()
                    .unwrap_or_else(|| e.release.clone());
                let other_aliases = if e.aliases.len() > 1 {
                    e.aliases[1..].join(",")
                } else {
                    "--".to_string()
                };
                rows.push(vec![
                    primary_alias,
                    other_aliases,
                    e.version.clone(),
                    e.description.clone(),
                ]);
            }
            Ok(render_ascii_table(&headers, &rows))
        }
        OutputFormat::Json => {
            let mut images = BTreeMap::new();
            for e in entries {
                let key = e
                    .aliases
                    .first()
                    .cloned()
                    .unwrap_or_else(|| e.release.clone());
                images.insert(key, e.clone());
            }
            #[derive(Serialize)]
            struct FindJson {
                images: BTreeMap<String, ImageCatalogEntry>,
            }
            Ok(serde_json::to_string_pretty(&FindJson { images })? + "\n")
        }
        OutputFormat::Yaml => Ok(serde_yaml::to_string(entries)?),
        OutputFormat::Csv => {
            let mut out = String::from("Image,Remote,Aliases,OS,Release,Version,Arch\n");
            for e in entries {
                let primary = e.aliases.first().cloned().unwrap_or_default();
                out.push_str(&format!(
                    "{},{},{},{},{},{},{}\n",
                    primary,
                    e.remote,
                    e.aliases.join(";"),
                    e.os,
                    e.release,
                    e.version,
                    e.arch
                ));
            }
            Ok(out)
        }
    }
}

pub fn render_ascii_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() && cell.len() > widths[i] {
                widths[i] = cell.len();
            }
        }
    }

    let mut out = String::new();
    // Header line
    for (i, h) in headers.iter().enumerate() {
        if i + 1 == headers.len() {
            out.push_str(h);
        } else {
            out.push_str(&format!("{:<width$}   ", h, width = widths[i]));
        }
    }
    out.push('\n');

    // Data lines
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i + 1 == headers.len() {
                out.push_str(cell);
            } else {
                out.push_str(&format!("{:<width$}   ", cell, width = widths[i]));
            }
        }
        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_byte_size() {
        assert_eq!(parse_byte_size("1024").unwrap(), 1024);
        assert_eq!(parse_byte_size("512M").unwrap(), 512 * 1024 * 1024);
        assert_eq!(parse_byte_size("4G").unwrap(), 4 * 1024 * 1024 * 1024);
        assert_eq!(parse_byte_size("20GiB").unwrap(), 20 * 1024 * 1024 * 1024);
    }
}
