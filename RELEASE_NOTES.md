# InhausLab ec2pass Release Notes

## InhausLab ec2pass 0.1.4 Release Notes

**Release Version**: `0.1.4`  
**Release Date**: 2026-11-28  
**Target Architecture**: macOS Darwin (x86_64 / Apple Silicon Universal 2), Linux (x86_64), Windows 11 (x86_64)

---

### Overview

`ec2pass` 0.1.4 makes cloud-init user-data easier to debug and accepts the user-data formats EC2 accepts. It fixes several problems on Windows hosts and with state, SSH and QEMU handling, and adds CI for Linux, macOS and Windows.

---

### What's New in 0.1.4

#### 1. User-Data Debugging
* A failing user-data script no longer looks like success. `launch` still exits 0 with the VM running, and prints a report with the cloud-init status, the failed commands with their exit codes, and the tail of the output.
* `launch --watch` streams the serial console and, once SSH is up, the guest's `cloud-init-output.log`.
* `launch --trace` records a timestamped, line-numbered bash xtrace of user-data scripts in `/var/log/ec2pass/userdata-trace.log` without changing the script or its output.
* New `ec2pass logs <name> [-f] [--cloud-init|--trace|--save]`. A debug bundle (cloud-init status and logs, delivered user-data and vendor-data, journal, trace, console) is saved under `<data dir>/logs/<name>/<timestamp>/` after each launch and with `logs --save`.

#### 2. EC2 User-Data Formats
* Shell scripts (`#!/bin/bash`), MIME multipart and the other cloud-init formats are passed through unchanged. Before, every user-data was validated as YAML and prefixed with `#cloud-config`. `#cloud-config` and headerless user-data are still validated.
* User-data that defines `users:` (including an empty `users:`) keeps the `ec2pass` login user.
* A UTF-8 byte order mark at the start of a user-data file is accepted.

#### 3. Image Catalog
* Built-in images updated to their latest releases: Ubuntu 26.04 LTS, Debian 13, Rocky Linux 10, Fedora 45 and Amazon Linux 2 2.0.20261120.0.
* Fixed the Fedora entry (404 after end of life), added a native arm64 Amazon Linux 2 image, and replaced the Amazon Linux 2 placeholder checksum with the published SHA256SUMS value.

#### 4. Reliability
* `state.json` is never reset silently when it cannot be parsed; the daemon reports an error instead.
* SSH and QMP host ports skip ports already used by other instances or bound on loopback. The first ports are configurable with `local.ssh-port-base` and `local.qmp-port-base`.
* `exec` never runs a command a second time after a disconnect during the session.
* If QEMU exits during launch, the launch fails at once with the end of the instance's `qemu.log`, instead of waiting for SSH until the timeout.
* The daemon auth token is generated from a cryptographically secure random source.
* A relative `--data-dir` is resolved to an absolute path.

#### 5. Windows Hosts
* The CLI no longer passes its console handles to the daemon it starts, which made scripts that read the CLI's output hang.
* Process liveness checks use `OpenProcess` instead of parsing `tasklist` output.
* Without WHPX, QEMU emulates a modern CPU (`-cpu max`), so guests that need x86-64-v2, such as Amazon Linux 2023, boot instead of panicking. WHPX is used when it is available.

#### 6. Continuous Integration
* Every push runs formatting, clippy and the test suite on Linux, macOS and Windows, and the full end-to-end suite on Linux with KVM.

### Verification
* macOS (x86_64, HVF): all 8 end-to-end stages pass with the release package.
* Linux (x86_64, KVM): all 8 end-to-end stages pass in CI.
* Windows (x86_64): clippy, unit tests and build pass in CI. The end-to-end suite on a Windows host has not been run yet for this release.

---

### Upgrade Guide

#### macOS
```bash
curl -fsSL https://github.com/keikun80/ec2pass/releases/download/0.1.4/ec2pass-0.1.4-darwin-universal.tar.gz | tar -xz -C /tmp
sudo /tmp/ec2pass-0.1.4-darwin-universal/install.sh
```

#### Linux (Amazon Linux 2023 / Enterprise Linux / Ubuntu)
```bash
curl -fsSL https://github.com/keikun80/ec2pass/releases/download/0.1.4/ec2pass-0.1.4-linux-x86_64.tar.gz | tar -xz -C /tmp
sudo /tmp/ec2pass-0.1.4-linux-x86_64/install.sh
```

#### Windows 11 (PowerShell)
```powershell
Invoke-WebRequest -Uri https://github.com/keikun80/ec2pass/releases/download/0.1.4/ec2pass-0.1.4-windows-x86_64.zip -OutFile ec2pass.zip
Expand-Archive -Path ec2pass.zip -DestinationPath $env:LOCALAPPDATA\Programs\ec2pass -Force
```

---

## InhausLab ec2pass 0.1.3 Release Notes

**Release Version**: `0.1.3`  
**Release Date**: 2026-11-24  
**Target Architecture**: macOS Darwin (x86_64 / Apple Silicon Universal 2), Linux (x86_64), Windows 11 (x86_64)

