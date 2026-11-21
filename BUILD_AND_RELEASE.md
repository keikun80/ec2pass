# Build and Release Guide

This document describes the compilation prerequisites, build procedures, and release packaging workflows for **InhausLab ec2pass** (`ec2pass` / `multipass`) across Windows, macOS, and Linux.

---

## 1. Prerequisites

### Rust Toolchain
- **Rust**: 1.75.0 or later (Stable edition 2021).
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

### Virtualization & Hypervisor Requirements
*InhausLab ec2pass requires genuine hardware virtualization acceleration. Mock/simulated virtualization is strictly prohibited.*
- **Linux**: QEMU (`qemu-system-x86_64` / `qemu-img`) with KVM enabled (`/dev/kvm`).
- **macOS**: QEMU (`qemu-system-x86_64` or `qemu-system-aarch64`) with Hypervisor.framework (`HVF`) support.
  ```bash
  brew install qemu
  ```
- **Windows**: QEMU with WHPX enabled, or native Windows Hyper-V.

---

## 2. Compiling the Binaries

Build all workspace crates in release mode:

```bash
cargo build --workspace --release
```

This generates the following binaries in `target/release/`:
- `ec2pass` (CLI client)
- `ec2passd` (Daemon server)

Create the drop-in `multipass` compatibility alias:
```bash
# On Unix (macOS / Linux):
ln -sf ec2pass target/release/multipass

# On Windows (PowerShell):
Copy-Item target\release\ec2pass.exe target\release\multipass.exe
```

---

## 3. Running Tests

Run the Rust workspace test suite:
```bash
cargo test --workspace
```

Run the end-to-end (E2E) integration test suite (requires local hypervisor):
```bash
./tests/e2e_test.sh
```

---

## 4. Packaging Release Artifacts

The repository includes automated packaging scripts in `packaging/`:

### macOS Universal 2 Binary Package
Builds both `x86_64-apple-darwin` and `aarch64-apple-darwin` targets and bundles them using `lipo`:
```bash
./packaging/build_release.sh
```
This generates:
- `dist/ec2pass-macos-universal.tar.gz` containing `ec2pass`, `ec2passd`, `multipass`, and `packaging/macos/install.sh`.

### Linux Release Package
On a Linux host (or inside the Vagrant AL2023 environment):
```bash
bash packaging/linux/package.sh
```
This generates `dist/ec2pass-linux-x86_64.tar.gz`.

### Windows Release Package
On a Windows host (PowerShell):
```powershell
powershell -ExecutionPolicy Bypass -File packaging\windows\package.ps1
```
This generates `ec2pass-setup.exe` and `dist\ec2pass-windows-x86_64.zip`.
