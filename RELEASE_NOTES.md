# InhausLab ec2pass 0.1.1 Release Notes

**Release Version**: `0.1.1`  
**Release Date**: 2026-11-22  
**Target Architecture**: macOS Darwin (x86_64 / Apple Silicon Universal 2), Linux (x86_64), Windows 11 (x86_64)

---

## Overview

InhausLab `ec2pass` 0.1.1 delivers critical usability, catalog reliability, and hypervisor resilience enhancements, alongside full multi-platform matrix verification across macOS, Linux (Amazon Linux 2023), and Windows 11.

---

## What's New in 0.1.1

### 1. Real-Time Image Download Progress Bar & Percentage
* When downloading Amazon Linux 2023 or Amazon Linux 2 base images from the official AWS CDN, `ec2pass` now displays a live, dynamic ASCII progress bar and percentage indicator:
  ```text
  [ 85%] [Retrieving image] [================>   ] Downloading Amazon Linux 2023.12 LTS (1.4GiB / 1.6GiB)
  [100%] [Retrieving image] [====================] Download complete. Verifying SHA256 checksum...
  [100%] [Retrieving image] [====================] SHA256 verified successfully.
  ```
* Seamless terminal redraw with carriage return (`\r`) and clean completion messaging.

### 2. Catalog Polish & Streamlined `find` Output
* **Description Column Removed**: Cleaned up `ec2pass find` table rendering by removing the verbose `Description` column, ensuring clean output on standard terminal widths.
* **CDN Catalog Sanitation**: Removed non-existent phantom image references (`al2023-k6.12` and `al2023-minimal`) that previously produced HTTP 403 Forbidden errors.
* **Automatic Compatibility Aliasing**: Requests for `al2023-k6.12`, `kernel-6.12`, `minimal`, or `micro` automatically resolve to the official verified AL2023 LTS base image without error.

### 3. Hypervisor Resilience & QMP Reconnect Loop
* **QMP Socket Reconnect**: Added a 10-iteration retry loop (150ms backoff) when connecting to the VM's QMP TCP socket in `send_qmp_command`. This eliminates transient `Connection refused (os error 61)` race conditions when rapid transitions occur (such as `restart` followed immediately by `suspend`).
* **Unaccelerated / Nested Emulation Fallback**: Added automatic detection and support for `EC2PASS_ALLOW_UNACCELERATED=1`, allowing `ec2pass` and `ec2passd` to run using QEMU TCG emulation in environments without hardware KVM/HVF virtualization (e.g. nested cloud VMs, CI runners, and test appliances).

### 4. Comprehensive Subcommand & Option Matrix Test Automation
* Introduced automated end-to-end matrix test runners:
  * `tests/comprehensive_matrix_test.sh` (Bash for macOS and Linux)
  * `tests/comprehensive_matrix_test.ps1` (PowerShell for Windows)
* Exercises all 26 subcommands and 38 parameter/formatting permutations (`table`, `json`, `yaml`, `csv`) with 100% pass rate across macOS Darwin, Amazon Linux 2023, and Windows 11.
* Captured execution logs and cross-platform matrix report published in `packaging/test_results_*.md`.

---

## Upgrade Guide

### macOS (Homebrew / Direct Binary)
```bash
# Update existing binary or replace with 0.1.1
curl -fsSL https://github.com/keikun80/ec2pass/releases/download/0.1.1/ec2pass-0.1.1-macos-universal.tar.gz | tar -xz -C /usr/local/bin
```

### Linux (Amazon Linux 2023 / Enterprise Linux 9 / Ubuntu)
```bash
# Download and install 0.1.1 release package
curl -fsSL https://github.com/keikun80/ec2pass/releases/download/0.1.1/ec2pass-0.1.1-linux-x86_64.tar.gz | tar -xz -C /tmp
sudo /tmp/ec2pass-0.1.1-linux-x86_64/install.sh
```

### Windows 11 (PowerShell)
```powershell
# Extract release ZIP and execute Setup.bat / install.ps1
Invoke-WebRequest -Uri https://github.com/keikun80/ec2pass/releases/download/0.1.1/ec2pass-0.1.1-windows-x86_64.zip -OutFile ec2pass.zip
Expand-Archive -Path ec2pass.zip -DestinationPath C:\ec2pass
& C:\ec2pass\install.ps1
```

---

## Full Commit History
* `feat(progress)`: Display live progress bar and percentage during image download
* `feat(cli)`: Remove Description column from find output
* `fix(catalog)`: Remove non-existent CDN images and alias legacy variants to al2023
* `fix(qemu)`: Add retry loop for QMP socket connection during rapid restart
* `test(matrix)`: Add multi-platform comprehensive subcommand and option test matrix runners and captured results
* `chore(release)`: Bump version to 0.1.1 and add 0.1.1 release notes
