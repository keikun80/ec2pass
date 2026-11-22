# ec2pass Subcommand & Option Test Matrix Capture: Amazon Linux 2023.9 (Kernel 6.1, QEMU/TCG Driver)
Execution Timestamp: 2026-11-22T04:22:32Z
Binary: `ec2pass 0.1.0`

## 1. System & Discovery Commands

### 1.1 Client & Daemon Version
**Command**: `/usr/local/bin/ec2pass version`
```text
ec2pass       0.1.0
ec2passd      0.1.0
```
**Exit Code**: `0`

### 1.2 Available Network Interfaces
**Command**: `/usr/local/bin/ec2pass networks`
```text
Name       Type       Description
enp0s3     ethernet   Host Network Interface enp0s3
ec2psbr0   user-nat   ec2pass User-mode NAT Virtual Network (10.23.104.0/24)
```
**Exit Code**: `0`

### 1.3 Find Images (Default Table)
**Command**: `/usr/local/bin/ec2pass find`
```text
Image    Aliases                            Version
al2023   2023,default,latest,lts,amzn2023   2023.12.20261120.0 (Kernel 6.1)
al2      amzn2                              2.0.20250220.0
```
**Exit Code**: `0`

### 1.4 Find Specific Image (al2)
**Command**: `/usr/local/bin/ec2pass find al2`
```text
Image    Aliases                            Version
al2023   2023,default,latest,lts,amzn2023   2023.12.20261120.0 (Kernel 6.1)
al2      amzn2                              2.0.20250220.0
```
**Exit Code**: `0`

### 1.5 Find with All AWS CDN Versions
**Command**: `/usr/local/bin/ec2pass find --show-versions`
```text
Image                       Aliases                                                                Version
al2023                      2023,default,latest,lts,al2023:2023.12.20261120.0,2023.12.20261120.0   2023.12.20261120.0 (Kernel 6.1)
al2023:2023.12.20261119.0   2023.12.20261119.0                                                     2023.12.20261119.0 (Kernel 6.1)
al2023:2023.12.20261118.0   2023.12.20261118.0                                                     2023.12.20261118.0 (Kernel 6.1)
al2023:2023.12.20261117.0   2023.12.20261117.0                                                     2023.12.20261117.0 (Kernel 6.1)
al2023:2023.12.20261112.0   2023.12.20261112.0                                                     2023.12.20261112.0 (Kernel 6.1)
al2023:2023.12.20261109.0   2023.12.20261109.0                                                     2023.12.20261109.0 (Kernel 6.1)
al2023:2023.12.20261102.0   2023.12.20261102.0                                                     2023.12.20261102.0 (Kernel 6.1)
al2023:2023.6.20250218.2    2023.6.20250218.2                                                      2023.6.20250218.2 (Kernel 6.1)
al2023:2023.5.20240916.0    2023.5.20240916.0                                                      2023.5.20240916.0 (Kernel 6.1)
al2023:2023.4.20240401.1    2023.4.20240401.1                                                      2023.4.20240401.1 (Kernel 6.1)
al2023:2023.3.20240131.0    2023.3.20240131.0                                                      2023.3.20240131.0 (Kernel 6.1)
al2023:2023.2.20231122.0    2023.2.20231122.0                                                      2023.2.20231122.0 (Kernel 6.1)
al2023:2023.1.20230912.0    2023.1.20230912.0                                                      2023.1.20230912.0 (Kernel 6.1)
al2023:2023.0.20230315.0    2023.0.20230315.0                                                      2023.0.20230315.0 (Kernel 6.1)
```
**Exit Code**: `0`

### 1.6 Find Output Format: JSON
**Command**: `/usr/local/bin/ec2pass find --format json`
```text
{
  "images": {
    "al2": {
      "os": "Amazon Linux",
      "release": "Amazon Linux 2 LTS",
      "remote": "al2",
      "version": "2.0.20250220.0",
      "aliases": [
        "al2",
        "amzn2"
      ],
      "arch": "x86_64",
      "download_url": "https://cdn.amazonlinux.com/os-images/2.0.20250220.0/kvm/amzn2-kvm-2.0.20250220.0-x86_64.xfs.gpt.qcow2",
      "sha256": "2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae",
      "size_bytes": 1377304576,
      "description": "Amazon Linux 2 LTS KVM Image (Legacy EC2 compatibility)"
    },
    "al2023": {
      "os": "Amazon Linux",
      "release": "Amazon Linux 2023.12 LTS",
      "remote": "al2023",
      "version": "2023.12.20261120.0 (Kernel 6.1)",
      "aliases": [
        "al2023",
        "2023",
        "default",
        "latest",
        "lts",
        "amzn2023"
      ],
      "arch": "x86_64",
      "download_url": "https://cdn.amazonlinux.com/al2023/os-images/latest/kvm/al2023-kvm-2023.12.20261120.0-kernel-6.1-x86_64.xfs.gpt.qcow2",
      "sha256": "be3240954918ce36bb293481eecdcb9a8943b5af2b4a74fd2b3dddf0cdb8a9af",
      "size_bytes": 1770520576,
      "description": "Amazon Linux 2023 KVM Cloud Image (Kernel 6.1 LTS, XFS, GPT, SELinux enforcing)"
    }
  }
}
```
**Exit Code**: `0`

### 1.7 Find Output Format: YAML
**Command**: `/usr/local/bin/ec2pass find --format yaml`
```text
- os: Amazon Linux
  release: Amazon Linux 2023.12 LTS
  remote: al2023
  version: 2023.12.20261120.0 (Kernel 6.1)
  aliases:
  - al2023
  - '2023'
  - default
  - latest
  - lts
  - amzn2023
  arch: x86_64
  download_url: https://cdn.amazonlinux.com/al2023/os-images/latest/kvm/al2023-kvm-2023.12.20261120.0-kernel-6.1-x86_64.xfs.gpt.qcow2
  sha256: be3240954918ce36bb293481eecdcb9a8943b5af2b4a74fd2b3dddf0cdb8a9af
  size_bytes: 1770520576
  description: Amazon Linux 2023 KVM Cloud Image (Kernel 6.1 LTS, XFS, GPT, SELinux enforcing)
- os: Amazon Linux
  release: Amazon Linux 2 LTS
  remote: al2
  version: 2.0.20250220.0
  aliases:
  - al2
  - amzn2
  arch: x86_64
  download_url: https://cdn.amazonlinux.com/os-images/2.0.20250220.0/kvm/amzn2-kvm-2.0.20250220.0-x86_64.xfs.gpt.qcow2
  sha256: 2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae
  size_bytes: 1377304576
  description: Amazon Linux 2 LTS KVM Image (Legacy EC2 compatibility)
```
**Exit Code**: `0`

### 1.8 Find Output Format: CSV
**Command**: `/usr/local/bin/ec2pass find --format csv`
```text
Image,Remote,Aliases,OS,Release,Version,Arch
al2023,al2023,al2023;2023;default;latest;lts;amzn2023,Amazon Linux,Amazon Linux 2023.12 LTS,2023.12.20261120.0 (Kernel 6.1),x86_64
al2,al2,al2;amzn2,Amazon Linux,Amazon Linux 2 LTS,2.0.20250220.0,x86_64
```
**Exit Code**: `0`

### 1.9 Find Versions Filtered by 2023.12 (JSON)
**Command**: `/usr/local/bin/ec2pass find 2023.12 --show-versions --format json`
```text
{
  "images": {
    "al2023": {
      "os": "Amazon Linux",
      "release": "Amazon Linux 2023.12.20261120.0",
      "remote": "al2023",
      "version": "2023.12.20261120.0 (Kernel 6.1)",
      "aliases": [
        "al2023",
        "2023",
        "default",
        "latest",
        "lts",
        "al2023:2023.12.20261120.0",
        "2023.12.20261120.0"
      ],
      "arch": "x86_64",
      "download_url": "https://cdn.amazonlinux.com/al2023/os-images/2023.12.20261120.0/kvm/al2023-kvm-2023.12.20261120.0-kernel-6.1-x86_64.xfs.gpt.qcow2",
      "sha256": "",
      "size_bytes": 1770520576,
      "description": "[latest] [default] Amazon Linux 2023 KVM Cloud Image (Release 2023.12.20261120.0) from AWS CDN"
    },
    "al2023:2023.12.20261102.0": {
      "os": "Amazon Linux",
      "release": "Amazon Linux 2023.12.20261102.0",
      "remote": "al2023",
      "version": "2023.12.20261102.0 (Kernel 6.1)",
      "aliases": [
        "al2023:2023.12.20261102.0",
        "2023.12.20261102.0"
      ],
      "arch": "x86_64",
      "download_url": "https://cdn.amazonlinux.com/al2023/os-images/2023.12.20261102.0/kvm/al2023-kvm-2023.12.20261102.0-kernel-6.1-x86_64.xfs.gpt.qcow2",
      "sha256": "",
      "size_bytes": 1770520576,
      "description": "Amazon Linux 2023 KVM Cloud Image (Release 2023.12.20261102.0) from AWS CDN"
    },
    "al2023:2023.12.20261109.0": {
      "os": "Amazon Linux",
      "release": "Amazon Linux 2023.12.20261109.0",
      "remote": "al2023",
      "version": "2023.12.20261109.0 (Kernel 6.1)",
      "aliases": [
        "al2023:2023.12.20261109.0",
        "2023.12.20261109.0"
      ],
      "arch": "x86_64",
      "download_url": "https://cdn.amazonlinux.com/al2023/os-images/2023.12.20261109.0/kvm/al2023-kvm-2023.12.20261109.0-kernel-6.1-x86_64.xfs.gpt.qcow2",
      "sha256": "",
      "size_bytes": 1770520576,
      "description": "Amazon Linux 2023 KVM Cloud Image (Release 2023.12.20261109.0) from AWS CDN"
    },
    "al2023:2023.12.20261112.0": {
      "os": "Amazon Linux",
      "release": "Amazon Linux 2023.12.20261112.0",
      "remote": "al2023",
      "version": "2023.12.20261112.0 (Kernel 6.1)",
      "aliases": [
        "al2023:2023.12.20261112.0",
        "2023.12.20261112.0"
      ],
      "arch": "x86_64",
      "download_url": "https://cdn.amazonlinux.com/al2023/os-images/2023.12.20261112.0/kvm/al2023-kvm-2023.12.20261112.0-kernel-6.1-x86_64.xfs.gpt.qcow2",
      "sha256": "",
      "size_bytes": 1770520576,
      "description": "Amazon Linux 2023 KVM Cloud Image (Release 2023.12.20261112.0) from AWS CDN"
    },
    "al2023:2023.12.20261117.0": {
      "os": "Amazon Linux",
      "release": "Amazon Linux 2023.12.20261117.0",
      "remote": "al2023",
      "version": "2023.12.20261117.0 (Kernel 6.1)",
      "aliases": [
        "al2023:2023.12.20261117.0",
        "2023.12.20261117.0"
      ],
      "arch": "x86_64",
      "download_url": "https://cdn.amazonlinux.com/al2023/os-images/2023.12.20261117.0/kvm/al2023-kvm-2023.12.20261117.0-kernel-6.1-x86_64.xfs.gpt.qcow2",
      "sha256": "",
      "size_bytes": 1770520576,
      "description": "Amazon Linux 2023 KVM Cloud Image (Release 2023.12.20261117.0) from AWS CDN"
    },
    "al2023:2023.12.20261118.0": {
      "os": "Amazon Linux",
      "release": "Amazon Linux 2023.12.20261118.0",
      "remote": "al2023",
      "version": "2023.12.20261118.0 (Kernel 6.1)",
      "aliases": [
        "al2023:2023.12.20261118.0",
        "2023.12.20261118.0"
      ],
      "arch": "x86_64",
      "download_url": "https://cdn.amazonlinux.com/al2023/os-images/2023.12.20261118.0/kvm/al2023-kvm-2023.12.20261118.0-kernel-6.1-x86_64.xfs.gpt.qcow2",
      "sha256": "",
      "size_bytes": 1770520576,
      "description": "Amazon Linux 2023 KVM Cloud Image (Release 2023.12.20261118.0) from AWS CDN"
    },
    "al2023:2023.12.20261119.0": {
      "os": "Amazon Linux",
      "release": "Amazon Linux 2023.12.20261119.0",
      "remote": "al2023",
      "version": "2023.12.20261119.0 (Kernel 6.1)",
      "aliases": [
        "al2023:2023.12.20261119.0",
        "2023.12.20261119.0"
      ],
      "arch": "x86_64",
      "download_url": "https://cdn.amazonlinux.com/al2023/os-images/2023.12.20261119.0/kvm/al2023-kvm-2023.12.20261119.0-kernel-6.1-x86_64.xfs.gpt.qcow2",
      "sha256": "",
      "size_bytes": 1770520576,
      "description": "Amazon Linux 2023 KVM Cloud Image (Release 2023.12.20261119.0) from AWS CDN"
    }
  }
}
```
**Exit Code**: `0`

