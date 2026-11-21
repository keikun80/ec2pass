# InhausLab ec2pass - Vagrant-based EC2 (Amazon Linux 2023) Build & Test Environment

This directory provides tooling to automatically provision an **Amazon Linux 2023 / Enterprise Linux 9 virtual machine using Vagrant**, enabling one-click native compilation and full Linux E2E integration testing for `ec2pass`.

---

## 1. Prerequisites

On your host machine (Windows / macOS / Linux), install Vagrant and a virtualization provider (e.g., VirtualBox):

### Windows (PowerShell)
```powershell
# 1. Install Vagrant CLI
winget install HashiCorp.Vagrant

# 2. Install VirtualBox
winget install Oracle.VirtualBox
```
> *Open a new terminal window after installation to update your system PATH.*

### macOS (Homebrew)
```bash
brew install --cask vagrant virtualbox
```

---

## 2. One-Click Automated Runner (PowerShell)

Executing the host runner script from the project root will automatically perform the entire pipeline: **Provision VM -> Configure Environment -> Compile Source Code -> Run Unit Tests -> Run 7-Stage E2E Integration Tests -> Shutdown**:

```powershell
# Default run (VM automatically suspends after tests complete)
powershell -ExecutionPolicy Bypass -File packaging\vagrant\run_vagrant_test.ps1

# Keep VM running after tests for interactive SSH debugging
powershell -ExecutionPolicy Bypass -File packaging\vagrant\run_vagrant_test.ps1 -KeepRunning

# Destroy VM immediately after test completion (Clean-room CI mode)
powershell -ExecutionPolicy Bypass -File packaging\vagrant\run_vagrant_test.ps1 -DestroyOnFinish
```

---

## 3. Manual CLI Usage Guide

If you prefer to control Vagrant directly step-by-step:

### 3.1 Provision and Boot the Virtual Machine
```bash
# Create and start the VM (downloads box and installs Rust/QEMU on initial boot)
vagrant up

# To launch with more vCPUs or RAM:
VM_CPUS="4" VM_MEMORY="8192" vagrant up
```

### 3.2 SSH Connection and Automated Verification
```bash
# Connect to the guest VM via SSH
vagrant ssh

# Run the build and 7-stage E2E test script inside the guest
bash /vagrant/packaging/vagrant/build_and_test.sh
```

### 3.3 Manual Development & Debugging Inside Guest
```bash
cd /vagrant

# 1. Release compilation
export CARGO_TARGET_DIR=~/target_cache
cargo build --workspace --release

# 2. Run unit tests
cargo test --workspace

# 3. Test ec2pass commands directly
/usr/local/bin/ec2pass --version
/usr/local/bin/ec2pass find
/usr/local/bin/ec2pass launch al2023 -n dev-box
/usr/local/bin/ec2pass shell dev-box
```

### 3.4 Teardown and Cleanup
```bash
# Suspend or halt the VM (preserves state)
vagrant halt

# Destroy the VM and reclaim disk space
vagrant destroy -f
```

---

## 4. File Structure & Roles

| File | Role |
| :--- | :--- |
| **`Vagrantfile`** | Located in project root. Configures vCPU/RAM, enables nested hardware virtualization (`--nested-hw-virt`), and defines `/vagrant` shared folder. |
| **`packaging/vagrant/provision.sh`** | Runs once during initial boot: installs DNF build dependencies, QEMU/KVM packages, and Rust toolchain. |
| **`packaging/vagrant/build_and_test.sh`** | Runs inside guest: performs Rust release build, runs unit tests, deploys binaries, and executes `tests/e2e_test.sh`. |
| **`packaging/vagrant/run_vagrant_test.ps1`** | One-click automated test runner for Windows hosts. |
