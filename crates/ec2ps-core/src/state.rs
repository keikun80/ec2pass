//! Persistent state database (`state.json`) managing instances, snapshots, settings tree,
//! and host command aliases with atomic file writes.

use anyhow::{bail, Context, Result};
use chrono::Utc;
use ec2ps_protocol::{
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
    #[serde(default)]
    pub networks: Vec<String>,
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
        settings.insert("local.driver".to_string(), "qemu".to_string());
        settings.insert("local.bridged-network".to_string(), "".to_string());
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
        let state_file = data_dir.join("state.json");
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

    /// Returns the guest IPv4 for QEMU user-mode SLIRP networking
    pub fn allocate_guest_ip(&self) -> String {
        "10.0.2.15".to_string()
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
            map.insert(format!("local.{}.cpus", name), rec.details.cpus.to_string());
            map.insert(
                format!("local.{}.memory", name),
                format_bytes(rec.details.memory_total_bytes),
            );
            map.insert(
                format!("local.{}.disk", name),
                format_bytes(rec.details.disk_total_bytes),
            );
            map.insert(format!("local.{}.bridged", name), "false".to_string());
        }
        map
    }

    /// Updates a setting in the tree. Supports modifying `local.<instance>.(cpus|memory|disk)`
    /// when the instance is `Stopped`.
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
                        self.data
                            .settings
                            .insert(key.to_string(), value.to_string());
                    }
                    other => bail!(
                        "Unrecognized instance property 'local.{}.{}'",
                        inst_name,
                        other
                    ),
                }
                self.save()?;
                return Ok(());
            }
        }

        if key == "local.passphrase" {
            if value.is_empty() || value == "false" {
                self.data
                    .settings
                    .insert("local.passphrase".to_string(), "false".to_string());
                self.data.settings.remove("local._passphrase_secret");
                self.save()?;
                return Ok(());
            }
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
            "local.driver",
            "local.bridged-network",
            "local.passphrase",
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

        if rec.details.state != InstanceState::Stopped
            && rec.details.state != InstanceState::Running
        {
            bail!(
                "Snapshots can only be taken for stopped or running instances ('{}' is currently {}).",
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

    pub fn restore_snapshot(
        &mut self,
        instance_name: &str,
        snapshot_name: &str,
        destructive: bool,
    ) -> Result<SnapshotInfo> {
        let rec = self
            .data
            .instances
            .get_mut(instance_name)
            .with_context(|| format!("No such instance: '{}'", instance_name))?;

        if rec.details.state != InstanceState::Stopped
            && rec.details.state != InstanceState::Running
        {
            bail!(
                "Snapshots can only be restored on stopped or running instances ('{}' is currently {}).",
                instance_name,
                rec.details.state
            );
        }

        let snap_idx = rec
            .details
            .snapshots
            .iter()
            .position(|s| s.name == snapshot_name)
            .with_context(|| {
                format!(
                    "Snapshot '{}' does not exist on instance '{}'",
                    snapshot_name, instance_name
                )
            })?;

        let snap = rec.details.snapshots[snap_idx].clone();

        rec.details.cpus = snap.cpus;
        rec.details.memory_total_bytes = snap.memory_bytes;
        rec.details.disk_total_bytes = snap.disk_bytes;

        if destructive {
            // Prune snapshots taken after the restored snapshot (truncate history tree)
            rec.details.snapshots.truncate(snap_idx + 1);
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_store_allocations() {
        let tmp_dir =
            std::env::temp_dir().join(format!("ec2ps_test_state_{}", rand::random::<u32>()));
        let mut store = StateStore::load_or_create(&tmp_dir).unwrap();

        let (ssh1, qmp1) = store.allocate_ports();
        let (ssh2, qmp2) = store.allocate_ports();
        assert_eq!(ssh2, ssh1 + 1);
        assert_eq!(qmp2, qmp1 + 1);

        let ip = store.allocate_guest_ip();
        assert_eq!(ip, "10.0.2.15");

        let mac = store.allocate_mac();
        assert!(mac.starts_with("52:54:00:23:"));

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    fn dummy_instance_record(name: &str, dir: &Path, state: InstanceState, disk_bytes: u64) -> InstanceRecord {
        let details = InstanceDetails {
            name: name.to_string(),
            state,
            ipv4: vec!["10.0.2.15".to_string()],
            release: "al2023".to_string(),
            image_hash: "dummy_hash".to_string(),
            image_release: "2023.0".to_string(),
            cpus: 1,
            load: vec![0.0, 0.0, 0.0],
            memory_usage_bytes: 0,
            memory_total_bytes: 1024 * 1024 * 1024,
            disk_usage_bytes: 0,
            disk_total_bytes: disk_bytes,
            mounts: BTreeMap::new(),
            ssh_port: 10022,
            ssh_user: "ec2-user".to_string(),
            ssh_key_path: "/dummy/key".to_string(),
            snapshots: Vec::new(),
            created_at: Utc::now(),
        };
        InstanceRecord {
            details,
            instance_dir: dir.join(name),
            disk_path: dir.join(name).join("disk.qcow2"),
            seed_iso_path: dir.join(name).join("seed.iso"),
            base_image_path: dir.join("base.qcow2"),
            qmp_port: 14444,
            pid: None,
            mac_address: "52:54:00:12:34:56".to_string(),
            cloud_init_yaml: None,
            networks: Vec::new(),
        }
    }

    #[test]
    fn test_state_store_disk_resize_setting() {
        let tmp_dir =
            std::env::temp_dir().join(format!("ec2ps_test_state_disk_{}", rand::random::<u32>()));
        let mut store = StateStore::load_or_create(&tmp_dir).unwrap();

        let inst_name = "test-vm";
        let rec = dummy_instance_record(inst_name, &tmp_dir, InstanceState::Stopped, 20 * 1024 * 1024 * 1024);
        store.data.instances.insert(inst_name.to_string(), rec);
        store.save().unwrap();

        // Valid increase: 20G -> 30G
        store.set_setting("local.test-vm.disk", "30G").unwrap();
        assert_eq!(
            store.data.instances[inst_name].details.disk_total_bytes,
            30 * 1024 * 1024 * 1024
        );

        // Decrease: 30G -> 25G should fail
        let res = store.set_setting("local.test-vm.disk", "25G");
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("can only be increased"));

        // When running: resizing should fail
        store.data.instances.get_mut(inst_name).unwrap().details.state = InstanceState::Running;
        let res_running = store.set_setting("local.test-vm.disk", "40G");
        assert!(res_running.is_err());
        assert!(res_running.unwrap_err().to_string().contains("Please stop the instance first"));

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_state_store_mount_management() {
        let tmp_dir =
            std::env::temp_dir().join(format!("ec2ps_test_state_mount_{}", rand::random::<u32>()));
        let mut store = StateStore::load_or_create(&tmp_dir).unwrap();

        let inst_name = "mount-vm";
        let rec = dummy_instance_record(inst_name, &tmp_dir, InstanceState::Stopped, 20 * 1024 * 1024 * 1024);
        store.data.instances.insert(inst_name.to_string(), rec);

        // Add 2 mounts
        store.add_mount(inst_name, "/host/dir1".to_string(), "/guest/mnt1".to_string(), Vec::new(), Vec::new(), "classic".to_string()).unwrap();
        store.add_mount(inst_name, "/host/dir2".to_string(), "/guest/mnt2".to_string(), Vec::new(), Vec::new(), "classic".to_string()).unwrap();
        assert_eq!(store.data.instances[inst_name].details.mounts.len(), 2);

        // Remove single mount
        store.remove_mount(inst_name, Some("/guest/mnt1")).unwrap();
        assert_eq!(store.data.instances[inst_name].details.mounts.len(), 1);
        assert!(!store.data.instances[inst_name].details.mounts.contains_key("/guest/mnt1"));
        assert!(store.data.instances[inst_name].details.mounts.contains_key("/guest/mnt2"));

        // Remove all mounts
        store.remove_mount(inst_name, None).unwrap();
        assert_eq!(store.data.instances[inst_name].details.mounts.len(), 0);

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_valid_and_invalid_settings() {
        let tmp_dir =
            std::env::temp_dir().join(format!("ec2ps_test_state_settings_{}", rand::random::<u32>()));
        let mut store = StateStore::load_or_create(&tmp_dir).unwrap();

        // Valid key
        assert!(store.set_setting("client.primary-name", "my-primary").is_ok());
        assert_eq!(store.data.settings.get("client.primary-name").unwrap(), "my-primary");

        // Passphrase
        assert!(store.set_setting("local.passphrase", "secret123").is_ok());
        assert_eq!(store.data.settings.get("local.passphrase").unwrap(), "true");
        assert_eq!(store.data.settings.get("local._passphrase_secret").unwrap(), "secret123");

        // Passphrase reset
        assert!(store.set_setting("local.passphrase", "").is_ok());
        assert_eq!(store.data.settings.get("local.passphrase").unwrap(), "false");
        assert!(!store.data.settings.contains_key("local._passphrase_secret"));

        // Invalid phantom keys should be rejected
        assert!(store.set_setting("client.apps.windows-terminal.profiles", "foo").is_err());
        assert!(store.set_setting("local.privileged-mounts", "true").is_err());
        assert!(store.set_setting("unknown.key", "val").is_err());

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_snapshot_create_and_restore_destructive() {
        let tmp_dir = std::env::temp_dir()
            .join(format!("ec2ps_test_state_snap_{}", rand::random::<u32>()));
        let mut store = StateStore::load_or_create(&tmp_dir).unwrap();

        let inst_name = "snap-vm";
        let rec = dummy_instance_record(
            inst_name,
            &tmp_dir,
            InstanceState::Stopped,
            20 * 1024 * 1024 * 1024,
        );
        store.data.instances.insert(inst_name.to_string(), rec);

        let snap1 = store
            .create_snapshot(inst_name, None, Some("snap 1".to_string()))
            .unwrap();
        let snap2 = store
            .create_snapshot(inst_name, None, Some("snap 2".to_string()))
            .unwrap();
        let _snap3 = store
            .create_snapshot(inst_name, None, Some("snap 3".to_string()))
            .unwrap();

        assert_eq!(store.data.instances[inst_name].details.snapshots.len(), 3);

        // Non-destructive restore: preserves subsequent snapshots
        let restored_non_destructive = store
            .restore_snapshot(inst_name, &snap2.name, false)
            .unwrap();
        assert_eq!(restored_non_destructive.name, snap2.name);
        assert_eq!(store.data.instances[inst_name].details.snapshots.len(), 3);

        // Destructive restore: prunes snapshots after restored one (snap3 removed, snap1 and snap2 retained)
        let restored_destructive = store
            .restore_snapshot(inst_name, &snap2.name, true)
            .unwrap();
        assert_eq!(restored_destructive.name, snap2.name);
        let remaining_snaps = &store.data.instances[inst_name].details.snapshots;
        assert_eq!(remaining_snaps.len(), 2);
        assert_eq!(remaining_snaps[0].name, snap1.name);
        assert_eq!(remaining_snaps[1].name, snap2.name);

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}