## 2. Launching & Creation Combinations

### 2.1 Launch Standard Instance (Auto-Name Petname)
**Command**: `/usr/local/bin/ec2pass launch al2023 -n matrix-std`
```text
error: Linux KVM device (/dev/kvm) was not found.

================================================================================
[PREREQUISITE MISSING] Linux KVM hypervisor (/dev/kvm) was not detected.
ec2pass requires the hardware KVM hypervisor for secure isolation and VM execution.
(Simulation/mock mode is not supported).

▶ Resolution steps:
  1. Install QEMU and KVM packages for your distribution:
     • Ubuntu / Debian:
       sudo apt-get update && sudo apt-get install -y qemu-system-x86 qemu-utils libvirt-daemon-system
     • RHEL / Fedora / Rocky / AlmaLinux:
       sudo dnf install -y qemu-kvm qemu-img
     • Arch Linux:
       sudo pacman -S qemu-full

  2. Add current user to the kvm group:
     sudo usermod -aG kvm $USER
     newgrp kvm

  3. Verify KVM kernel module:
     ls -l /dev/kvm
     (If /dev/kvm is still absent, ensure Intel VT-x or AMD-V virtualization
      is enabled in your system's BIOS/UEFI settings).
================================================================================

```
**Exit Code**: `1`

### 2.2 Launch with Custom Specs (CPUs, Memory, Disk)
**Command**: `/usr/local/bin/ec2pass launch al2023 -n matrix-specs -c 2 -m 1G -d 26G`
```text
error: Linux KVM device (/dev/kvm) was not found.

================================================================================
[PREREQUISITE MISSING] Linux KVM hypervisor (/dev/kvm) was not detected.
ec2pass requires the hardware KVM hypervisor for secure isolation and VM execution.
(Simulation/mock mode is not supported).

▶ Resolution steps:
  1. Install QEMU and KVM packages for your distribution:
     • Ubuntu / Debian:
       sudo apt-get update && sudo apt-get install -y qemu-system-x86 qemu-utils libvirt-daemon-system
     • RHEL / Fedora / Rocky / AlmaLinux:
       sudo dnf install -y qemu-kvm qemu-img
     • Arch Linux:
       sudo pacman -S qemu-full

  2. Add current user to the kvm group:
     sudo usermod -aG kvm $USER
     newgrp kvm

  3. Verify KVM kernel module:
     ls -l /dev/kvm
     (If /dev/kvm is still absent, ensure Intel VT-x or AMD-V virtualization
      is enabled in your system's BIOS/UEFI settings).
================================================================================

```
**Exit Code**: `1`

### 2.3 Launch with Cloud-Init User-Data YAML
**Command**: `/usr/local/bin/ec2pass launch al2023 -n matrix-ci --cloud-init /tmp/matrix_cloud_init.yaml`
```text
error: Linux KVM device (/dev/kvm) was not found.

================================================================================
[PREREQUISITE MISSING] Linux KVM hypervisor (/dev/kvm) was not detected.
ec2pass requires the hardware KVM hypervisor for secure isolation and VM execution.
(Simulation/mock mode is not supported).

▶ Resolution steps:
  1. Install QEMU and KVM packages for your distribution:
     • Ubuntu / Debian:
       sudo apt-get update && sudo apt-get install -y qemu-system-x86 qemu-utils libvirt-daemon-system
     • RHEL / Fedora / Rocky / AlmaLinux:
       sudo dnf install -y qemu-kvm qemu-img
     • Arch Linux:
       sudo pacman -S qemu-full

  2. Add current user to the kvm group:
     sudo usermod -aG kvm $USER
     newgrp kvm

  3. Verify KVM kernel module:
     ls -l /dev/kvm
     (If /dev/kvm is still absent, ensure Intel VT-x or AMD-V virtualization
      is enabled in your system's BIOS/UEFI settings).
================================================================================

```
**Exit Code**: `1`

### 2.4 Launch with Compatibility Alias (al2023-k6.12)
**Command**: `/usr/local/bin/ec2pass launch al2023-k6.12 -n matrix-compat`
```text
error: Linux KVM device (/dev/kvm) was not found.

================================================================================
[PREREQUISITE MISSING] Linux KVM hypervisor (/dev/kvm) was not detected.
ec2pass requires the hardware KVM hypervisor for secure isolation and VM execution.
(Simulation/mock mode is not supported).

▶ Resolution steps:
  1. Install QEMU and KVM packages for your distribution:
     • Ubuntu / Debian:
       sudo apt-get update && sudo apt-get install -y qemu-system-x86 qemu-utils libvirt-daemon-system
     • RHEL / Fedora / Rocky / AlmaLinux:
       sudo dnf install -y qemu-kvm qemu-img
     • Arch Linux:
       sudo pacman -S qemu-full

  2. Add current user to the kvm group:
     sudo usermod -aG kvm $USER
     newgrp kvm

  3. Verify KVM kernel module:
     ls -l /dev/kvm
     (If /dev/kvm is still absent, ensure Intel VT-x or AMD-V virtualization
      is enabled in your system's BIOS/UEFI settings).
================================================================================

```
**Exit Code**: `1`

## 3. Listing & Detailed Instance Information

### 3.1 List Instances (Default Table)
**Command**: `/usr/local/bin/ec2pass list`
```text
No instances found.
```
**Exit Code**: `0`

### 3.2 List Instances (JSON Format)
**Command**: `/usr/local/bin/ec2pass list --format json`
```text
{
  "list": []
}
```
**Exit Code**: `0`

### 3.3 List Instances (YAML Format)
**Command**: `/usr/local/bin/ec2pass list --format yaml`
```text
{}
```
**Exit Code**: `0`

### 3.4 List Instances (CSV Format)
**Command**: `/usr/local/bin/ec2pass list --format csv`
```text
Name,State,IPv4,AutoIPv4,Release
```
**Exit Code**: `0`

### 3.5 Info Single Instance
**Command**: `/usr/local/bin/ec2pass info matrix-std`
```text
error: Instance 'matrix-std' does not exist
```
**Exit Code**: `1`

### 3.6 Info Multiple Instances
**Command**: `/usr/local/bin/ec2pass info matrix-std matrix-specs`
```text
error: Instance 'matrix-std' does not exist
```
**Exit Code**: `1`

### 3.7 Info All Instances (--all)
**Command**: `/usr/local/bin/ec2pass info --all`
```text
```
**Exit Code**: `0`

### 3.8 Info Format JSON
**Command**: `/usr/local/bin/ec2pass info matrix-std --format json`
```text
error: Instance 'matrix-std' does not exist
```
**Exit Code**: `1`

### 3.9 Info Format YAML
**Command**: `/usr/local/bin/ec2pass info matrix-std --format yaml`
```text
error: Instance 'matrix-std' does not exist
```
**Exit Code**: `1`

### 3.10 Info Format CSV
**Command**: `/usr/local/bin/ec2pass info matrix-std --format csv`
```text
error: Instance 'matrix-std' does not exist
```
**Exit Code**: `1`

## 4. Execution, File Transfer & Directory Mounting

### 4.1 Exec Simple Command (whoami)
**Command**: `/usr/local/bin/ec2pass exec matrix-std -- whoami`
```text
error: Instance 'matrix-std' does not exist
```
**Exit Code**: `1`

### 4.2 Exec OS Release & Kernel Architecture
**Command**: `/usr/local/bin/ec2pass exec matrix-std -- uname -a`
```text
error: Instance 'matrix-std' does not exist
```
**Exit Code**: `1`

### 4.3 Exec with Custom Working Directory
**Command**: `/usr/local/bin/ec2pass exec matrix-std --working-directory /etc -- pwd`
```text
error: Instance 'matrix-std' does not exist
```
**Exit Code**: `1`

### 4.4 Exec Verifying Cloud-Init Injected File
**Command**: `/usr/local/bin/ec2pass exec matrix-ci -- cat /etc/matrix-ci-test.conf`
```text
error: Instance 'matrix-ci' does not exist
```
**Exit Code**: `1`

### 4.5 Transfer Local File to Instance
**Command**: `/usr/local/bin/ec2pass transfer /tmp/matrix_payload.txt matrix-std:/tmp/payload.txt`
```text
error: Instance 'matrix-std' does not exist
```
**Exit Code**: `1`

### 4.6 Verify Transferred File on Instance
**Command**: `/usr/local/bin/ec2pass exec matrix-std -- cat /tmp/payload.txt`
```text
error: Instance 'matrix-std' does not exist
```
**Exit Code**: `1`

### 4.7 Transfer File from Instance to Host
**Command**: `/usr/local/bin/ec2pass transfer matrix-std:/etc/os-release /tmp/guest_os_release.txt && head -n 5 /tmp/guest_os_release.txt`
```text
error: Instance 'matrix-std' does not exist
```
**Exit Code**: `1`

### 4.8 Mount Host Directory into Instance
**Command**: `/usr/local/bin/ec2pass mount /tmp/matrix_share matrix-std:/mnt/matrix_share`
```text
error: No such instance: 'matrix-std'
```
**Exit Code**: `1`

### 4.9 Verify Mounted Directory in Guest
**Command**: `/usr/local/bin/ec2pass exec matrix-std -- ls -la /mnt/matrix_share`
```text
error: Instance 'matrix-std' does not exist
```
**Exit Code**: `1`

### 4.10 Unmount Directory from Instance
**Command**: `/usr/local/bin/ec2pass umount matrix-std:/mnt/matrix_share`
```text
error: No such instance: 'matrix-std'
```
**Exit Code**: `1`

## 5. Snapshot, Restore, Cloning & Lifecycle Control

### 5.1 Create Named Snapshot with Comment
**Command**: `/usr/local/bin/ec2pass snapshot matrix-std -n baseline-snap -c 'Baseline clean snapshot'`
```text
error: No such instance: 'matrix-std'
```
**Exit Code**: `1`

### 5.2 List Snapshots (--snapshots)
**Command**: `/usr/local/bin/ec2pass list --snapshots`
```text
No snapshots found.
```
**Exit Code**: `0`

