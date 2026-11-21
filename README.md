# InhausLab ec2pass (`ec2pass` / `multipass`)

[![Rust](https://img.shields.io/badge/rust-2021_edition-orange.svg)](https://www.rust-lang.org/)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg)](#platform-support)

**InhausLab ec2pass** is a high-performance, lightweight virtualization orchestrator written in 100% pure Rust, designed to run official Amazon Web Services (AWS) **Amazon Linux 2023 (AL2023)** images with **100% CLI drop-in compatibility for Canonical Multipass**.

> **[Notice]** `InhausLab ec2pass` is an independent, clean-room open-source project and is not affiliated with, endorsed by, or sponsored by Canonical Ltd. or Amazon Web Services, Inc. The `multipass` binary is provided strictly as a drop-in compatibility alias for migration convenience. See [Legal Notice & Trademark Disclaimer](#legal-notice--trademark-disclaimer).

---

## Key Features

- **100% Multipass CLI Compatibility**: Full drop-in replacement supporting `find`, `launch`, `list`, `info`, `exec`, `shell`, `start`, `stop`, `restart`, `suspend`, `transfer`, `mount`, `umount`, `snapshot`, `restore`, `clone`, `delete`, `recover`, `purge`, `get`, `set`, `alias`, `unalias`, `aliases`, and `networks`.
- **Pure Rust Architecture**: Clean-room implementation with zero dependencies on Canonical C++ code, built on the modern Rust 2021 edition.
- **Pure Rust ISO9660 / NoCloud Engine**: Built-in `ec2ps-iso9660` generates Rock Ridge / Joliet `cidata` seed ISOs directly in memory with zero external utilities (`mkisofs`, `genisoimage`, or `xorriso` are not required).
- **Official AWS AL2023 Images**: Automatically discovers and streams official AWS KVM images from the AWS CDN, verifying SHA256 checksums and caching them locally.
- **Strict Native Hypervisors (No Simulation)**:
  - **Linux**: Kernel-based Virtual Machine (**KVM**) via QEMU (`-enable-kvm -cpu host`).
  - **macOS**: Hypervisor.framework (**HVF**) via QEMU (`-accel hvf -cpu host`).
  - **Windows**: Windows Hypervisor Platform (**WHPX**) or native **Hyper-V**.
  - *Simulation or mock modes are strictly disabled: real hypervisor hardware acceleration is required.*
- **Two-Tier Storage Separation**: Execution binaries (`%LOCALAPPDATA%\Programs\ec2pass` or `/usr/local/bin`) and persistent VM data (`%LOCALAPPDATA%\ec2pass` or `~/.local/share/ec2pass`) are cleanly isolated. Reinstallations or updates never touch existing VM images or SSH keys.
- **Enterprise Uninstaller**: Clean removal via GUI Settings, shell script, or batch file, with an optional `-PurgeData` flag to wipe VM disks and caches.

---

## Workspace Structure

```text
ec2pass/
├── crates/
│   ├── ec2ps-core/        # Virtualization engine, QEMU hypervisor driver, image cache & cloud-init
│   ├── ec2ps-cli/         # CLI client binary (compiled as ec2pass & multipass)
│   ├── ec2ps-daemon/      # Background virtualization management service (ec2passd)
│   ├── ec2ps-protocol/    # IPC protocol definitions (JSON-RPC over TCP / Unix Domain Sockets)
│   └── ec2ps-iso9660/     # Pure Rust ISO9660 / Rock Ridge generator for cloud-init NoCloud seed.iso
├── packaging/
│   ├── windows/           # Windows installer (Setup.bat, install.ps1, uninstall.ps1, clean.ps1)
│   ├── macos/             # macOS universal package (install.sh, uninstall.sh)
│   ├── linux/             # Linux package & systemd unit (install.sh, uninstall.sh)
│   └── vagrant/           # Automated AL2023 KVM integration test environment
├── tests/
│   └── e2e_test.sh        # Comprehensive 7-stage end-to-end lifecycle verification test suite
├── README.md
├── MANUAL.md              # Full CLI reference guide
├── platform.md            # Multi-platform support matrix and hypervisor setup guide
├── USERDATA.md            # Shell script and cloud-init user-data testing guide
└── BUILD_AND_RELEASE.md   # Compilation and packaging instructions
```

---

## Quick Start

### 1. Launching an Amazon Linux 2023 Instance
```bash
# Launch a default Amazon Linux 2023 VM (auto-downloads official AWS image)
ec2pass launch

# Launch with custom specs and custom name
ec2pass launch al2023 -n dev-box --cpus 2 --memory 4G --disk 15G
```

### 2. Inspecting and Connecting
```bash
# List instances and IP addresses
ec2pass list

# Open an interactive bash shell into the instance
ec2pass shell dev-box

# Execute commands directly
ec2pass exec dev-box -- uname -a
ec2pass exec dev-box -- cat /etc/os-release
```

### 3. File Sharing and Directory Mounts
```bash
# Transfer files
ec2pass transfer ./app.tar.gz dev-box:/home/ec2-user/

# Mount host directory into guest in real-time
ec2pass mount ./src dev-box:/home/ec2-user/src
```

### 4. Snapshots and Cleanup
```bash
# Take a snapshot
ec2pass stop dev-box
ec2pass snapshot dev-box --name pre-update

# Restore from snapshot
ec2pass restore dev-box.pre-update

# Delete and purge
ec2pass delete dev-box
ec2pass purge
```

---

## Installation & Packaging

See [BUILD_AND_RELEASE.md](file:///Users/jason/project/ec2pass/BUILD_AND_RELEASE.md) for detailed compilation instructions.

- **Windows**: Run `ec2pass-setup.exe` or `Setup.bat`.
- **macOS**: Extract `ec2pass-macos-universal.tar.gz` and run `sudo ./install.sh`.
- **Linux**: Extract `ec2pass-linux-x86_64.tar.gz` and run `sudo ./install.sh`.

---

## Legal Notice & Trademark Disclaimer

1. **Multipass & Ubuntu**: Multipass and Ubuntu are registered trademarks of Canonical Ltd. `InhausLab ec2pass` is an independent third-party project with no corporate affiliation, endorsement, or sponsorship from Canonical Ltd. The `multipass` binary is provided solely as a nominative fair use compatibility alias to facilitate workflow transition.
2. **Amazon Linux & AWS**: Amazon Linux, AWS, and EC2 are trademarks of Amazon.com, Inc. or its affiliates. Official AL2023 KVM images are fetched directly from public AWS distribution mirrors according to AWS terms.
3. **Clean-Room Implementation**: All code in this repository was authored from scratch in Rust based on public documentation and protocol analysis, without utilizing Canonical's C++ source code.
