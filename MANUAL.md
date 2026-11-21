# InhausLab ec2pass (`ec2pass` / `multipass`) CLI Manual

This manual provides a complete reference for all subcommands, options, output formats, and recipes in **InhausLab ec2pass**, an Amazon Linux 2023 virtualization orchestrator providing 100% command-line compatibility with Canonical Multipass.

> **[Notice]** `InhausLab ec2pass` is an independent, clean-room open-source project and is not affiliated with, endorsed by, or sponsored by Canonical Ltd. or Amazon Web Services, Inc. The `multipass` binary is provided strictly as a drop-in compatibility alias for migration convenience. See [Section 4. Legal Notice & Trademark Disclaimer](#4-legal-notice--trademark-disclaimer).

---

## 1. Global Options

```bash
ec2pass [OPTIONS] <COMMAND>
```

| Option | Environment Variable | Default | Description |
| :--- | :--- | :--- | :--- |
| `--daemon-addr <ADDR>` | `MP_DAEMON_ADDR` | `127.0.0.1:50051` | Address or socket of the background daemon (`ec2passd`) |
| `-v, --verbose` | - | 0 | Increase logging verbosity (`-v`, `-vv`, `-vvv`) |
| `-h, --help` | - | - | Print help information |
| `-V, --version` | - | - | Print version information |

---

## 2. Command Reference

### 2.1 `find` - Search Available Image Catalog
Queries available operating system images from the official AWS Amazon Linux 2023 distribution CDN.

```bash
ec2pass find [SEARCH_STRING] [--show-unsupported] [--force-update] [--format <FORMAT>]
```
* **Arguments & Flags**:
  * `[SEARCH_STRING]`: Filter keywords (e.g., `minimal`, `kernel`, `al2`).
  * `--force-update`: Force refresh of the remote AWS CDN image index.
  * `--format <FORMAT>`: Output format (`table`, `json`, `yaml`, `csv`).
* **Examples**:
  ```bash
  # List all available images
  ec2pass find

  # Filter with JSON output
  ec2pass find minimal --format json
  ```

---

### 2.2 `launch` - Create and Boot an Instance
Creates a CoW QCOW2 overlay disk and NoCloud `seed.iso` from an Amazon Linux 2023 base image, then boots the instance.

```bash
ec2pass launch [IMAGE] [OPTIONS]
```
* **Options**:
  * `[IMAGE]`: Image alias (default: `al2023` / `latest`). Supported: `al2023`, `al2023-k6.12`, `al2023-minimal`, `al2`, or a custom image path/URL.
  * `-n, --name <NAME>`: Instance name (auto-generates petnames such as `bouncy-lynx` if omitted).
  * `-c, --cpus <COUNT>`: Number of vCPUs to allocate (default: `1`).
  * `-m, --memory <SIZE>`: Memory size (default: `1G`, e.g., `512M`, `2G`, `4GiB`).
  * `-d, --disk <SIZE>`: Root disk size (default: `5G`, e.g., `10G`, `20GiB`).
  * `--cloud-init <FILE|- >`: Path to a cloud-config YAML file or shell script (or `-` for stdin).
  * `--mount <LOCAL:GUEST>`: Mount a host directory on initial launch.
  * `--timeout <SECS>`: Maximum wait time for instance ready state (default: `300`).
* **Examples**:
  ```bash
  # Launch default instance
  ec2pass launch

  # Launch with custom specs and name
  ec2pass launch al2023 -n ec2-dev --cpus 2 --memory 4G --disk 15G

  # Launch with cloud-init and host directory mount
  ec2pass launch al2023 -n web-srv --cloud-init ./init.yaml --mount ./my-web:/var/www/html
  ```

---

### 2.3 `list` - List Instances or Snapshots
Displays the status, IPv4 address, and image information for instances, or lists snapshot trees.

```bash
ec2pass list [--format <FORMAT>] [--snapshots]
```
* **Options**:
  * `--format <FORMAT>`: `table` (default), `json`, `yaml`, `csv`.
  * `--snapshots`: List all snapshot trees instead of instances.
* **Examples**:
  ```bash
  ec2pass list
  ec2pass list --format json
  ec2pass list --snapshots
  ```

---

### 2.4 `info` - Display Instance Details
Shows vCPU allocation, real-time memory and disk usage, load averages, active mounts, and snapshot history.

```bash
ec2pass info [NAMES...] [--all] [--format <FORMAT>]
```
* **Examples**:
  ```bash
  ec2pass info ec2-dev
  ec2pass info --all --format yaml
  ```

---

### 2.5 `exec` - Run Commands Inside Instance
Executes commands inside a running instance and returns stdout, stderr, and exit codes.

```bash
ec2pass exec <NAME> [-d <DIR>] -- <COMMAND...>
```
* **Options**:
  * `-d, --working-directory <DIR>`: Working directory inside the guest prior to execution.
* **Examples**:
  ```bash
  # Check Amazon Linux 2023 release version
  ec2pass exec ec2-dev -- cat /etc/os-release

  # Install packages
  ec2pass exec ec2-dev -- sudo dnf install -y htop

  # Run command from a specific directory
  ec2pass exec ec2-dev -d /var/log -- ls -la
  ```

---

### 2.6 `shell` - Open an Interactive Terminal
Connects an interactive shell session as `ec2-user`.

```bash
ec2pass shell [NAME]
```
* If no name is provided, connects to the default instance (`primary`), automatically provisioning it if necessary.

---

### 2.7 `start` / `stop` / `restart` / `suspend` - Lifecycle Management

```bash
# Start instance
ec2pass start [NAMES...] [--all]

# Graceful shutdown (ACPI system_powerdown)
ec2pass stop [NAMES...] [--all]
ec2pass stop ec2-dev --force           # Force immediate stop
ec2pass stop ec2-dev -t 10             # Scheduled shutdown in 10 minutes
ec2pass stop ec2-dev -c                # Cancel scheduled shutdown

# Restart instance
ec2pass restart [NAMES...] [--all]

# Suspend instance (freeze memory state)
ec2pass suspend [NAMES...] [--all]
```

---

### 2.8 `transfer` - Transfer Files Host $\leftrightarrow$ Guest
Transfers files or directories between the host and instance paths (`instance:path`).

```bash
ec2pass transfer [OPTIONS] <SRC...> <DST>
```
* **Options**:
  * `-r, --recursive`: Recursively copy directories.
  * `-p, --parents`: Create parent directories at destination if missing.
* **Examples**:
  ```bash
  # Copy file from host to instance
  ec2pass transfer ./app.jar ec2-dev:/home/ec2-user/app.jar

  # Download file from instance to host
  ec2pass transfer ec2-dev:/var/log/audit.log ./audit.log

  # Transfer an entire folder recursively
  ec2pass transfer -r ./src-dir ec2-dev:/home/ec2-user/project
  ```

---

### 2.9 `mount` & `umount` - Directory Mounting
Mounts a local host directory into a guest path in real-time.

```bash
# Mount host path
ec2pass mount <HOST_PATH> <INSTANCE>[:<GUEST_PATH>]

# Unmount guest path
ec2pass umount <INSTANCE>[:<GUEST_PATH>]
```
* **Examples**:
  ```bash
  ec2pass mount /Users/dev/workspace ec2-dev:/home/ec2-user/workspace
  ec2pass umount ec2-dev:/home/ec2-user/workspace
  ```

---

### 2.10 `snapshot` & `restore` - Snapshots and Rollback
Captures point-in-time disk and hardware configuration for stopped instances and restores them on demand.

```bash
# Create snapshot
ec2pass snapshot <INSTANCE> [-n <NAME>] [-c <COMMENT>]

# Restore from snapshot
ec2pass restore <INSTANCE>.<SNAPSHOT_NAME>
```
* **Examples**:
  ```bash
  ec2pass stop ec2-dev
  ec2pass snapshot ec2-dev --name pre-upgrade --comment "Backup before kernel update"
  ec2pass restore ec2-dev.pre-upgrade
  ```

---

### 2.11 `clone` - Duplicate an Instance
Clones a stopped instance's disk and hardware configuration into a new, independent instance.

```bash
ec2pass clone <SOURCE_NAME> [-n <NEW_NAME>]
```
* **Examples**:
  ```bash
  ec2pass stop ec2-dev
  ec2pass clone ec2-dev -n ec2-staging
  ```

---

### 2.12 `delete` / `recover` / `purge` - Instance Deletion & Recovery
Implements a safe two-stage deletion model:

```bash
# 1. Mark instance as Deleted (moved to trash)
ec2pass delete <NAMES...> [--all]

# Option: delete and purge immediately
ec2pass delete ec2-dev --purge

# 2. Recover an instance from Deleted state
ec2pass recover <NAMES...> [--all]

# 3. Permanently remove all Deleted instances from disk
ec2pass purge
```

---

### 2.13 `get` & `set` - Dynamic Configuration & Resource Resizing
Queries configuration trees and resizes resources (vCPU, RAM, disk) on stopped instances.

```bash
# List all available configuration keys
ec2pass get --keys

# Inspect configuration values
ec2pass get local.ec2-dev.cpus
ec2pass get local.ec2-dev.memory

# Resize resources on a stopped instance
ec2pass stop ec2-dev
ec2pass set local.ec2-dev.cpus=4
ec2pass set local.ec2-dev.memory=8G
ec2pass set local.ec2-dev.disk=30G
ec2pass start ec2-dev
```

---

### 2.14 `alias` / `unalias` / `aliases` - Guest Command Aliasing
Exposes commands installed inside an instance as local host CLI commands.

```bash
# Expose the dnf package manager from ec2-dev to host shell
ec2pass alias ec2-dev:dnf

# List active aliases
ec2pass aliases

# Remove alias
ec2pass unalias dnf
```

---

### 2.15 `networks` - Virtual Network Interfaces
Displays virtual bridge and NAT interfaces available on the host.

```bash
ec2pass networks [--format <FORMAT>]
```

---

## 3. Recipes and Practical Examples

### Recipe 1: Pre-validating `cloud-init` locally before AWS EC2 deployment
```yaml
# cloud-config.yaml
#cloud-config
packages:
  - nginx
  - git
write_files:
  - path: /var/www/html/index.html
    content: "<h1>Amazon Linux 2023 on Local ec2pass</h1>"
runcmd:
  - systemctl enable --now nginx
```

```bash
ec2pass launch al2023 -n ec2-web --cloud-init cloud-config.yaml
ec2pass exec ec2-web -- curl -s http://localhost
```

### Recipe 2: Live development environment connected to local IDE
```bash
ec2pass launch al2023 -n dev-env --cpus 2 --memory 4G
ec2pass mount /Users/dev/Projects/MyService dev-env:/home/ec2-user/service
ec2pass exec dev-env -d /home/ec2-user/service -- cargo test
```

---

## 4. Legal Notice & Trademark Disclaimer

### 4.1 Trademark Notice & Non-Affiliation
* **Multipass** and **Ubuntu** are registered trademarks or trademarks of Canonical Ltd.
* **InhausLab ec2pass** is an independent third-party open-source project and is not affiliated with, sponsored by, or endorsed by Canonical Ltd.
* References to "Multipass" and "Ubuntu" are made strictly for identification, technical compatibility, and interoperability purposes under **Nominative Fair Use**.

### 4.2 Drop-in Compatibility Alias Statement
* The `multipass` executable provided in release bundles is not intended to claim brand ownership. It is provided strictly as a **functional drop-in compatibility alias** to enable seamless script migration and workflow continuity for existing Multipass users.
* The official product name is **`InhausLab ec2pass`**.

### 4.3 Copyright & Clean-Room Implementation
* This software does not copy or contain Canonical's C++ source code. It is an independent, clean-room implementation written 100% in Rust based on public specifications and protocol observations.
* Command-line interfaces and syntactic compatibility fall under fair use principles for functional interoperability (*Google LLC v. Oracle America, Inc.*).

### 4.4 Amazon Linux Trademark Notice
* **Amazon Linux**, **Amazon Web Services**, **AWS**, and **EC2** are trademarks of Amazon.com, Inc. or its affiliates.
* This project downloads and runs official Amazon Linux KVM images from public AWS mirrors and does not imply an official partnership or endorsement.

---

## 5. Installation & Uninstallation Guide

### 5.1 Windows

#### Installation
1. **Installer**: Double-click `ec2pass-setup.exe` to copy files to `%LOCALAPPDATA%\Programs\ec2pass` and register the directory to the user `PATH`.
2. **Script Installation**:
   ```cmd
   Setup.bat
   ```
   Or PowerShell:
   ```powershell
   powershell -ExecutionPolicy Bypass -File install.ps1
   ```

#### Uninstallation
`ec2pass` supports clean uninstallation:
1. **Windows Settings (GUI)**: Go to **Settings** (`Win + I`) > **Apps** > **Installed Apps**, locate **`InhausLab ec2pass`**, and click **Uninstall**.
2. **Uninstall.bat**: Run `Uninstall.bat` in `%LOCALAPPDATA%\Programs\ec2pass`.
3. **PowerShell**:
   ```powershell
   # Remove binaries and PATH entries (preserves VM data)
   powershell -ExecutionPolicy Bypass -File uninstall.ps1

   # Permanently remove VM disks and image caches as well
   powershell -ExecutionPolicy Bypass -File uninstall.ps1 -PurgeData
   ```

---

### 5.2 Linux & macOS

#### Installation
```bash
# Linux
tar -xzf ec2pass-linux-x86_64.tar.gz
sudo ./install.sh

# macOS
tar -xzf ec2pass-macos-universal.tar.gz
sudo ./install.sh
```

#### Uninstallation
```bash
# Standard uninstall (stops daemon, removes binaries, preserves VM data)
sudo ./packaging/linux/uninstall.sh    # Linux
sudo ./packaging/macos/uninstall.sh    # macOS

# Full purge (removes all VM disks and caches)
sudo ./packaging/linux/uninstall.sh --purge-data
sudo ./packaging/macos/uninstall.sh --purge-data
```

---

## 6. Directory and Storage Architecture

`InhausLab ec2pass` separates runtime binaries from persistent VM data in a **two-tier storage architecture**, ensuring upgrades never affect existing virtual machines or SSH keys.

```text
┌────────────────────────────────────────────────────────┐
│  1. Program Binary Directory (Install Directory)       │
│  - ec2pass, ec2passd, multipass                        │
│  - Uninstaller scripts, documentation                   │
└────────────────────────────────────────────────────────┘
                           ▲
                           │  Orchestration
                           ▼
┌────────────────────────────────────────────────────────┐
│  2. Persistent VM Data Store (Data Directory)          │
│  - AWS AL2023 base KVM image cache                     │
│  - CoW overlay delta QCOW2 virtual disks per instance  │
│  - cloud-init seed.iso and ED25519 keypairs            │
│  - Atomic state database (state.json)                  │
└────────────────────────────────────────────────────────┘
```

---

### 6.1 Binary Installation Directories

| Operating System | Mode | Directory Path |
| :--- | :--- | :--- |
| **Windows** | Standard User (Default) | `%LOCALAPPDATA%\Programs\ec2pass`<br>*(e.g., `C:\Users\<user>\AppData\Local\Programs\ec2pass`)* |
| **Windows** | Administrator | `%ProgramFiles%\ec2pass`<br>*(e.g., `C:\Program Files\ec2pass`)* |
| **Linux** | System-wide | `/usr/local/bin/` |
| **macOS** | System-wide | `/usr/local/bin/` |

---

### 6.2 Data and Virtual Machine Directories

| Operating System | Default Data Directory Path |
| :--- | :--- |
| **Windows** | `%LOCALAPPDATA%\ec2pass`<br>*(e.g., `C:\Users\<user>\AppData\Local\ec2pass`)* |
| **Linux** | `/var/lib/ec2pass` (system daemon) or `~/.local/share/ec2pass` |
| **macOS** | `~/Library/Application Support/ec2pass`<br>*(or `/var/root/Library/Application Support/ec2pass`)* |

> **[Customizing Data Storage Path]**  
> To store VM images on a secondary drive (e.g., `/data/ec2pass` or `D:\ec2pass-data`), set the **`EC2PASS_HOME`** environment variable.

#### Data Directory Structure
```text
ec2pass/
│
├── cache/
│   └── images/
│       ├── al2023-kvm-2023.6.20250128.0-kernel-6.1-x86_64.x86_64.qcow2
│       │   # Official, unmodified base KVM image from AWS CDN (read-only)
│       └── ...
│
├── instances/
│   ├── dev-env/
│   │   ├── disk.qcow2    # CoW delta overlay disk referencing base image
│   │   └── seed.iso      # Pure Rust generated NoCloud seed image (cidata)
│   │
│   └── web-srv/
│       ├── disk.qcow2
│       └── seed.iso
│
├── ssh-keys/
│   ├── id_ed25519_ec2pass      # ED25519 private key for passwordless login
│   └── id_ed25519_ec2pass.pub  # Injected into seed.iso authorized_keys
│
└── state.json
    # Atomic JSON database storing instance state, IP, vCPU, RAM, disk, mounts
```