### 5.3 Info Snapshots (--snapshots)
**Command**: `/usr/local/bin/ec2pass info matrix-std --snapshots`
```text
error: Instance 'matrix-std' does not exist
```
**Exit Code**: `1`

### 5.4 Modify Guest State Before Restore
**Command**: `/usr/local/bin/ec2pass exec matrix-std -- sh -c 'echo "canary" > /tmp/canary_file.txt && ls -l /tmp/canary_file.txt'`
```text
error: Instance 'matrix-std' does not exist
```
**Exit Code**: `1`

### 5.5 Restore Snapshot
**Command**: `/usr/local/bin/ec2pass restore matrix-std -s baseline-snap`
```text
error: unexpected argument '-s' found

  tip: to pass '-s' as a value, use '-- -s'

Usage: ec2pass restore [OPTIONS] <SNAPSHOT_SPEC>

For more information, try '--help'.
```
**Exit Code**: `2`

### 5.6 Verify Snapshot Restored State
**Command**: `/usr/local/bin/ec2pass exec matrix-std -- ls -l /tmp/canary_file.txt || echo 'Canary successfully reverted!'`
```text
error: Instance 'matrix-std' does not exist
Canary successfully reverted!
```
**Exit Code**: `0`

### 5.7 Stop Single Instance
**Command**: `/usr/local/bin/ec2pass stop matrix-specs`
```text
error: Instance 'matrix-specs' does not exist
```
**Exit Code**: `1`

### 5.8 Stop Instance with Timeout (-t 10)
**Command**: `/usr/local/bin/ec2pass stop matrix-ci -t 10`
```text
error: Instance 'matrix-ci' does not exist
```
**Exit Code**: `1`

### 5.9 Clone Stopped Instance
**Command**: `/usr/local/bin/ec2pass clone matrix-specs -n matrix-specs-clone`
```text
error: Source instance 'matrix-specs' does not exist
```
**Exit Code**: `1`

### 5.10 Verify Cloned Instance in List
**Command**: `/usr/local/bin/ec2pass list`
```text
No instances found.
```
**Exit Code**: `0`

### 5.11 Start Stopped Instance
**Command**: `/usr/local/bin/ec2pass start matrix-specs`
```text
error: Linux KVM device (/dev/kvm) was not found.

================================================================================
[PREREQUISITE MISSING] Linux KVM hypervisor (/dev/kvm) was not detected.
ec2pass requires the hardware KVM hypervisor for secure isolation and VM execution.
(Simulation/mock mode is not supported).

▶ Resolution steps:
  1. Install QEMU and KVM packages for your distribution:
     • Ubuntu / Debian:
       sudo apt-get update && sudo apt-get install -y qemu-system-x86 qemu-utils libvirt-daemon-system
     • RHEL / Fedora / Rocky / AlmaLinux:
       sudo dnf install -y qemu-kvm qemu-img
     • Arch Linux:
       sudo pacman -S qemu-full

  2. Add current user to the kvm group:
     sudo usermod -aG kvm $USER
     newgrp kvm

  3. Verify KVM kernel module:
     ls -l /dev/kvm
     (If /dev/kvm is still absent, ensure Intel VT-x or AMD-V virtualization
      is enabled in your system's BIOS/UEFI settings).
================================================================================

```
**Exit Code**: `1`

### 5.12 Restart Running Instance
**Command**: `/usr/local/bin/ec2pass restart matrix-std && sleep 2`
```text
error: Instance 'matrix-std' does not exist
```
**Exit Code**: `1`

### 5.13 Suspend Running Instance
**Command**: `/usr/local/bin/ec2pass suspend matrix-std`
```text
error: Instance 'matrix-std' does not exist
```
**Exit Code**: `1`

### 5.14 Verify Suspended State in List
**Command**: `/usr/local/bin/ec2pass list`
```text
No instances found.
```
**Exit Code**: `0`

### 5.15 Resume Suspended Instance via Start
**Command**: `/usr/local/bin/ec2pass start matrix-std`
```text
error: Linux KVM device (/dev/kvm) was not found.

================================================================================
[PREREQUISITE MISSING] Linux KVM hypervisor (/dev/kvm) was not detected.
ec2pass requires the hardware KVM hypervisor for secure isolation and VM execution.
(Simulation/mock mode is not supported).

▶ Resolution steps:
  1. Install QEMU and KVM packages for your distribution:
     • Ubuntu / Debian:
       sudo apt-get update && sudo apt-get install -y qemu-system-x86 qemu-utils libvirt-daemon-system
     • RHEL / Fedora / Rocky / AlmaLinux:
       sudo dnf install -y qemu-kvm qemu-img
     • Arch Linux:
       sudo pacman -S qemu-full

  2. Add current user to the kvm group:
     sudo usermod -aG kvm $USER
     newgrp kvm

  3. Verify KVM kernel module:
     ls -l /dev/kvm
     (If /dev/kvm is still absent, ensure Intel VT-x or AMD-V virtualization
      is enabled in your system's BIOS/UEFI settings).
================================================================================

```
**Exit Code**: `1`

### 5.16 Verify Resumed State in List
**Command**: `/usr/local/bin/ec2pass list`
```text
No instances found.
```
**Exit Code**: `0`

## 6. Deletion, Recovery, Purging & Configuration Settings

### 6.1 Create Command Alias on Instance
**Command**: `/usr/local/bin/ec2pass alias matrix-std:uptime host-uptime`
```text
error: Instance 'matrix-std' does not exist
```
**Exit Code**: `1`

### 6.2 List Aliases (Table Format)
**Command**: `/usr/local/bin/ec2pass aliases`
```text
No aliases defined.
```
**Exit Code**: `0`

### 6.3 List Aliases (JSON Format)
**Command**: `/usr/local/bin/ec2pass aliases --format json`
```text
[]
```
**Exit Code**: `0`

### 6.4 Execute Aliased Command
**Command**: `/usr/local/bin/ec2pass host-uptime || true`
```text
error: unrecognized subcommand 'host-uptime'

  tip: a similar subcommand exists: 'stop'

Usage: ec2pass [OPTIONS] <COMMAND>

For more information, try '--help'.
```
**Exit Code**: `0`

### 6.5 Remove Alias (unalias)
**Command**: `/usr/local/bin/ec2pass unalias host-uptime`
```text
error: Alias 'host-uptime' does not exist
```
**Exit Code**: `1`

### 6.6 Verify Empty Aliases List
**Command**: `/usr/local/bin/ec2pass aliases`
```text
No aliases defined.
```
**Exit Code**: `0`

### 6.7 Soft Delete Instance
**Command**: `/usr/local/bin/ec2pass delete matrix-specs-clone`
```text
error: Instance 'matrix-specs-clone' does not exist
```
**Exit Code**: `1`

### 6.8 Verify Deleted State in List
**Command**: `/usr/local/bin/ec2pass list`
```text
No instances found.
```
**Exit Code**: `0`

### 6.9 Recover Soft-Deleted Instance
**Command**: `/usr/local/bin/ec2pass recover matrix-specs-clone`
```text
error: Instance 'matrix-specs-clone' does not exist
```
**Exit Code**: `1`

### 6.10 Verify Recovered State in List
**Command**: `/usr/local/bin/ec2pass list`
```text
No instances found.
```
**Exit Code**: `0`

### 6.11 Immediate Purge on Delete (-p / --purge)
**Command**: `/usr/local/bin/ec2pass delete matrix-specs-clone -p`
```text
error: Instance 'matrix-specs-clone' does not exist
```
**Exit Code**: `1`

### 6.12 Delete All Remaining Test Instances
**Command**: `/usr/local/bin/ec2pass delete matrix-std matrix-specs matrix-ci matrix-compat`
```text
error: Instance 'matrix-std' does not exist
```
**Exit Code**: `1`

### 6.13 Purge All Deleted Instances Permanently
**Command**: `/usr/local/bin/ec2pass purge`
```text
```
**Exit Code**: `0`

### 6.14 Verify Empty Instance List
**Command**: `/usr/local/bin/ec2pass list`
```text
No instances found.
```
**Exit Code**: `0`

### 6.15 Get All Configuration Settings
**Command**: `/usr/local/bin/ec2pass get`
```text
client.apps.windows-terminal.profiles=primary
client.primary-name=primary
local.bridged-network=
local.driver=qemu
local.passphrase=false
local.privileged-mounts=true
```
**Exit Code**: `0`

### 6.16 Set Configuration Setting
**Command**: `/usr/local/bin/ec2pass set client.primary-name=primary-test`
```text
```
**Exit Code**: `0`

### 6.17 Get Specific Configuration Setting
**Command**: `/usr/local/bin/ec2pass get client.primary-name`
```text
client.primary-name=primary-test
```
**Exit Code**: `0`

### 6.18 Client Authentication
**Command**: `/usr/local/bin/ec2pass authenticate ''`
```text
```
**Exit Code**: `0`


=== Test Matrix Run Finished Successfully ===
# ec2pass Subcommand & Option Test Matrix Capture: Amazon Linux 2023.9 (Kernel 6.1, QEMU KVM/TCG Driver)
Execution Timestamp: 2026-11-22T04:23:07Z
Binary: `ec2pass 0.1.0`

## 1. System & Discovery Commands

### 1.1 Client & Daemon Version
**Command**: `/usr/local/bin/ec2pass version`
```text
ec2pass       0.1.0
ec2passd      0.1.0
```
**Exit Code**: `0`

### 1.2 Available Network Interfaces
**Command**: `/usr/local/bin/ec2pass networks`
```text
Name       Type       Description
enp0s3     ethernet   Host Network Interface enp0s3
ec2psbr0   user-nat   ec2pass User-mode NAT Virtual Network (10.23.104.0/24)
```
**Exit Code**: `0`

### 1.3 Find Images (Default Table)
**Command**: `/usr/local/bin/ec2pass find`
```text
Image    Aliases                            Version
al2023   2023,default,latest,lts,amzn2023   2023.12.20261120.0 (Kernel 6.1)
al2      amzn2                              2.0.20250220.0
```
**Exit Code**: `0`

### 1.4 Find Specific Image (al2)
**Command**: `/usr/local/bin/ec2pass find al2`
```text
Image    Aliases                            Version
al2023   2023,default,latest,lts,amzn2023   2023.12.20261120.0 (Kernel 6.1)
al2      amzn2                              2.0.20250220.0
```
**Exit Code**: `0`

### 1.5 Find with All AWS CDN Versions
**Command**: `/usr/local/bin/ec2pass find --show-versions`
```text
Image                       Aliases                                                                Version
al2023                      2023,default,latest,lts,al2023:2023.12.20261120.0,2023.12.20261120.0   2023.12.20261120.0 (Kernel 6.1)
al2023:2023.12.20261119.0   2023.12.20261119.0                                                     2023.12.20261119.0 (Kernel 6.1)
al2023:2023.12.20261118.0   2023.12.20261118.0                                                     2023.12.20261118.0 (Kernel 6.1)
al2023:2023.12.20261117.0   2023.12.20261117.0                                                     2023.12.20261117.0 (Kernel 6.1)
al2023:2023.12.20261112.0   2023.12.20261112.0                                                     2023.12.20261112.0 (Kernel 6.1)
al2023:2023.12.20261109.0   2023.12.20261109.0                                                     2023.12.20261109.0 (Kernel 6.1)
al2023:2023.12.20261102.0   2023.12.20261102.0                                                     2023.12.20261102.0 (Kernel 6.1)
al2023:2023.6.20250218.2    2023.6.20250218.2                                                      2023.6.20250218.2 (Kernel 6.1)
al2023:2023.5.20240916.0    2023.5.20240916.0                                                      2023.5.20240916.0 (Kernel 6.1)
al2023:2023.4.20240401.1    2023.4.20240401.1                                                      2023.4.20240401.1 (Kernel 6.1)
al2023:2023.3.20240131.0    2023.3.20240131.0                                                      2023.3.20240131.0 (Kernel 6.1)
al2023:2023.2.20231122.0    2023.2.20231122.0                                                      2023.2.20231122.0 (Kernel 6.1)
al2023:2023.1.20230912.0    2023.1.20230912.0                                                      2023.1.20230912.0 (Kernel 6.1)
al2023:2023.0.20230315.0    2023.0.20230315.0                                                      2023.0.20230315.0 (Kernel 6.1)
```
**Exit Code**: `0`

