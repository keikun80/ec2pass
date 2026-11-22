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

### 3.4 Windows 11 Enterprise / Pro Vagrant Environment
To test `ec2pass` on Windows 11 using VirtualBox:

```bash
# 1. Boot Windows 11 VM (downloads box with TPM 2.0 and UEFI emulation)
vagrant up win11

# 2. Or run the complete one-click host runner from macOS / Linux:
bash packaging/vagrant/run_vagrant_win11_test.sh

# 3. Interactive WinRM shell into Windows 11 guest:
vagrant winrm win11

# 4. Halt Windows 11 VM when finished:
vagrant halt win11
```

### 3.5 Teardown and Cleanup
```bash
# Suspend or halt the VMs (preserves state)
vagrant halt al2023
vagrant halt win11

# Destroy VMs and reclaim disk space
vagrant destroy -f al2023
vagrant destroy -f win11
```

---

## 4. File Structure & Roles

| File | Role |
| :--- | :--- |
| **`Vagrantfile`** | Project root multi-machine Vagrant definition supporting `al2023` (primary Linux) and `win11` (Windows 11 with TPM 2.0 & EFI). |
| **`packaging/vagrant/Vagrantfile.win11`** | Standalone Windows 11 Vagrantfile for dedicated Windows 11 test runs. |
| **`packaging/vagrant/provision.sh`** | Linux initial boot script: installs DNF build dependencies, QEMU/KVM packages, and Rust toolchain. |
| **`packaging/vagrant/provision_win11.ps1`** | Windows 11 initial boot script: configures execution policy, installs Git and Rust MSVC toolchain. |
| **`packaging/vagrant/build_and_test.sh`** | Runs inside Linux guest: compiles, runs unit tests, and executes `tests/e2e_test.sh`. |
| **`packaging/vagrant/build_and_test_win11.ps1`** | Runs inside Windows 11 guest: compiles `ec2pass.exe`, runs unit tests, runs `tests/e2e_test.ps1`, and creates `.zip` release. |
| **`packaging/vagrant/run_vagrant_test.sh`** | Host automation runner for Linux (`al2023`) build and test. |
| **`packaging/vagrant/run_vagrant_win11_test.sh`** | Host automation runner for Windows 11 (`win11`) build and test. |
| **`packaging/vagrant/run_vagrant_test.ps1`** | One-click automated test runner for Windows hosts. |