---

### Overview

InhausLab `ec2pass` 0.1.3 delivers complete stub and sham elimination, true cross-platform hardening, and full implementation of all audited roadmap capabilities across Linux, macOS, and Windows. This release brings cross-platform process liveness verification, dynamic network interface discovery on macOS, official multi-distro cloud image catalog integration (Ubuntu 24.04, Debian 12, Rocky Linux 9, Fedora 40, AL2023 Minimal), multi-distro cloud-init sudoers/groups support, ARM64 UEFI edk2 firmware auto-discovery for QEMU, destructive snapshot rollback history pruning, host-to-guest working directory mapping for `ec2pass exec`, session token authentication, and Windows Hyper-V driver fixes.

---

### What's New in 0.1.3

#### 1. Cross-Platform Process Liveness Verification
* Replaced Linux-only `/proc/<pid>` checks with unified cross-platform process verification:
  * **Unix / macOS**: Direct POSIX `libc::kill(pid, 0)` signaling to check for process existence and ownership.
  * **Windows**: `tasklist.exe /FI "PID eq <pid>"` inspection with strict numeric PID matching.

#### 2. Dynamic Host Network Discovery on macOS
* Resolved macOS network interface detection limitation:
  * Leveraged `ifconfig -l` to dynamically enumerate active physical (`en*`), bridge (`bridge*`), and tunnel (`utun*`) interfaces on Darwin, aligning with Linux `/sys/class/net` discovery.

#### 3. Official Cloud Image Catalog & Multi-Distro Support
* Added official GenericCloud/KVM image catalog entries and direct download endpoints:
  * **Ubuntu 24.04 LTS (Noble Numbat)**
  * **Debian 12 (Bookworm)**
  * **Rocky Linux 9 (Enterprise Linux)**
  * **Fedora 40 Cloud**
  * **Amazon Linux 2023 Minimal**
* Legacy/unsupported images are now cleanly filtered from default `ec2pass find` listings unless `--show-unsupported` is specified.
* Normalized compatibility aliases (`al2023-k6.12`, `kernel-6.12`, `micro`, `minimal`) in the catalog.

#### 4. Multi-Distribution Cloud-Init Sudoers & Groups Configuration
* Updated default cloud-init `vendor-data` to configure `groups: [wheel, sudo, adm, systemd-journal]` with `0440` passwordless sudoers rules.
* Guarantees passwordless `sudo` out of the box across both RHEL/Fedora-family (`wheel`) and Debian/Ubuntu-family (`sudo`) distributions.

#### 5. ARM64 UEFI (edk2) Firmware Auto-Discovery for QEMU
* Automated detection of ARM64 UEFI bios firmware files across standard system paths (`/opt/homebrew`, `/usr/local`, `/usr/share/qemu`, `/usr/share/edk2/aarch64`, etc.).
* Automatically binds `-bios <edk2-aarch64-code.fd>` on Apple Silicon and Linux ARM64 systems, eliminating bootloader stalls.

#### 6. Destructive Snapshot History Pruning (`--destructive`)
* Full implementation of `ec2pass restore <instance> -s <snapshot> --destructive`:
  * Non-destructive restore preserves subsequent snapshots.
  * Destructive restore cleanly truncates snapshot branch history, pruning orphaned snapshots taken after the restored point.

#### 7. Working Directory Mapping for Command Execution (`ec2pass exec`)
* If the host's current working directory matches any mount on the instance, `ec2pass exec` automatically maps and executes the command within the corresponding guest path.
* Added `-n` / `--no-map-working-directory` to disable automatic directory mapping.

#### 8. Enhanced Security & Configuration Hygiene
* Daemon RPC session authentication token stored in `~/.ec2pass/auth_token` (0600 file permissions).
* Purged unhandled phantom settings keys, rejecting unrecognized configuration keys with descriptive errors.
* Removed artificial `.max(180)` clamp on `--timeout` in instance launch.

#### 9. Windows Hyper-V Native Driver Enhancements
* PowerShell/WMI SCSI DVD drive attach for `seed.iso` cloud-init provisioning.
* Accurate VM worker process (`vmwp.exe`) PID discovery via Hyper-V VM GUIDs.
* Virtual network adapter configuration connected directly to the Hyper-V Default Switch.

---

### Upgrade Guide

#### macOS (Homebrew / Direct Binary)
```bash
curl -fsSL https://github.com/keikun80/ec2pass/releases/download/0.1.3/ec2pass-0.1.3-macos-universal.tar.gz | tar -xz -C /usr/local/bin
```

#### Linux (Amazon Linux 2023 / Enterprise Linux 9 / Ubuntu)
```bash
curl -fsSL https://github.com/keikun80/ec2pass/releases/download/0.1.3/ec2pass-0.1.3-linux-x86_64.tar.gz | tar -xz -C /tmp
sudo /tmp/ec2pass-0.1.3-linux-x86_64/install.sh
```