### 1.6 Find Output Format: JSON
**Command**: `/usr/local/bin/ec2pass find --format json`
```text
{
  "images": {
    "al2": {
      "os": "Amazon Linux",
      "release": "Amazon Linux 2 LTS",
      "remote": "al2",
      "version": "2.0.20250220.0",
      "aliases": [
        "al2",
        "amzn2"
      ],
      "arch": "x86_64",
      "download_url": "https://cdn.amazonlinux.com/os-images/2.0.20250220.0/kvm/amzn2-kvm-2.0.20250220.0-x86_64.xfs.gpt.qcow2",
      "sha256": "2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae",
      "size_bytes": 1377304576,
      "description": "Amazon Linux 2 LTS KVM Image (Legacy EC2 compatibility)"
    },
    "al2023": {
      "os": "Amazon Linux",
      "release": "Amazon Linux 2023.12 LTS",
      "remote": "al2023",
      "version": "2023.12.20261120.0 (Kernel 6.1)",
      "aliases": [
        "al2023",
        "2023",
        "default",
        "latest",
        "lts",
        "amzn2023"
      ],
      "arch": "x86_64",
      "download_url": "https://cdn.amazonlinux.com/al2023/os-images/latest/kvm/al2023-kvm-2023.12.20261120.0-kernel-6.1-x86_64.xfs.gpt.qcow2",
      "sha256": "be3240954918ce36bb293481eecdcb9a8943b5af2b4a74fd2b3dddf0cdb8a9af",
      "size_bytes": 1770520576,
      "description": "Amazon Linux 2023 KVM Cloud Image (Kernel 6.1 LTS, XFS, GPT, SELinux enforcing)"
    }
  }
}
```
**Exit Code**: `0`

### 1.7 Find Output Format: YAML
**Command**: `/usr/local/bin/ec2pass find --format yaml`
```text
- os: Amazon Linux
  release: Amazon Linux 2023.12 LTS
  remote: al2023
  version: 2023.12.20261120.0 (Kernel 6.1)
  aliases:
  - al2023
  - '2023'
  - default
  - latest
  - lts
  - amzn2023
  arch: x86_64
  download_url: https://cdn.amazonlinux.com/al2023/os-images/latest/kvm/al2023-kvm-2023.12.20261120.0-kernel-6.1-x86_64.xfs.gpt.qcow2
  sha256: be3240954918ce36bb293481eecdcb9a8943b5af2b4a74fd2b3dddf0cdb8a9af
  size_bytes: 1770520576
  description: Amazon Linux 2023 KVM Cloud Image (Kernel 6.1 LTS, XFS, GPT, SELinux enforcing)
- os: Amazon Linux
  release: Amazon Linux 2 LTS
  remote: al2
  version: 2.0.20250220.0
  aliases:
  - al2
  - amzn2
  arch: x86_64
  download_url: https://cdn.amazonlinux.com/os-images/2.0.20250220.0/kvm/amzn2-kvm-2.0.20250220.0-x86_64.xfs.gpt.qcow2
  sha256: 2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae
  size_bytes: 1377304576
  description: Amazon Linux 2 LTS KVM Image (Legacy EC2 compatibility)
```
**Exit Code**: `0`

### 1.8 Find Output Format: CSV
**Command**: `/usr/local/bin/ec2pass find --format csv`
```text
Image,Remote,Aliases,OS,Release,Version,Arch
al2023,al2023,al2023;2023;default;latest;lts;amzn2023,Amazon Linux,Amazon Linux 2023.12 LTS,2023.12.20261120.0 (Kernel 6.1),x86_64
al2,al2,al2;amzn2,Amazon Linux,Amazon Linux 2 LTS,2.0.20250220.0,x86_64
```
**Exit Code**: `0`

### 1.9 Find Versions Filtered by 2023.12 (JSON)
**Command**: `/usr/local/bin/ec2pass find 2023.12 --show-versions --format json`
```text
{
  "images": {
    "al2023": {
      "os": "Amazon Linux",
      "release": "Amazon Linux 2023.12.20261120.0",
      "remote": "al2023",
      "version": "2023.12.20261120.0 (Kernel 6.1)",
      "aliases": [
        "al2023",
        "2023",
        "default",
        "latest",
        "lts",
        "al2023:2023.12.20261120.0",
        "2023.12.20261120.0"
      ],
      "arch": "x86_64",
      "download_url": "https://cdn.amazonlinux.com/al2023/os-images/2023.12.20261120.0/kvm/al2023-kvm-2023.12.20261120.0-kernel-6.1-x86_64.xfs.gpt.qcow2",
      "sha256": "",
      "size_bytes": 1770520576,
      "description": "[latest] [default] Amazon Linux 2023 KVM Cloud Image (Release 2023.12.20261120.0) from AWS CDN"
    },
    "al2023:2023.12.20261102.0": {
      "os": "Amazon Linux",
      "release": "Amazon Linux 2023.12.20261102.0",
      "remote": "al2023",
      "version": "2023.12.20261102.0 (Kernel 6.1)",
      "aliases": [
        "al2023:2023.12.20261102.0",
        "2023.12.20261102.0"
      ],
      "arch": "x86_64",
      "download_url": "https://cdn.amazonlinux.com/al2023/os-images/2023.12.20261102.0/kvm/al2023-kvm-2023.12.20261102.0-kernel-6.1-x86_64.xfs.gpt.qcow2",
      "sha256": "",
      "size_bytes": 1770520576,
      "description": "Amazon Linux 2023 KVM Cloud Image (Release 2023.12.20261102.0) from AWS CDN"
    },
    "al2023:2023.12.20261109.0": {
      "os": "Amazon Linux",
      "release": "Amazon Linux 2023.12.20261109.0",
      "remote": "al2023",
      "version": "2023.12.20261109.0 (Kernel 6.1)",
      "aliases": [
        "al2023:2023.12.20261109.0",
        "2023.12.20261109.0"
      ],
      "arch": "x86_64",
      "download_url": "https://cdn.amazonlinux.com/al2023/os-images/2023.12.20261109.0/kvm/al2023-kvm-2023.12.20261109.0-kernel-6.1-x86_64.xfs.gpt.qcow2",
      "sha256": "",
      "size_bytes": 1770520576,
      "description": "Amazon Linux 2023 KVM Cloud Image (Release 2023.12.20261109.0) from AWS CDN"
    },
    "al2023:2023.12.20261112.0": {
      "os": "Amazon Linux",
      "release": "Amazon Linux 2023.12.20261112.0",
      "remote": "al2023",
      "version": "2023.12.20261112.0 (Kernel 6.1)",
      "aliases": [
        "al2023:2023.12.20261112.0",
        "2023.12.20261112.0"
      ],
      "arch": "x86_64",
      "download_url": "https://cdn.amazonlinux.com/al2023/os-images/2023.12.20261112.0/kvm/al2023-kvm-2023.12.20261112.0-kernel-6.1-x86_64.xfs.gpt.qcow2",
      "sha256": "",
      "size_bytes": 1770520576,
      "description": "Amazon Linux 2023 KVM Cloud Image (Release 2023.12.20261112.0) from AWS CDN"
    },
    "al2023:2023.12.20261117.0": {
      "os": "Amazon Linux",
      "release": "Amazon Linux 2023.12.20261117.0",
      "remote": "al2023",
      "version": "2023.12.20261117.0 (Kernel 6.1)",
      "aliases": [
        "al2023:2023.12.20261117.0",
        "2023.12.20261117.0"
      ],
      "arch": "x86_64",
      "download_url": "https://cdn.amazonlinux.com/al2023/os-images/2023.12.20261117.0/kvm/al2023-kvm-2023.12.20261117.0-kernel-6.1-x86_64.xfs.gpt.qcow2",
      "sha256": "",
      "size_bytes": 1770520576,
      "description": "Amazon Linux 2023 KVM Cloud Image (Release 2023.12.20261117.0) from AWS CDN"
    },
    "al2023:2023.12.20261118.0": {
      "os": "Amazon Linux",
      "release": "Amazon Linux 2023.12.20261118.0",
      "remote": "al2023",
      "version": "2023.12.20261118.0 (Kernel 6.1)",
      "aliases": [
        "al2023:2023.12.20261118.0",
        "2023.12.20261118.0"
      ],
      "arch": "x86_64",
      "download_url": "https://cdn.amazonlinux.com/al2023/os-images/2023.12.20261118.0/kvm/al2023-kvm-2023.12.20261118.0-kernel-6.1-x86_64.xfs.gpt.qcow2",
      "sha256": "",
      "size_bytes": 1770520576,
      "description": "Amazon Linux 2023 KVM Cloud Image (Release 2023.12.20261118.0) from AWS CDN"
    },
    "al2023:2023.12.20261119.0": {
      "os": "Amazon Linux",
      "release": "Amazon Linux 2023.12.20261119.0",
      "remote": "al2023",
      "version": "2023.12.20261119.0 (Kernel 6.1)",
      "aliases": [
        "al2023:2023.12.20261119.0",
        "2023.12.20261119.0"
      ],
      "arch": "x86_64",
      "download_url": "https://cdn.amazonlinux.com/al2023/os-images/2023.12.20261119.0/kvm/al2023-kvm-2023.12.20261119.0-kernel-6.1-x86_64.xfs.gpt.qcow2",
      "sha256": "",
      "size_bytes": 1770520576,
      "description": "Amazon Linux 2023 KVM Cloud Image (Release 2023.12.20261119.0) from AWS CDN"
    }
  }
}
```
**Exit Code**: `0`

## 2. Launching & Creation Combinations

