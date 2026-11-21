//! Persistent state database (`state.json`) managing instances, snapshots, settings tree,
//! and host command aliases with atomic file writes.

use anyhow::{bail, Context, Result};
use chrono::Utc;
use mp_protocol::{
    format_bytes, parse_byte_size, CommandAlias, InstanceDetails, InstanceState, MountSpec,
    SnapshotInfo,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceRecord {
    pub details: InstanceDetails,
    pub instance_dir: PathBuf,
    pub disk_path: PathBuf,
    pub seed_iso_path: PathBuf,
    pub base_image_path: PathBuf,
    pub qmp_port: u16,
    pub pid: Option<u32>,
    pub mac_address: String,
    pub cloud_init_yaml: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentState {
    pub settings: BTreeMap<String, String>,
    pub instances: BTreeMap<String, InstanceRecord>,
    pub aliases: BTreeMap<String, CommandAlias>,
    pub next_ssh_port: u16,
    pub next_qmp_port: u16,
}

impl Default for PersistentState {
    fn default() -> Self {
        let mut settings = BTreeMap::new();
        settings.insert("client.primary-name".to_string(), "primary".to_string());
        settings.insert("client.apps.windows-terminal.profiles".to_string(), "primary".to_string());
        settings.insert("local.driver".to_string(), "qemu".to_string());
        settings.insert("local.bridged-network".to_string(), "".to_string());
        settings.insert("local.privileged-mounts".to_string(), "true".to_string());
        settings.insert("local.passphrase".to_string(), "false".to_string());

        Self {
            settings,
            instances: BTreeMap::new(),
            aliases: BTreeMap::new(),
            next_ssh_port: 10022,
            next_qmp_port: 14444,
        }
    }
}

pub struct StateStore {
    state_file: PathBuf,
    pub data: PersistentState,
}

impl StateStore {
    pub fn load_or_create(data_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(data_dir)?;
        let state_file = data_dir.join("multipassd-state.json");
        let data = if state_file.exists() {
            let content = std::fs::read_to_string(&state_file)
                .with_context(|| format!("Failed to read state file {:?}", state_file))?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            PersistentState::default()
        };

        let store = Self { state_file, data };
        store.save()?;
        Ok(store)
    }

    pub fn save(&self) -> Result<()> {
        let tmp = self.state_file.with_extension("json.tmp");
        let content = serde_json::to_string_pretty(&self.data)?;
        std::fs::write(&tmp, content)?;
        std::fs::rename(&tmp, &self.state_file)?;
        Ok(())
    }

    pub fn allocate_ports(&mut self) -> (u16, u16) {
        let ssh = self.data.next_ssh_port;
        let qmp = self.data.next_qmp_port;
        self.data.next_ssh_port += 1;
        self.data.next_qmp_port += 1;
        (ssh, qmp)
    }

    /// Generates a deterministic virtual guest IPv4 in the 10.23.104.x/24 Multipass subnet
    pub fn allocate_guest_ip(&self) -> String {
        let host_id = 10 + (self.data.instances.len() % 230);
        format!("10.23.104.{}", host_id)
    }

    /// Generates a deterministic KVM/QEMU locally administered MAC address (`52:54:00:xx:xx:xx`)
    pub fn allocate_mac(&self) -> String {
        let idx = self.data.instances.len() + 1;
        format!("52:54:00:23:{:02x}:{:02x}", (idx >> 8) & 0xff, idx & 0xff)
    }

    /// Returns a complete flattened view of the hierarchical settings tree, including dynamic
    /// per-instance keys (`local.<instance>.cpus`, `local.<instance>.memory`, `local.<instance>.disk`, `local.<instance>.bridged`).
    pub fn get_all_settings(&self) -> BTreeMap<String, String> {
        let mut map = self.data.settings.clone();
        for (name, rec) in &self.data.instances {
            map.insert(
                format!("local.{}.cpus", name),
                rec.details.cpus.to_string(),
            );
            map.insert(
                format!("local.{}.memory", name),
                format_bytes(rec.details.memory_total_bytes),
            );
            map.insert(
                format!("local.{}.disk", name),
                format_bytes(rec.details.disk_total_bytes),
            );
            map.insert(
                format!("local.{}.bridged", name),
                "false".to_string(),
            );
        }
        map
    }

    /// Updates a setting in the tree. Supports modifying `local.<instance>.(cpus|memory|disk)`
    /// when the instance is `Stopped`, matching Canonical Multipass behavior.
    pub fn set_setting(&mut self, key: &str, value: &str) -> Result<()> {
        let parts: Vec<&str> = key.split('.').collect();
        if parts.len() == 3 && parts[0] == "local" {
            let inst_name = parts[1];
            let prop = parts[2];
            if let Some(rec) = self.data.instances.get_mut(inst_name) {
                if rec.details.state != InstanceState::Stopped {
                    bail!(
                        "Cannot resize instance '{}' while it is {}. Please stop the instance first.",
                        inst_name,
                        rec.details.state
                    );
                }
                match prop {
                    "cpus" => {
                        let cpus: u32 = value
                            .parse()
                            .context("Invalid CPU count: must be a positive integer")?;
                        if cpus == 0 {
                            bail!("CPU count must be at least 1");
                        }
                        rec.details.cpus = cpus;
                    }
                    "memory" => {
                        let bytes = parse_byte_size(value)?;
                        if bytes < 128 * 1024 * 1024 {
                            bail!("Memory size must be at least 128MiB");
                        }
                        rec.details.memory_total_bytes = bytes;
                    }
                    "disk" => {
                        let bytes = parse_byte_size(value)?;
                        if bytes < rec.details.disk_total_bytes {
                            bail!(
                                "Disk size can only be increased (current: {}, requested: {})",
                                format_bytes(rec.details.disk_total_bytes),
                                format_bytes(bytes)
                            );
                        }
                        rec.details.disk_total_bytes = bytes;
                    }
                    "bridged" => {
                        self.data.settings.insert(key.to_string(), value.to_string());
                    }
                    other => bail!("Unrecognized instance property 'local.{}.{}'", inst_name, other),
                }
                self.save()?;
                return Ok(());
            }
        }

        if key == "local.passphrase" {
            // Store whether passphrase is set
            self.data
                .settings
                .insert("local.passphrase".to_string(), "true".to_string());
            self.data
                .settings
                .insert("local._passphrase_secret".to_string(), value.to_string());
            self.save()?;
            return Ok(());
        }

        let valid_global_keys = [
            "client.primary-name",
            "client.apps.windows-terminal.profiles",
            "local.driver",
            "local.bridged-network",
            "local.privileged-mounts",
        ];
        if !valid_global_keys.contains(&key) {
            bail!(
                "Invalid setting key '{}'. Run 'ec2pass get --keys' to inspect available keys.",
                key
            );
        }

        self.data
            .settings
            .insert(key.to_string(), value.to_string());
        self.save()?;
        Ok(())
    }

    pub fn create_snapshot(
        &mut self,
        instance_name: &str,
        snapshot_name: Option<String>,
        comment: Option<String>,
    ) -> Result<SnapshotInfo> {
        let rec = self
            .data
            .instances
            .get_mut(instance_name)
            .with_context(|| format!("No such instance: '{}'", instance_name))?;

        if rec.details.state != InstanceState::Stopped {
            bail!(
                "Snapshots can only be taken for stopped instances ('{}' is currently {}).",
                instance_name,
                rec.details.state
            );
        }

        let next_idx = (rec.details.snapshots.len() as u32) + 1;
        let name = snapshot_name.unwrap_or_else(|| format!("snapshot{}", next_idx));
        if rec.details.snapshots.iter().any(|s| s.name == name) {
            bail!(
                "Snapshot '{}' already exists for instance '{}'",
                name,
                instance_name
            );
        }

        let parent = rec.details.snapshots.last().map(|s| s.name.clone());
        let snap = SnapshotInfo {
            name,
            index: next_idx,
            parent,
            comment: comment.unwrap_or_default(),
            created_at: Utc::now(),
            cpus: rec.details.cpus,
            memory_bytes: rec.details.memory_total_bytes,
            disk_bytes: rec.details.disk_total_bytes,
        };
        rec.details.snapshots.push(snap.clone());
        self.save()?;
        Ok(snap)
    }

    pub fn restore_snapshot(&mut self, instance_name: &str, snapshot_name: &str) -> Result<SnapshotInfo> {
        let rec = self
            .data
            .instances
            .get_mut(instance_name)
            .with_context(|| format!("No such instance: '{}'", instance_name))?;

        if rec.details.state != InstanceState::Stopped {
            bail!(
                "Snapshots can only be restored on stopped instances ('{}' is currently {}).",
                instance_name,
                rec.details.state
            );
        }

        let snap = rec
            .details
            .snapshots
            .iter()
            .find(|s| s.name == snapshot_name)
            .cloned()
            .with_context(|| {
                format!(
                    "Snapshot '{}' does not exist on instance '{}'",
                    snapshot_name, instance_name
                )
            })?;

        rec.details.cpus = snap.cpus;
        rec.details.memory_total_bytes = snap.memory_bytes;
        rec.details.disk_total_bytes = snap.disk_bytes;
        self.save()?;
        Ok(snap)
    }

    pub fn add_mount(
        &mut self,
        instance_name: &str,
        source_path: String,
        target_path: String,
        uid_map: Vec<(i32, i32)>,
        gid_map: Vec<(i32, i32)>,
        mount_type: String,
    ) -> Result<()> {
        let rec = self
            .data
            .instances
            .get_mut(instance_name)
            .with_context(|| format!("No such instance: '{}'", instance_name))?;

        rec.details.mounts.insert(
            target_path.clone(),
            MountSpec {
                source_path,
                target_path,
                uid_map,
                gid_map,
                mount_type,
            },
        );
        self.save()?;
        Ok(())
    }

    pub fn remove_mount(&mut self, instance_name: &str, target_path: Option<&str>) -> Result<()> {
        let rec = self
            .data
            .instances
            .get_mut(instance_name)
            .with_context(|| format!("No such instance: '{}'", instance_name))?;

        match target_path {
            Some(path) => {
                rec.details.mounts.remove(path);
            }
            None => {
                rec.details.mounts.clear();
            }
        }
        self.save()?;
        Ok(())
    }
}
