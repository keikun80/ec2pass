# InhausLab ec2pass

[![Rust](https://img.shields.io/badge/rust-2021_edition-orange.svg)](https://www.rust-lang.org/)
[![Version: 0.1.3](https://img.shields.io/badge/version-0.1.3-green.svg)](RELEASE_NOTES.md)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg)](#platform-support)

**InhausLab ec2pass** is a high-performance, lightweight virtualization orchestrator written in 100% pure Rust, designed to run official Amazon Web Services (AWS) **Amazon Linux 2023 (AL2023)** images with a modern, intuitive command-line interface.

> **[Notice]** `InhausLab ec2pass` is an independent, clean-room open-source project and is not affiliated with, endorsed by, or sponsored by Amazon Web Services, Inc. See [Legal Notice & Trademark Disclaimer](#legal-notice--trademark-disclaimer).

---

## Key Features

- **Full-Featured Cloud Instance CLI**: Complete VM lifecycle management supporting `find`, `launch`, `list`, `info`, `exec`, `shell`, `start`, `stop`, `restart`, `suspend`, `transfer`, `mount`, `umount`, `snapshot`, `restore`, `clone`, `delete`, `recover`, `purge`, `get`, `set`, `alias`, `unalias`, `aliases`, and `networks`.
- **Pure Rust Architecture**: Clean-room implementation built on the modern Rust 2021 edition.
- **Pure Rust ISO9660 / NoCloud Engine**: Built-in `ec2ps-iso9660` generates Rock Ridge / Joliet `cidata` seed ISOs directly in memory with zero external utilities (`mkisofs`, `genisoimage`, or `xorriso` are not required).
- **Official AWS AL2023 Images**: Automatically discovers and streams official AWS KVM images from the AWS CDN, verifying SHA256 checksums and caching them locally.
- **Dynamic Download Progress**: Live terminal ASCII progress bar, percentage (`[XX%]`), downloaded bytes/total (`(X.XGiB / Y.YGiB)`), and SHA-256 verification when pulling AWS CDN images.
- **Clean Catalog & Automatic Aliasing**: Streamlined `find` table output; automatic resolution for legacy and convenience aliases (`al2023-k6.12`, `kernel-6.12`, `minimal`, `micro`).
- **Resilient Hypervisor Layer**: Automatic QMP socket connection retry loop with backoff to prevent race conditions during rapid lifecycle transitions; unaccelerated fallback (`EC2PASS_ALLOW_UNACCELERATED=1`) for nested virtualization and CI environments.
- **Comprehensive Multi-Platform Test Automation**: Automated 38-combination matrix tests across macOS, Linux, and Windows with 100% pass verification.
- **Strict Native Hypervisors**:
  - **Linux**: Kernel-based Virtual Machine (**KVM**) via QEMU (`-enable-kvm -cpu host`) with optional software TCG fallback.
  - **macOS**: Hypervisor.framework (**HVF**) via QEMU (`-accel hvf -cpu host`).
  - **Windows**: Windows Hypervisor Platform (**WHPX**) or native **Hyper-V**.
- **Two-Tier Storage Separation**: Execution binaries (`%LOCALAPPDATA%\Programs\ec2pass` or `/usr/local/bin`) and persistent VM data (`%LOCALAPPDATA%\ec2pass` or `~/.local/share/ec2pass`) are cleanly isolated. Reinstallations or updates never touch existing VM images or SSH keys.
- **Enterprise Uninstaller**: Clean removal via GUI Settings, shell script, or batch file, with an optional `-PurgeData` flag to wipe VM disks and caches.

---

## Workspace Structure

```text
ec2pass/
├── crates/
│   ├── ec2ps-core/        # Virtualization engine, QEMU hypervisor driver, image cache & cloud-init
│   ├── ec2ps-cli/         # CLI client binary (ec2pass)
│   ├── ec2ps-daemon/      # Background virtualization management service (ec2passd)
│   ├── ec2ps-protocol/    # IPC protocol definitions (JSON-RPC over TCP / Unix Domain Sockets)
│   └── ec2ps-iso9660/     # Pure Rust ISO9660 / Rock Ridge generator for cloud-init NoCloud seed.iso
├── packaging/
│   ├── windows/           # Windows installer (Setup.bat, install.ps1, uninstall.ps1, clean.ps1)
│   ├── macos/             # macOS universal package (install.sh, uninstall.sh)
│   ├── linux/             # Linux package & systemd unit (install.sh, uninstall.sh)
│   └── vagrant/           # Automated AL2023 KVM integration test environment
├── tests/
│   ├── comprehensive_matrix_test.sh   # Automated 38-combination matrix test runner (Bash)
│   ├── comprehensive_matrix_test.ps1  # Automated 38-combination matrix test runner (PowerShell)
│   ├── e2e_test.sh        # Comprehensive 7-stage end-to-end lifecycle verification test suite (Bash)
│   └── e2e_test.ps1       # Comprehensive 7-stage end-to-end lifecycle verification test suite (PowerShell)
├── README.md
├── MANUAL.md              # Full CLI reference guide
├── RELEASE_NOTES.md       # Version release notes and changelog
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
ec2pass snapshot dev-box -n pre-update -c "Backup before package update"

# Restore from snapshot
ec2pass restore dev-box -s pre-update

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

1. **Amazon Linux & AWS**: Amazon Linux, AWS, and EC2 are trademarks of Amazon.com, Inc. or its affiliates. Official AL2023 KVM images are fetched directly from public AWS distribution mirrors according to AWS terms.
2. **Clean-Room Implementation**: All code in this repository was authored from scratch in Rust based on public documentation and protocol analysis.