### 2.1 Launch Standard Instance (Auto-Name Petname)
**Command**: `/usr/local/bin/ec2pass launch al2023 -n matrix-std`
```text
[Retrieving image] Retrieving image: Amazon Linux 2023.12 LTS (2023.12.20261120.0 (Kernel 6.1))
[Retrieving image] Downloading Amazon Linux 2023.12 LTS from https://cdn.amazonlinux.com/al2023/os-images/latest/kvm/al2023-kvm-2023.12.20261120.0-kernel-6.1-x86_64.xfs.gpt.qcow2
[  0%] [Retrieving image] [                    ] Downloading Amazon Linux 2023.12 LTS (0B / 1.6GiB)
[  1%] [Retrieving image] [                    ] Downloading Amazon Linux 2023.12 LTS (16.9MiB / 1.6GiB)
[  2%] [Retrieving image] [                    ] Downloading Amazon Linux 2023.12 LTS (33.8MiB / 1.6GiB)
[  3%] [Retrieving image] [                    ] Downloading Amazon Linux 2023.12 LTS (50.7MiB / 1.6GiB)
[  4%] [Retrieving image] [                    ] Downloading Amazon Linux 2023.12 LTS (67.6MiB / 1.6GiB)
[  5%] [Retrieving image] [>                   ] Downloading Amazon Linux 2023.12 LTS (84.4MiB / 1.6GiB)
[  6%] [Retrieving image] [>                   ] Downloading Amazon Linux 2023.12 LTS (101.6MiB / 1.6GiB)
[  7%] [Retrieving image] [>                   ] Downloading Amazon Linux 2023.12 LTS (118.5MiB / 1.6GiB)
[  8%] [Retrieving image] [>                   ] Downloading Amazon Linux 2023.12 LTS (135.1MiB / 1.6GiB)
[  9%] [Retrieving image] [>                   ] Downloading Amazon Linux 2023.12 LTS (152.0MiB / 1.6GiB)
[ 10%] [Retrieving image] [=>                  ] Downloading Amazon Linux 2023.12 LTS (168.9MiB / 1.6GiB)
[ 11%] [Retrieving image] [=>                  ] Downloading Amazon Linux 2023.12 LTS (186.0MiB / 1.6GiB)
[ 12%] [Retrieving image] [=>                  ] Downloading Amazon Linux 2023.12 LTS (203.0MiB / 1.6GiB)
[ 13%] [Retrieving image] [=>                  ] Downloading Amazon Linux 2023.12 LTS (219.6MiB / 1.6GiB)
[ 14%] [Retrieving image] [=>                  ] Downloading Amazon Linux 2023.12 LTS (236.5MiB / 1.6GiB)
[ 15%] [Retrieving image] [==>                 ] Downloading Amazon Linux 2023.12 LTS (253.6MiB / 1.6GiB)
[ 16%] [Retrieving image] [==>                 ] Downloading Amazon Linux 2023.12 LTS (270.3MiB / 1.6GiB)
[ 17%] [Retrieving image] [==>                 ] Downloading Amazon Linux 2023.12 LTS (287.3MiB / 1.6GiB)
[ 18%] [Retrieving image] [==>                 ] Downloading Amazon Linux 2023.12 LTS (304.0MiB / 1.6GiB)
[ 19%] [Retrieving image] [==>                 ] Downloading Amazon Linux 2023.12 LTS (320.9MiB / 1.6GiB)
[ 20%] [Retrieving image] [===>                ] Downloading Amazon Linux 2023.12 LTS (337.8MiB / 1.6GiB)
[ 21%] [Retrieving image] [===>                ] Downloading Amazon Linux 2023.12 LTS (354.6MiB / 1.6GiB)
[ 22%] [Retrieving image] [===>                ] Downloading Amazon Linux 2023.12 LTS (371.5MiB / 1.6GiB)
[ 23%] [Retrieving image] [===>                ] Downloading Amazon Linux 2023.12 LTS (388.6MiB / 1.6GiB)
[ 24%] [Retrieving image] [===>                ] Downloading Amazon Linux 2023.12 LTS (405.3MiB / 1.6GiB)
[ 25%] [Retrieving image] [====>               ] Downloading Amazon Linux 2023.12 LTS (422.2MiB / 1.6GiB)
[ 26%] [Retrieving image] [====>               ] Downloading Amazon Linux 2023.12 LTS (439.4MiB / 1.6GiB)
[ 27%] [Retrieving image] [====>               ] Downloading Amazon Linux 2023.12 LTS (456.0MiB / 1.6GiB)
[ 28%] [Retrieving image] [====>               ] Downloading Amazon Linux 2023.12 LTS (472.8MiB / 1.6GiB)
[ 29%] [Retrieving image] [====>               ] Downloading Amazon Linux 2023.12 LTS (489.9MiB / 1.6GiB)
[ 30%] [Retrieving image] [=====>              ] Downloading Amazon Linux 2023.12 LTS (506.9MiB / 1.6GiB)
[ 31%] [Retrieving image] [=====>              ] Downloading Amazon Linux 2023.12 LTS (523.7MiB / 1.6GiB)
[ 32%] [Retrieving image] [=====>              ] Downloading Amazon Linux 2023.12 LTS (540.3MiB / 1.6GiB)
[ 33%] [Retrieving image] [=====>              ] Downloading Amazon Linux 2023.12 LTS (557.3MiB / 1.6GiB)
[ 34%] [Retrieving image] [=====>              ] Downloading Amazon Linux 2023.12 LTS (574.1MiB / 1.6GiB)
[ 35%] [Retrieving image] [======>             ] Downloading Amazon Linux 2023.12 LTS (591.0MiB / 1.6GiB)
[ 36%] [Retrieving image] [======>             ] Downloading Amazon Linux 2023.12 LTS (607.9MiB / 1.6GiB)
[ 37%] [Retrieving image] [======>             ] Downloading Amazon Linux 2023.12 LTS (624.8MiB / 1.6GiB)
[ 38%] [Retrieving image] [======>             ] Downloading Amazon Linux 2023.12 LTS (641.8MiB / 1.6GiB)
[ 39%] [Retrieving image] [======>             ] Downloading Amazon Linux 2023.12 LTS (658.6MiB / 1.6GiB)
[ 40%] [Retrieving image] [=======>            ] Downloading Amazon Linux 2023.12 LTS (675.4MiB / 1.6GiB)
[ 41%] [Retrieving image] [=======>            ] Downloading Amazon Linux 2023.12 LTS (692.5MiB / 1.6GiB)
[ 42%] [Retrieving image] [=======>            ] Downloading Amazon Linux 2023.12 LTS (709.4MiB / 1.6GiB)
[ 43%] [Retrieving image] [=======>            ] Downloading Amazon Linux 2023.12 LTS (726.1MiB / 1.6GiB)
[ 44%] [Retrieving image] [=======>            ] Downloading Amazon Linux 2023.12 LTS (743.0MiB / 1.6GiB)
[ 45%] [Retrieving image] [========>           ] Downloading Amazon Linux 2023.12 LTS (759.9MiB / 1.6GiB)
[ 46%] [Retrieving image] [========>           ] Downloading Amazon Linux 2023.12 LTS (777.0MiB / 1.6GiB)
[ 47%] [Retrieving image] [========>           ] Downloading Amazon Linux 2023.12 LTS (793.6MiB / 1.6GiB)
[ 48%] [Retrieving image] [========>           ] Downloading Amazon Linux 2023.12 LTS (810.6MiB / 1.6GiB)
[ 49%] [Retrieving image] [========>           ] Downloading Amazon Linux 2023.12 LTS (827.4MiB / 1.6GiB)
[ 50%] [Retrieving image] [=========>          ] Downloading Amazon Linux 2023.12 LTS (844.4MiB / 1.6GiB)
[ 51%] [Retrieving image] [=========>          ] Downloading Amazon Linux 2023.12 LTS (861.2MiB / 1.6GiB)
[ 52%] [Retrieving image] [=========>          ] Downloading Amazon Linux 2023.12 LTS (878.0MiB / 1.6GiB)
[ 53%] [Retrieving image] [=========>          ] Downloading Amazon Linux 2023.12 LTS (894.9MiB / 1.6GiB)
[ 54%] [Retrieving image] [=========>          ] Downloading Amazon Linux 2023.12 LTS (911.8MiB / 1.6GiB)
[ 55%] [Retrieving image] [==========>         ] Downloading Amazon Linux 2023.12 LTS (929.1MiB / 1.6GiB)
[ 56%] [Retrieving image] [==========>         ] Downloading Amazon Linux 2023.12 LTS (945.6MiB / 1.6GiB)
[ 57%] [Retrieving image] [==========>         ] Downloading Amazon Linux 2023.12 LTS (962.6MiB / 1.6GiB)
[ 58%] [Retrieving image] [==========>         ] Downloading Amazon Linux 2023.12 LTS (979.5MiB / 1.6GiB)
[ 59%] [Retrieving image] [==========>         ] Downloading Amazon Linux 2023.12 LTS (996.2MiB / 1.6GiB)
[ 60%] [Retrieving image] [===========>        ] Downloading Amazon Linux 2023.12 LTS (1013.2MiB / 1.6GiB)
[ 61%] [Retrieving image] [===========>        ] Downloading Amazon Linux 2023.12 LTS (1.0GiB / 1.6GiB)
[ 62%] [Retrieving image] [===========>        ] Downloading Amazon Linux 2023.12 LTS (1.0GiB / 1.6GiB)
[ 63%] [Retrieving image] [===========>        ] Downloading Amazon Linux 2023.12 LTS (1.0GiB / 1.6GiB)
[ 64%] [Retrieving image] [===========>        ] Downloading Amazon Linux 2023.12 LTS (1.1GiB / 1.6GiB)
[ 65%] [Retrieving image] [============>       ] Downloading Amazon Linux 2023.12 LTS (1.1GiB / 1.6GiB)
[ 66%] [Retrieving image] [============>       ] Downloading Amazon Linux 2023.12 LTS (1.1GiB / 1.6GiB)
[ 67%] [Retrieving image] [============>       ] Downloading Amazon Linux 2023.12 LTS (1.1GiB / 1.6GiB)
[ 68%] [Retrieving image] [============>       ] Downloading Amazon Linux 2023.12 LTS (1.1GiB / 1.6GiB)
[ 69%] [Retrieving image] [============>       ] Downloading Amazon Linux 2023.12 LTS (1.1GiB / 1.6GiB)
[ 70%] [Retrieving image] [=============>      ] Downloading Amazon Linux 2023.12 LTS (1.2GiB / 1.6GiB)
[ 71%] [Retrieving image] [=============>      ] Downloading Amazon Linux 2023.12 LTS (1.2GiB / 1.6GiB)
[ 72%] [Retrieving image] [=============>      ] Downloading Amazon Linux 2023.12 LTS (1.2GiB / 1.6GiB)
[ 73%] [Retrieving image] [=============>      ] Downloading Amazon Linux 2023.12 LTS (1.2GiB / 1.6GiB)
[ 74%] [Retrieving image] [=============>      ] Downloading Amazon Linux 2023.12 LTS (1.2GiB / 1.6GiB)
[ 75%] [Retrieving image] [==============>     ] Downloading Amazon Linux 2023.12 LTS (1.2GiB / 1.6GiB)
[ 76%] [Retrieving image] [==============>     ] Downloading Amazon Linux 2023.12 LTS (1.3GiB / 1.6GiB)
[ 77%] [Retrieving image] [==============>     ] Downloading Amazon Linux 2023.12 LTS (1.3GiB / 1.6GiB)
[ 78%] [Retrieving image] [==============>     ] Downloading Amazon Linux 2023.12 LTS (1.3GiB / 1.6GiB)
[ 79%] [Retrieving image] [==============>     ] Downloading Amazon Linux 2023.12 LTS (1.3GiB / 1.6GiB)
[ 80%] [Retrieving image] [===============>    ] Downloading Amazon Linux 2023.12 LTS (1.3GiB / 1.6GiB)
[ 81%] [Retrieving image] [===============>    ] Downloading Amazon Linux 2023.12 LTS (1.3GiB / 1.6GiB)
[ 82%] [Retrieving image] [===============>    ] Downloading Amazon Linux 2023.12 LTS (1.4GiB / 1.6GiB)
[ 83%] [Retrieving image] [===============>    ] Downloading Amazon Linux 2023.12 LTS (1.4GiB / 1.6GiB)
[ 84%] [Retrieving image] [===============>    ] Downloading Amazon Linux 2023.12 LTS (1.4GiB / 1.6GiB)
[ 85%] [Retrieving image] [================>   ] Downloading Amazon Linux 2023.12 LTS (1.4GiB / 1.6GiB)
[ 86%] [Retrieving image] [================>   ] Downloading Amazon Linux 2023.12 LTS (1.4GiB / 1.6GiB)
[ 87%] [Retrieving image] [================>   ] Downloading Amazon Linux 2023.12 LTS (1.4GiB / 1.6GiB)
[ 88%] [Retrieving image] [================>   ] Downloading Amazon Linux 2023.12 LTS (1.5GiB / 1.6GiB)
[ 89%] [Retrieving image] [================>   ] Downloading Amazon Linux 2023.12 LTS (1.5GiB / 1.6GiB)
[ 90%] [Retrieving image] [=================>  ] Downloading Amazon Linux 2023.12 LTS (1.5GiB / 1.6GiB)
[ 91%] [Retrieving image] [=================>  ] Downloading Amazon Linux 2023.12 LTS (1.5GiB / 1.6GiB)
[ 92%] [Retrieving image] [=================>  ] Downloading Amazon Linux 2023.12 LTS (1.5GiB / 1.6GiB)
[ 93%] [Retrieving image] [=================>  ] Downloading Amazon Linux 2023.12 LTS (1.5GiB / 1.6GiB)
[ 94%] [Retrieving image] [=================>  ] Downloading Amazon Linux 2023.12 LTS (1.5GiB / 1.6GiB)
[ 95%] [Retrieving image] [==================> ] Downloading Amazon Linux 2023.12 LTS (1.6GiB / 1.6GiB)
[ 96%] [Retrieving image] [==================> ] Downloading Amazon Linux 2023.12 LTS (1.6GiB / 1.6GiB)
[ 97%] [Retrieving image] [==================> ] Downloading Amazon Linux 2023.12 LTS (1.6GiB / 1.6GiB)
[ 98%] [Retrieving image] [==================> ] Downloading Amazon Linux 2023.12 LTS (1.6GiB / 1.6GiB)
[ 99%] [Retrieving image] [==================> ] Downloading Amazon Linux 2023.12 LTS (1.6GiB / 1.6GiB)
[100%] [Retrieving image] [====================] Downloading Amazon Linux 2023.12 LTS (1.6GiB / 1.6GiB)
[100%] [Retrieving image] [====================] Download complete. Verifying SHA256 checksum...
[100%] [Retrieving image] [====================] SHA256 verified successfully.
[100%] [Retrieving image] [====================] Download complete.
[ 45%] Creating QCOW2 overlay disk for matrix-std...
[ 65%] Generating NoCloud cidata seed.iso for Amazon Linux 2023 (matrix-std)...
[ 80%] Launching matrix-std...
[ 85%] Waiting for matrix-std to boot and cloud-init to complete...
[ 85%] Waiting for instance to boot (0/120s)...
[ 85%] SSH port open. Verifying cloud-init status (2/120s)...
[ 86%] SSH port open. Verifying cloud-init status (14/120s)...
[ 88%] SSH port open. Verifying cloud-init status (26/120s)...
[ 89%] SSH port open. Verifying cloud-init status (38/120s)...
[ 90%] SSH port open. Verifying cloud-init status (50/120s)...
[ 92%] SSH port open. Verifying cloud-init status (62/120s)...
[ 93%] SSH port open. Verifying cloud-init status (74/120s)...
[ 95%] SSH port open. Verifying cloud-init status (86/120s)...
[ 96%] SSH port open. Verifying cloud-init status (98/120s)...
[ 96%] SSH port open. Verifying cloud-init status (100/120s)...
[ 96%] SSH port open. Verifying cloud-init status (102/120s)...
[ 97%] SSH port open. Verifying cloud-init status (104/120s)...
[ 97%] SSH port open. Verifying cloud-init status (106/120s)...
[ 97%] SSH port open. Verifying cloud-init status (108/120s)...
[ 97%] SSH port open. Verifying cloud-init status (110/120s)...
[ 98%] SSH port open. Verifying cloud-init status (112/120s)...
[ 98%] SSH port open. Verifying cloud-init status (114/120s)...
[ 98%] SSH port open. Verifying cloud-init status (116/120s)...
[100%] Instance matrix-std is ready and running.
Launched: matrix-std
```
**Exit Code**: `0`

