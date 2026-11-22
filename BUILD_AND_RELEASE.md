# Build and Release Guide

This document describes the compilation prerequisites, build procedures, and release packaging workflows for **InhausLab ec2pass** across Windows, macOS, and Linux.

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

---

## 3. Running Tests

### 3.1 Unit & Component Tests
Run the Rust workspace test suite:
```bash
cargo test --workspace
```

### 3.2 End-to-End (E2E) Integration Tests
Run the standard 7-stage lifecycle verification test suite:
* **Linux / macOS**:
  ```bash
  ./tests/e2e_test.sh
  ```
* **Windows (PowerShell)**:
  ```powershell
  pwsh -File .\tests\e2e_test.ps1
  ```

### 3.3 Comprehensive Subcommand & Option Matrix Test
Runs all 26 subcommands and 38 parameter/formatting permutations, capturing output to a markdown report:
* **Linux / macOS**:
  ```bash
  REPORT_FILE="packaging/test_results_local.md" bash tests/comprehensive_matrix_test.sh
  ```
* **Windows (PowerShell)**:
  ```powershell
  pwsh -File tests\comprehensive_matrix_test.ps1 -ReportFile "packaging\test_results_local.md"
  ```

---

## 4. Packaging Release Artifacts

The repository includes automated packaging scripts in `packaging/`:

### macOS Universal 2 Binary Package
Builds both `x86_64-apple-darwin` and `aarch64-apple-darwin` targets and bundles them using `lipo`:
```bash
./packaging/macos/build_macos_packages.sh 0.1.1
```
This generates:
- `dist/ec2pass-0.1.1-darwin-universal.tar.gz` containing `ec2pass`, `ec2passd`, documentation, and `packaging/macos/install.sh`.

### Linux Release Package
On a Linux host (or inside the Vagrant AL2023 environment):
```bash
bash packaging/build_release.sh 0.1.1
```
This generates `dist/ec2pass-0.1.1-linux-x86_64.tar.gz`.

### Windows Release Package
On a Windows host (PowerShell):
```powershell
powershell -ExecutionPolicy Bypass -File packaging\build_release.ps1 -version "0.1.1"
```
This generates `dist\ec2pass-0.1.1-windows-x86_64.zip`.

---

## 5. Automated CI/CD Release via GitHub Actions

This repository includes a fully automated release pipeline in [`.github/workflows/release.yml`](.github/workflows/release.yml).

When you push a version tag to GitHub:
```bash
# 1. Create a release tag (without 'v' prefix)
git tag 0.1.1

# 2. Push the tag to GitHub
git push origin 0.1.1
```

GitHub Actions will automatically:
1. Spin up **Linux (`ubuntu-latest`)**, **macOS (`macos-latest`)**, and **Windows (`windows-latest`)** cloud runners.
2. Compile and test release binaries on each native platform.
3. Package `.tar.gz` and `.zip` archives with SHA-256 checksums.
4. Publish a new release under **GitHub Releases** with all download assets attached.