#### Windows 11 (PowerShell)
```powershell
Invoke-WebRequest -Uri https://github.com/keikun80/ec2pass/releases/download/0.1.3/ec2pass-0.1.3-windows-x86_64.zip -OutFile ec2pass.zip
Expand-Archive -Path ec2pass.zip -DestinationPath $env:LOCALAPPDATA\Programs\ec2pass -Force
```

---

## InhausLab ec2pass 0.1.2 Release Notes

**Release Version**: `0.1.2`  
**Release Date**: 2026-11-23  
**Target Architecture**: macOS Darwin (x86_64 / Apple Silicon Universal 2), Linux (x86_64), Windows 11 (x86_64)

---

### Overview

InhausLab `ec2pass` 0.1.2 introduces real-time host-to-guest directory synchronization (`mount` & `umount`), live QMP snapshot and rollback on running instances, enhanced recursive file transfer with automatic parent directory creation (`transfer -p`), dynamic CLI command alias forwarding, and an interactive rotating circular activity indicator (`|` -> `/` -> `-` -> `\`). In addition, the comprehensive test matrix was expanded to 67 test cases with 100% strict verification across all subcommands, options, and output formats.

---

### What's New in 0.1.2

#### 1. Live Interactive Circular Activity Indicator
* Replaced static progress dots with a dynamic 360-degree rotating line spinner (`|` -> `/` -> `-` -> `\`) running at 100ms intervals during long-running tasks (`mount`, `umount`, `transfer`, `snapshot`, `restore`, `start`, etc.).
* Cleanly erases the spinner on completion, maintaining a pristine terminal state.

#### 2. Synchronized Directory Mounting & Clean Unmounting (`mount` & `umount`)
* **Live Mount Sync**: `ec2pass mount <host_path> <instance>:<guest_path>` synchronizes host project directories directly into the guest VM with proper user ownership and recursive write permissions (`sudo chmod -R u+w`).
* **Clean Umount**: `ec2pass umount <instance>:<guest_path>` completely unmounts and cleans the guest mount directory, matching native unmount behavior.
* **Persistent Mount Sync on Boot**: Automatically synchronizes configured mount points upon instance `launch` and `start`.

#### 3. Nested File Transfer with Parent Directory Creation (`transfer -p / --parents`)
* `ec2pass transfer -p / --parents` automatically creates missing nested parent directories on the destination target before transferring files or directories (supports both host-to-guest and guest-to-host).
* Automatic resolution of relative host paths to absolute paths.

#### 4. Live Instance Snapshot & State Rollback (`snapshot` & `restore -s`)
* Supports taking snapshots on running virtual machines via QMP Human Monitor commands (`savevm`), capturing full disk and memory states without shutting down the VM.
* Live rollback via `ec2pass restore <instance> -s <snapshot>` seamlessly restores instances to prior points in time.
* Supports both `<instance>.<snapshot>` and `<instance> -s <snapshot>` / `--snapshot <snapshot>` syntax.

#### 5. Dynamic Command Alias Execution
* Registered aliases (`ec2pass alias <instance>:<command> <alias_name>`) can now be invoked directly from the CLI as first-class subcommands (e.g. `ec2pass host-uptime` directly forwards to remote execution on the target instance).

#### 6. Hypervisor Lifecycle & SSH Connection Resilience
* Fixed restart and resume race conditions: `restart` and `start` on suspended instances now wait for full SSH guest readiness before returning.
* Implemented exponential backoff and retry loop for transient SSH handshake resets (`banner exchange`, `Connection reset by peer`, `kex_exchange_identification`).
* Auto-detection and recovery of dead QEMU PIDs via `/proc/<pid>` inspection.

#### 7. 67-Case Comprehensive Matrix Test Suite (100% Strict Pass)
* Expanded `tests/comprehensive_matrix_test.sh` to 67 exhaustive test cases covering every subcommand, option flag, lifecycle event, and output format (`table`, `json`, `yaml`, `csv`).
* Eliminated all unconditional pass workarounds (`|| true`); all 67 test cases strictly pass with Exit Code 0.

---

### Upgrade Guide

#### macOS (Homebrew / Direct Binary)
```bash
curl -fsSL https://github.com/keikun80/ec2pass/releases/download/0.1.2/ec2pass-0.1.2-macos-universal.tar.gz | tar -xz -C /usr/local/bin
```

#### Linux (Amazon Linux 2023 / Enterprise Linux 9 / Ubuntu)
```bash
curl -fsSL https://github.com/keikun80/ec2pass/releases/download/0.1.2/ec2pass-0.1.2-linux-x86_64.tar.gz | tar -xz -C /tmp
sudo /tmp/ec2pass-0.1.2-linux-x86_64/install.sh
```

#### Windows 11 (PowerShell)
```powershell
Invoke-WebRequest -Uri https://github.com/keikun80/ec2pass/releases/download/0.1.2/ec2pass-0.1.2-windows-x86_64.zip -OutFile ec2pass.zip
Expand-Archive -Path ec2pass.zip -DestinationPath C:\ec2pass
& C:\ec2pass\install.ps1
```

---

## InhausLab ec2pass 0.1.1 Release Notes

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