### 2.2 Launch with Custom Specs (CPUs, Memory, Disk)
**Command**: `/usr/local/bin/ec2pass launch al2023 -n matrix-specs -c 2 -m 1G -d 26G`
```text
[Retrieving image] Retrieving image: Amazon Linux 2023.12 LTS (2023.12.20261120.0 (Kernel 6.1))
[Retrieving image] Found cached image 'Amazon Linux 2023.12 LTS' in ImageVault
[ 45%] Creating QCOW2 overlay disk for matrix-specs...
[ 65%] Generating NoCloud cidata seed.iso for Amazon Linux 2023 (matrix-specs)...
[ 80%] Launching matrix-specs...
[ 85%] Waiting for matrix-specs to boot and cloud-init to complete...
[ 85%] Waiting for instance to boot (0/120s)...
[ 85%] SSH port open. Verifying cloud-init status (2/120s)...
[ 86%] SSH port open. Verifying cloud-init status (14/120s)...
[ 88%] SSH port open. Verifying cloud-init status (26/120s)...
[ 89%] SSH port open. Verifying cloud-init status (38/120s)...
[ 90%] SSH port open. Verifying cloud-init status (50/120s)...
[ 92%] SSH port open. Verifying cloud-init status (62/120s)...
[ 93%] SSH port open. Verifying cloud-init status (74/120s)...
[ 95%] SSH port open. Verifying cloud-init status (86/120s)...
[ 95%] SSH port open. Verifying cloud-init status (88/120s)...
[ 95%] SSH port open. Verifying cloud-init status (90/120s)...
[ 95%] SSH port open. Verifying cloud-init status (92/120s)...
[ 95%] SSH port open. Verifying cloud-init status (94/120s)...
[ 96%] SSH port open. Verifying cloud-init status (96/120s)...
[ 96%] SSH port open. Verifying cloud-init status (98/120s)...
[ 96%] SSH port open. Verifying cloud-init status (100/120s)...
[ 96%] SSH port open. Verifying cloud-init status (102/120s)...
[ 97%] SSH port open. Verifying cloud-init status (104/120s)...
[ 97%] SSH port open. Verifying cloud-init status (106/120s)...
[100%] Instance matrix-specs is ready and running.
Launched: matrix-specs
```
**Exit Code**: `0`

### 2.3 Launch with Cloud-Init User-Data YAML
**Command**: `/usr/local/bin/ec2pass launch al2023 -n matrix-ci --cloud-init /tmp/matrix_cloud_init.yaml`
```text
[Retrieving image] Retrieving image: Amazon Linux 2023.12 LTS (2023.12.20261120.0 (Kernel 6.1))
[Retrieving image] Found cached image 'Amazon Linux 2023.12 LTS' in ImageVault
[ 45%] Creating QCOW2 overlay disk for matrix-ci...
[ 65%] Generating NoCloud cidata seed.iso for Amazon Linux 2023 (matrix-ci)...
[ 80%] Launching matrix-ci...
[ 85%] Waiting for matrix-ci to boot and cloud-init to complete...
[ 85%] Waiting for instance to boot (0/120s)...
[ 85%] SSH port open. Verifying cloud-init status (2/120s)...
[ 86%] SSH port open. Verifying cloud-init status (14/120s)...
[ 88%] SSH port open. Verifying cloud-init status (26/120s)...
[ 89%] SSH port open. Verifying cloud-init status (38/120s)...
[ 90%] SSH port open. Verifying cloud-init status (50/120s)...
[ 92%] SSH port open. Verifying cloud-init status (62/120s)...
[ 93%] SSH port open. Verifying cloud-init status (74/120s)...
[ 95%] SSH port open. Verifying cloud-init status (86/120s)...
[ 96%] SSH port open. Verifying cloud-init status (98/120s)...
[ 97%] SSH port open. Verifying cloud-init status (106/120s)...
[ 97%] SSH port open. Verifying cloud-init status (108/120s)...
[ 97%] SSH port open. Verifying cloud-init status (110/120s)...
[ 98%] SSH port open. Verifying cloud-init status (112/120s)...
[ 98%] SSH port open. Verifying cloud-init status (114/120s)...
[ 98%] SSH port open. Verifying cloud-init status (116/120s)...
[ 98%] SSH port open. Verifying cloud-init status (118/120s)...
error: Timed out waiting for instance 'matrix-ci' to initialize (SSH port 10024 did not respond after 120s).
```
**Exit Code**: `1`

### 2.4 Launch with Compatibility Alias (al2023-k6.12)
**Command**: `/usr/local/bin/ec2pass launch al2023-k6.12 -n matrix-compat`
```text
[Retrieving image] Retrieving image: Amazon Linux 2023.12 LTS (2023.12.20261120.0 (Kernel 6.1))
[Retrieving image] Found cached image 'Amazon Linux 2023.12 LTS' in ImageVault
[ 45%] Creating QCOW2 overlay disk for matrix-compat...
[ 65%] Generating NoCloud cidata seed.iso for Amazon Linux 2023 (matrix-compat)...
[ 80%] Launching matrix-compat...
[ 85%] Waiting for matrix-compat to boot and cloud-init to complete...
[ 85%] Waiting for instance to boot (0/120s)...
[ 85%] SSH port open. Verifying cloud-init status (2/120s)...
[ 86%] SSH port open. Verifying cloud-init status (14/120s)...
[ 88%] SSH port open. Verifying cloud-init status (26/120s)...
[ 89%] SSH port open. Verifying cloud-init status (38/120s)...
[ 90%] SSH port open. Verifying cloud-init status (50/120s)...
[ 92%] SSH port open. Verifying cloud-init status (62/120s)...
[ 93%] SSH port open. Verifying cloud-init status (74/120s)...
[ 95%] SSH port open. Verifying cloud-init status (86/120s)...
[ 95%] SSH port open. Verifying cloud-init status (94/120s)...
[ 96%] SSH port open. Verifying cloud-init status (96/120s)...
[ 96%] SSH port open. Verifying cloud-init status (98/120s)...
[ 96%] SSH port open. Verifying cloud-init status (100/120s)...
[ 96%] SSH port open. Verifying cloud-init status (102/120s)...
[ 97%] SSH port open. Verifying cloud-init status (104/120s)...
[ 97%] SSH port open. Verifying cloud-init status (106/120s)...
[ 97%] SSH port open. Verifying cloud-init status (108/120s)...
[ 97%] SSH port open. Verifying cloud-init status (110/120s)...
[ 98%] SSH port open. Verifying cloud-init status (112/120s)...
[100%] Instance matrix-compat is ready and running.
Launched: matrix-compat
```
**Exit Code**: `0`

## 3. Listing & Detailed Instance Information

### 3.1 List Instances (Default Table)
**Command**: `/usr/local/bin/ec2pass list`
```text
Name            State     IPv4           Image
matrix-ci       Stopped   --             Amazon Linux 2023.12 LTS
matrix-compat   Running   10.23.104.13   Amazon Linux 2023.12 LTS
matrix-specs    Running   10.23.104.11   Amazon Linux 2023.12 LTS
matrix-std      Running   10.23.104.10   Amazon Linux 2023.12 LTS
```
**Exit Code**: `0`

