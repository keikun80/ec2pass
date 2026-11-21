# Multi-Platform Support & Hypervisor Architecture

This document outlines the multi-platform operating system matrix, native hypervisor requirements, and runtime architectures supported by **InhausLab ec2pass**.

---

## 1. Platform Support Matrix

| Operating System | Architecture | Virtualization Backend | Acceleration Flag | Guest Architecture |
| :--- | :--- | :--- | :--- | :--- |
| **Linux** (Ubuntu, RHEL, AL2023, Debian, Fedora, Arch) | `x86_64`, `aarch64` | **KVM** (`/dev/kvm`) | `-enable-kvm -cpu host` | x86_64 / aarch64 native |
| **macOS** (Sequoia, Sonoma, Ventura) | Apple Silicon (`aarch64`), Intel (`x86_64`) | **Hypervisor.framework (HVF)** | `-accel hvf -cpu host` | aarch64 (M-series) / x86_64 |
| **Windows** (10 / 11 64-bit) | `x86_64` | **WHPX** / **Hyper-V** | `-accel whpx` / Native Hyper-V | x86_64 native |

---

## 2. Mandatory Native Hypervisor Policy (No Simulation)

InhausLab ec2pass enforces strict pre-flight checks in `crates/ec2ps-core/src/hypervisor/`. If hardware acceleration is missing or inaccessible, **ec2pass immediately terminates with a clear, actionable guide rather than falling back to an unaccelerated simulation or mock environment.**

```text
┌─────────────────────────────────────────────────────────────┐
│                    ec2pass Pre-flight Check                 │
│  - Linux:   Validates /dev/kvm accessibility                │
│  - macOS:   Validates HVF (sysctl kern.hv_support == 1)    │
│  - Windows: Validates WHPX / Hyper-V service status         │
└──────────────────────────────┬──────────────────────────────┘
                               │
                Is acceleration available?
                               │
               ┌───────────────┴───────────────┐
               ▼                               ▼
            [ YES ]                         [ NO ]
      Proceed with native           ABORT with diagnostic guide:
       hardware execution           - Exact installation commands
                                    - Permission fix instructions
                                    - BIOS virtualization guide
```

---

## 3. Platform Setup Guides

### 3.1 Linux (KVM)
Ensure that your processor supports hardware virtualization (VT-x/AMD-V) and that KVM is enabled:

```bash
# 1. Verify CPU virtualization support
egrep -c '(vmx|svm)' /proc/cpuinfo

# 2. Install QEMU and KVM packages
# On Debian/Ubuntu:
sudo apt-get update && sudo apt-get install -y qemu-system-x86 qemu-utils

# On Fedora/RHEL/AL2023:
sudo dnf install -y qemu-kvm qemu-img

# 3. Grant current user access to /dev/kvm
sudo usermod -aG kvm $USER
# (Log out and log back in for group changes to take effect)
```

### 3.2 macOS (Hypervisor.framework)
Hypervisor.framework (`HVF`) is natively built into macOS on both Apple Silicon (M1/M2/M3/M4) and Intel Macs:

```bash
# 1. Verify HVF support (must return 1)
sysctl -n kern.hv_support

# 2. Install QEMU
brew install qemu
```

### 3.3 Windows (Hyper-V / WHPX)
Enable Windows Hypervisor Platform and Hyper-V:

```powershell
# Run in an elevated administrator PowerShell:
Enable-WindowsOptionalFeature -Online -FeatureName HypervisorPlatform -All
Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V -All
```

---

## 4. IPC and Process Architecture

- **Daemon (`ec2passd`)**:
  - **Linux / macOS**: Listens on Unix Domain Sockets (`/var/run/ec2pass.sock` for system services, or `$XDG_RUNTIME_DIR/ec2pass.sock` for user sessions).
  - **Windows**: Listens on local TCP loopback (`127.0.0.1:50051`) or named pipes.
- **Client (`ec2pass`)**:
  - Connects to the daemon via the unified protocol layer (`ec2ps-protocol`).
  - Transparently handles streaming output for commands such as `exec`, `shell`, and `launch`.
