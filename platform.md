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

## 2. Native Hypervisor Policy & Emulation Modes

InhausLab ec2pass defaults to strict pre-flight checks in `crates/ec2ps-core/src/hypervisor/`. If hardware acceleration is missing or inaccessible, **ec2pass immediately terminates with a clear, actionable guide rather than silently degrading performance.**

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

### 2.1 Optional Unaccelerated Fallback (`EC2PASS_ALLOW_UNACCELERATED=1`)
In specialized scenarios such as nested cloud environments, Docker containers, or CI/CD test appliances where `/dev/kvm` cannot be exposed, setting `EC2PASS_ALLOW_UNACCELERATED=1` allows `ec2pass` to boot VMs using QEMU's TCG (Tiny Code Generator) software CPU emulator (`-accel tcg -cpu max`).

### 2.2 QMP Reconnect Resilience
When managing VM lifecycle states rapidly (e.g. `restart` followed immediately by `suspend`), `ec2pass` applies a 10-retry loop with exponential backoff on the QMP TCP socket to ensure zero dropped commands during hypervisor boot handoffs.

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

---

## 5. Multi-Platform Verification Results (v0.1.1)

All 26 subcommands and 38 parameter/format permutations have been comprehensively tested with a **100% pass rate** across all supported platforms:

| Target Operating System | Architecture | Virtualization Driver | Test Runner | Results Capture |
| :--- | :--- | :--- | :--- | :--- |
| **macOS Darwin** | `x86_64` (Intel) / Apple Silicon | QEMU Hypervisor.framework (`HVF`) | `tests/comprehensive_matrix_test.sh` | [test_results_macos.md](packaging/test_results_macos.md) |
| **Linux (Amazon Linux 2023.9)** | `x86_64` (Kernel 6.1) | QEMU KVM / TCG | `tests/comprehensive_matrix_test.sh` | [test_results_linux_al2023.md](packaging/test_results_linux_al2023.md) |
| **Windows 11 Enterprise** | `x86_64` | Hyper-V / WHPX | `tests/comprehensive_matrix_test.ps1` | [test_results_windows.md](packaging/test_results_windows.md) |