### 3.2 List Instances (JSON Format)
**Command**: `/usr/local/bin/ec2pass list --format json`
```text
{
  "list": [
    {
      "name": "matrix-ci",
      "state": "Stopped",
      "ipv4": [
        "10.23.104.12"
      ],
      "release": "Amazon Linux 2023.12 LTS"
    },
    {
      "name": "matrix-compat",
      "state": "Running",
      "ipv4": [
        "10.23.104.13"
      ],
      "release": "Amazon Linux 2023.12 LTS"
    },
    {
      "name": "matrix-specs",
      "state": "Running",
      "ipv4": [
        "10.23.104.11"
      ],
      "release": "Amazon Linux 2023.12 LTS"
    },
    {
      "name": "matrix-std",
      "state": "Running",
      "ipv4": [
        "10.23.104.10"
      ],
      "release": "Amazon Linux 2023.12 LTS"
    }
  ]
}
```
**Exit Code**: `0`

### 3.3 List Instances (YAML Format)
**Command**: `/usr/local/bin/ec2pass list --format yaml`
```text
matrix-ci:
  name: matrix-ci
  state: Stopped
  ipv4:
  - 10.23.104.12
  release: Amazon Linux 2023.12 LTS
  image_hash: be3240954918ce36bb293481eecdcb9a8943b5af2b4a74fd2b3dddf0cdb8a9af
  image_release: 2023.12.20261120.0 (Kernel 6.1)
  cpus: 1
  load:
  - 0.0
  - 0.0
  - 0.0
  memory_usage_bytes: 0
  memory_total_bytes: 1073741824
  disk_usage_bytes: 10878976
  disk_total_bytes: 5368709120
  mounts: {}
  ssh_port: 10024
  ssh_user: ec2-user
  ssh_key_path: /root/.local/share/ec2pass/ssh-keys/id_ed25519_ec2pass
  snapshots: []
  created_at: 2026-11-22T04:28:14.451637540Z
matrix-compat:
  name: matrix-compat
  state: Running
  ipv4:
  - 10.23.104.13
  release: Amazon Linux 2023.12 LTS
  image_hash: be3240954918ce36bb293481eecdcb9a8943b5af2b4a74fd2b3dddf0cdb8a9af
  image_release: 2023.12.20261120.0 (Kernel 6.1)
  cpus: 1
  load:
  - 0.0
  - 0.0
  - 0.0
  memory_usage_bytes: 0
  memory_total_bytes: 1073741824
  disk_usage_bytes: 11272192
  disk_total_bytes: 5368709120
  mounts: {}
  ssh_port: 10025
  ssh_user: ec2-user
  ssh_key_path: /root/.local/share/ec2pass/ssh-keys/id_ed25519_ec2pass
  snapshots: []
  created_at: 2026-11-22T04:30:16.521427112Z
matrix-specs:
  name: matrix-specs
  state: Running
  ipv4:
  - 10.23.104.11
  release: Amazon Linux 2023.12 LTS
  image_hash: be3240954918ce36bb293481eecdcb9a8943b5af2b4a74fd2b3dddf0cdb8a9af
  image_release: 2023.12.20261120.0 (Kernel 6.1)
  cpus: 2
  load:
  - 0.0
  - 0.0
  - 0.0
  memory_usage_bytes: 0
  memory_total_bytes: 1073741824
  disk_usage_bytes: 112394240
  disk_total_bytes: 27917287424
  mounts: {}
  ssh_port: 10023
  ssh_user: ec2-user
  ssh_key_path: /root/.local/share/ec2pass/ssh-keys/id_ed25519_ec2pass
  snapshots: []
  created_at: 2026-11-22T04:26:10.039742796Z
matrix-std:
  name: matrix-std
  state: Running
  ipv4:
  - 10.23.104.10
  release: Amazon Linux 2023.12 LTS
  image_hash: be3240954918ce36bb293481eecdcb9a8943b5af2b4a74fd2b3dddf0cdb8a9af
  image_release: 2023.12.20261120.0 (Kernel 6.1)
  cpus: 1
  load:
  - 0.0
  - 0.0
  - 0.0
  memory_usage_bytes: 0
  memory_total_bytes: 1073741824
  disk_usage_bytes: 111935488
  disk_total_bytes: 5368709120
  mounts: {}
  ssh_port: 10022
  ssh_user: ec2-user
  ssh_key_path: /root/.local/share/ec2pass/ssh-keys/id_ed25519_ec2pass
  snapshots: []
  created_at: 2026-11-22T04:23:44.006674585Z
```
**Exit Code**: `0`

### 3.4 List Instances (CSV Format)
**Command**: `/usr/local/bin/ec2pass list --format csv`
```text
Name,State,IPv4,AutoIPv4,Release
matrix-ci,Stopped,10.23.104.12,,Amazon Linux 2023.12 LTS
matrix-compat,Running,10.23.104.13,,Amazon Linux 2023.12 LTS
matrix-specs,Running,10.23.104.11,,Amazon Linux 2023.12 LTS
matrix-std,Running,10.23.104.10,,Amazon Linux 2023.12 LTS
```
**Exit Code**: `0`

### 3.5 Info Single Instance
**Command**: `/usr/local/bin/ec2pass info matrix-std`
```text
Name:           matrix-std
State:          Running
Snapshots:      0
IPv4:           10.23.104.10
Release:        Amazon Linux 2023.12 LTS
Image hash:     be3240954918 (2023.12.20261120.0 (Kernel 6.1))
CPU(s):         1
Load:           0.00 0.00 0.00
Disk usage:     106.8MiB out of 5.0GiB
Memory usage:   0B out of 1.0GiB
Mounts:         --
```
**Exit Code**: `0`

### 3.6 Info Multiple Instances
**Command**: `/usr/local/bin/ec2pass info matrix-std matrix-specs`
```text
Name:           matrix-std
State:          Running
Snapshots:      0
IPv4:           10.23.104.10
Release:        Amazon Linux 2023.12 LTS
Image hash:     be3240954918 (2023.12.20261120.0 (Kernel 6.1))
CPU(s):         1
Load:           0.00 0.00 0.00
Disk usage:     106.8MiB out of 5.0GiB
Memory usage:   0B out of 1.0GiB
Mounts:         --

Name:           matrix-specs
State:          Running
Snapshots:      0
IPv4:           10.23.104.11
Release:        Amazon Linux 2023.12 LTS
Image hash:     be3240954918 (2023.12.20261120.0 (Kernel 6.1))
CPU(s):         2
Load:           0.00 0.00 0.00
Disk usage:     107.2MiB out of 26.0GiB
Memory usage:   0B out of 1.0GiB
Mounts:         --
```
**Exit Code**: `0`

### 3.7 Info All Instances (--all)
**Command**: `/usr/local/bin/ec2pass info --all`
```text
Name:           matrix-ci
State:          Stopped
Snapshots:      0
IPv4:           --
Release:        Amazon Linux 2023.12 LTS
Image hash:     be3240954918 (2023.12.20261120.0 (Kernel 6.1))
CPU(s):         1
Load:           --
Disk usage:     10.4MiB out of 5.0GiB
Memory usage:   0B out of 1.0GiB
Mounts:         --

Name:           matrix-compat
State:          Running
Snapshots:      0
IPv4:           10.23.104.13
Release:        Amazon Linux 2023.12 LTS
Image hash:     be3240954918 (2023.12.20261120.0 (Kernel 6.1))
CPU(s):         1
Load:           0.00 0.00 0.00
Disk usage:     11.1MiB out of 5.0GiB
Memory usage:   0B out of 1.0GiB
Mounts:         --

Name:           matrix-specs
State:          Running
Snapshots:      0
IPv4:           10.23.104.11
Release:        Amazon Linux 2023.12 LTS
Image hash:     be3240954918 (2023.12.20261120.0 (Kernel 6.1))
CPU(s):         2
Load:           0.00 0.00 0.00
Disk usage:     107.2MiB out of 26.0GiB
Memory usage:   0B out of 1.0GiB
Mounts:         --

Name:           matrix-std
State:          Running
Snapshots:      0
IPv4:           10.23.104.10
Release:        Amazon Linux 2023.12 LTS
Image hash:     be3240954918 (2023.12.20261120.0 (Kernel 6.1))
CPU(s):         1
Load:           0.00 0.00 0.00
Disk usage:     106.9MiB out of 5.0GiB
Memory usage:   0B out of 1.0GiB
Mounts:         --
```
**Exit Code**: `0`

### 3.8 Info Format JSON
**Command**: `/usr/local/bin/ec2pass info matrix-std --format json`
```text
{
  "errors": [],
  "info": {
    "matrix-std": {
      "name": "matrix-std",
      "state": "Running",
      "ipv4": [
        "10.23.104.10"
      ],
      "release": "Amazon Linux 2023.12 LTS",
      "image_hash": "be3240954918ce36bb293481eecdcb9a8943b5af2b4a74fd2b3dddf0cdb8a9af",
      "image_release": "2023.12.20261120.0 (Kernel 6.1)",
      "cpus": 1,
      "load": [
        0.0,
        0.0,
        0.0
      ],
      "memory_usage_bytes": 0,
      "memory_total_bytes": 1073741824,
      "disk_usage_bytes": 112066560,
      "disk_total_bytes": 5368709120,
      "mounts": {},
      "ssh_port": 10022,
      "ssh_user": "ec2-user",
      "ssh_key_path": "/root/.local/share/ec2pass/ssh-keys/id_ed25519_ec2pass",
      "snapshots": [],
      "created_at": "2026-11-22T04:23:44.006674585Z"
    }
  }
}
```
**Exit Code**: `0`

### 3.9 Info Format YAML
**Command**: `/usr/local/bin/ec2pass info matrix-std --format yaml`
```text
matrix-std:
  name: matrix-std
  state: Running
  ipv4:
  - 10.23.104.10
  release: Amazon Linux 2023.12 LTS
  image_hash: be3240954918ce36bb293481eecdcb9a8943b5af2b4a74fd2b3dddf0cdb8a9af
  image_release: 2023.12.20261120.0 (Kernel 6.1)
  cpus: 1
  load:
  - 0.0
  - 0.0
  - 0.0
  memory_usage_bytes: 0
  memory_total_bytes: 1073741824
  disk_usage_bytes: 112066560
  disk_total_bytes: 5368709120
  mounts: {}
  ssh_port: 10022
  ssh_user: ec2-user
  ssh_key_path: /root/.local/share/ec2pass/ssh-keys/id_ed25519_ec2pass
  snapshots: []
  created_at: 2026-11-22T04:23:44.006674585Z
```
**Exit Code**: `0`

### 3.10 Info Format CSV
**Command**: `/usr/local/bin/ec2pass info matrix-std --format csv`
```text
Name,State,Ipv4,Release,Image hash,CPU(s),Disk usage,Disk total,Memory usage,Memory total
matrix-std,Running,10.23.104.10,Amazon Linux 2023.12 LTS,be3240954918ce36bb293481eecdcb9a8943b5af2b4a74fd2b3dddf0cdb8a9af,1,112066560,5368709120,0,1073741824
```
**Exit Code**: `0`

## 4. Execution, File Transfer & Directory Mounting

### 4.1 Exec Simple Command (whoami)
**Command**: `/usr/local/bin/ec2pass exec matrix-std -- whoami`
```text
ec2-user
```
**Exit Code**: `0`

### 4.2 Exec OS Release & Kernel Architecture
**Command**: `/usr/local/bin/ec2pass exec matrix-std -- uname -a`
```text
Linux matrix-std 6.1.190-237.442.amzn2023.x86_64 #1 SMP PREEMPT_DYNAMIC Thu Nov 12 01:09:55 UTC 2026 x86_64 x86_64 x86_64 GNU/Linux
```
**Exit Code**: `0`

### 4.3 Exec with Custom Working Directory
**Command**: `/usr/local/bin/ec2pass exec matrix-std --working-directory /etc -- pwd`
```text
/etc
```
**Exit Code**: `0`

### 4.4 Exec Verifying Cloud-Init Injected File
**Command**: `/usr/local/bin/ec2pass exec matrix-ci -- cat /etc/matrix-ci-test.conf`
```text
error: Instance 'matrix-ci' is not running (current state: Stopped)
```
**Exit Code**: `1`

### 4.5 Transfer Local File to Instance
**Command**: `/usr/local/bin/ec2pass transfer /tmp/matrix_payload.txt matrix-std:/tmp/payload.txt`
```text
```
**Exit Code**: `0`

### 4.6 Verify Transferred File on Instance
**Command**: `/usr/local/bin/ec2pass exec matrix-std -- cat /tmp/payload.txt`
```text
ec2pass-payload-transfer-content
```
**Exit Code**: `0`

### 4.7 Transfer File from Instance to Host
**Command**: `/usr/local/bin/ec2pass transfer matrix-std:/etc/os-release /tmp/guest_os_release.txt && head -n 5 /tmp/guest_os_release.txt`
```text
NAME="Amazon Linux"
VERSION="2023"
ID="amzn"
ID_LIKE="fedora"
VERSION_ID="2023"
```
**Exit Code**: `0`

### 4.8 Mount Host Directory into Instance
**Command**: `/usr/local/bin/ec2pass mount /tmp/matrix_share matrix-std:/mnt/matrix_share`
```text
```
**Exit Code**: `0`

### 4.9 Verify Mounted Directory in Guest
**Command**: `/usr/local/bin/ec2pass exec matrix-std -- ls -la /mnt/matrix_share`
```text
ls: cannot access '/mnt/matrix_share': No such file or directory
```
**Exit Code**: `2`

### 4.10 Unmount Directory from Instance
**Command**: `/usr/local/bin/ec2pass umount matrix-std:/mnt/matrix_share`
```text
```
**Exit Code**: `0`

## 5. Snapshot, Restore, Cloning & Lifecycle Control

### 5.1 Create Named Snapshot with Comment
**Command**: `/usr/local/bin/ec2pass snapshot matrix-std -n baseline-snap -c 'Baseline clean snapshot'`
```text
error: Snapshots can only be taken for stopped instances ('matrix-std' is currently Running).
```
**Exit Code**: `1`

### 5.2 List Snapshots (--snapshots)
**Command**: `/usr/local/bin/ec2pass list --snapshots`
```text
No snapshots found.
```
**Exit Code**: `0`

### 5.3 Info Snapshots (--snapshots)
**Command**: `/usr/local/bin/ec2pass info matrix-std --snapshots`
```text
No snapshots found.
```
**Exit Code**: `0`

### 5.4 Modify Guest State Before Restore
**Command**: `/usr/local/bin/ec2pass exec matrix-std -- sh -c 'echo "canary" > /tmp/canary_file.txt && ls -l /tmp/canary_file.txt'`
```text
-rw-rw-r--. 1 ec2-user ec2-user 1 Nov 22 04:33 /tmp/canary_file.txt
```
**Exit Code**: `0`

### 5.5 Restore Snapshot
**Command**: `/usr/local/bin/ec2pass restore matrix-std -s baseline-snap`
```text
error: unexpected argument '-s' found

  tip: to pass '-s' as a value, use '-- -s'

Usage: ec2pass restore [OPTIONS] <SNAPSHOT_SPEC>

For more information, try '--help'.
```
**Exit Code**: `2`

### 5.6 Verify Snapshot Restored State
**Command**: `/usr/local/bin/ec2pass exec matrix-std -- ls -l /tmp/canary_file.txt || echo 'Canary successfully reverted!'`
```text
-rw-rw-r--. 1 ec2-user ec2-user 1 Nov 22 04:33 /tmp/canary_file.txt
```
**Exit Code**: `0`

### 5.7 Stop Single Instance
**Command**: `/usr/local/bin/ec2pass stop matrix-specs`
```text
```
**Exit Code**: `0`

### 5.8 Stop Instance with Timeout (-t 10)
**Command**: `/usr/local/bin/ec2pass stop matrix-ci -t 10`
```text
```
**Exit Code**: `0`

### 5.9 Clone Stopped Instance
**Command**: `/usr/local/bin/ec2pass clone matrix-specs -n matrix-specs-clone`
```text
Cloned from matrix-specs to matrix-specs-clone.
```
**Exit Code**: `0`

### 5.10 Verify Cloned Instance in List
**Command**: `/usr/local/bin/ec2pass list`
```text
Name                 State              IPv4           Image
matrix-ci            Delayed Shutdown   --             Amazon Linux 2023.12 LTS
matrix-compat        Running            10.23.104.13   Amazon Linux 2023.12 LTS
matrix-specs         Stopped            --             Amazon Linux 2023.12 LTS
matrix-specs-clone   Stopped            --             Amazon Linux 2023.12 LTS
matrix-std           Running            10.23.104.10   Amazon Linux 2023.12 LTS
```
**Exit Code**: `0`

### 5.11 Start Stopped Instance
**Command**: `/usr/local/bin/ec2pass start matrix-specs`
```text
```
**Exit Code**: `0`

### 5.12 Restart Running Instance
**Command**: `/usr/local/bin/ec2pass restart matrix-std && sleep 2`
```text
```
**Exit Code**: `0`

### 5.13 Suspend Running Instance
**Command**: `/usr/local/bin/ec2pass suspend matrix-std`
```text
```
**Exit Code**: `0`

### 5.14 Verify Suspended State in List
**Command**: `/usr/local/bin/ec2pass list`
```text
Name                 State              IPv4           Image
matrix-ci            Delayed Shutdown   --             Amazon Linux 2023.12 LTS
matrix-compat        Running            10.23.104.13   Amazon Linux 2023.12 LTS
matrix-specs         Running            10.23.104.11   Amazon Linux 2023.12 LTS
matrix-specs-clone   Stopped            --             Amazon Linux 2023.12 LTS
matrix-std           Suspended          --             Amazon Linux 2023.12 LTS
```
**Exit Code**: `0`

### 5.15 Resume Suspended Instance via Start
**Command**: `/usr/local/bin/ec2pass start matrix-std`
```text
```
**Exit Code**: `0`

### 5.16 Verify Resumed State in List
**Command**: `/usr/local/bin/ec2pass list`
```text
Name                 State              IPv4           Image
matrix-ci            Delayed Shutdown   --             Amazon Linux 2023.12 LTS
matrix-compat        Running            10.23.104.13   Amazon Linux 2023.12 LTS
matrix-specs         Running            10.23.104.11   Amazon Linux 2023.12 LTS
matrix-specs-clone   Stopped            --             Amazon Linux 2023.12 LTS
matrix-std           Running            10.23.104.10   Amazon Linux 2023.12 LTS
```
**Exit Code**: `0`

## 6. Deletion, Recovery, Purging & Configuration Settings

### 6.1 Create Command Alias on Instance
**Command**: `/usr/local/bin/ec2pass alias matrix-std:uptime host-uptime`
```text
```
**Exit Code**: `0`

### 6.2 List Aliases (Table Format)
**Command**: `/usr/local/bin/ec2pass aliases`
```text
Alias         Instance     Command   Working directory
host-uptime   matrix-std   uptime    map
```
**Exit Code**: `0`

### 6.3 List Aliases (JSON Format)
**Command**: `/usr/local/bin/ec2pass aliases --format json`
```text
[
  {
    "alias_name": "host-uptime",
    "instance": "matrix-std",
    "command": "uptime",
    "working_directory": "map"
  }
]
```
**Exit Code**: `0`

### 6.4 Execute Aliased Command
**Command**: `/usr/local/bin/ec2pass host-uptime || true`
```text
error: unrecognized subcommand 'host-uptime'

  tip: a similar subcommand exists: 'stop'

Usage: ec2pass [OPTIONS] <COMMAND>

For more information, try '--help'.
```
**Exit Code**: `0`

### 6.5 Remove Alias (unalias)
**Command**: `/usr/local/bin/ec2pass unalias host-uptime`
```text
```
**Exit Code**: `0`

### 6.6 Verify Empty Aliases List
**Command**: `/usr/local/bin/ec2pass aliases`
```text
No aliases defined.
```
**Exit Code**: `0`

### 6.7 Soft Delete Instance
**Command**: `/usr/local/bin/ec2pass delete matrix-specs-clone`
```text
```
**Exit Code**: `0`

### 6.8 Verify Deleted State in List
**Command**: `/usr/local/bin/ec2pass list`
```text
Name                 State              IPv4           Image
matrix-ci            Delayed Shutdown   --             Amazon Linux 2023.12 LTS
matrix-compat        Running            10.23.104.13   Amazon Linux 2023.12 LTS
matrix-specs         Running            10.23.104.11   Amazon Linux 2023.12 LTS
matrix-specs-clone   Deleted            --             Amazon Linux 2023.12 LTS
matrix-std           Running            10.23.104.10   Amazon Linux 2023.12 LTS
```
**Exit Code**: `0`

### 6.9 Recover Soft-Deleted Instance
**Command**: `/usr/local/bin/ec2pass recover matrix-specs-clone`
```text
```
**Exit Code**: `0`

### 6.10 Verify Recovered State in List
**Command**: `/usr/local/bin/ec2pass list`
```text
Name                 State              IPv4           Image
matrix-ci            Delayed Shutdown   --             Amazon Linux 2023.12 LTS
matrix-compat        Running            10.23.104.13   Amazon Linux 2023.12 LTS
matrix-specs         Running            10.23.104.11   Amazon Linux 2023.12 LTS
matrix-specs-clone   Stopped            --             Amazon Linux 2023.12 LTS
matrix-std           Running            10.23.104.10   Amazon Linux 2023.12 LTS
```
**Exit Code**: `0`

### 6.11 Immediate Purge on Delete (-p / --purge)
**Command**: `/usr/local/bin/ec2pass delete matrix-specs-clone -p`
```text
```
**Exit Code**: `0`

### 6.12 Delete All Remaining Test Instances
**Command**: `/usr/local/bin/ec2pass delete matrix-std matrix-specs matrix-ci matrix-compat`
```text
```
**Exit Code**: `0`

### 6.13 Purge All Deleted Instances Permanently
**Command**: `/usr/local/bin/ec2pass purge`
```text
```
**Exit Code**: `0`

### 6.14 Verify Empty Instance List
**Command**: `/usr/local/bin/ec2pass list`
```text
No instances found.
```
**Exit Code**: `0`

### 6.15 Get All Configuration Settings
**Command**: `/usr/local/bin/ec2pass get`
```text
client.apps.windows-terminal.profiles=primary
client.primary-name=primary-test
local.bridged-network=
local.driver=qemu
local.passphrase=false
local.privileged-mounts=true
```
**Exit Code**: `0`

### 6.16 Set Configuration Setting
**Command**: `/usr/local/bin/ec2pass set client.primary-name=primary-test`
```text
```
**Exit Code**: `0`

### 6.17 Get Specific Configuration Setting
**Command**: `/usr/local/bin/ec2pass get client.primary-name`
```text
client.primary-name=primary-test
```
**Exit Code**: `0`

### 6.18 Client Authentication
**Command**: `/usr/local/bin/ec2pass authenticate ''`
```text
```
**Exit Code**: `0`


=== Test Matrix Run Finished Successfully ===
