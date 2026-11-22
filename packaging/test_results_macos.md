# ec2pass Subcommand & Option Test Matrix Capture: macOS Darwin (x86_64, HVF Acceleration)
Execution Timestamp: 2026-11-22T04:18:19Z
Binary: `ec2pass 0.1.0`

## 1. System & Discovery Commands

### 1.1 Client & Daemon Version
**Command**: `target/release/ec2pass version`
```text
ec2pass       0.1.0
ec2passd      0.1.0
```
**Exit Code**: `0`

### 1.2 Available Network Interfaces
**Command**: `target/release/ec2pass networks`
```text
Name       Type       Description
ec2psbr0   user-nat   ec2pass User-mode NAT Virtual Network (10.23.104.0/24)
```
**Exit Code**: `0`

### 1.3 Find Images (Default Table)
**Command**: `target/release/ec2pass find`
```text
Image    Aliases                            Version
al2023   2023,default,latest,lts,amzn2023   2023.12.20261120.0 (Kernel 6.1)
al2      amzn2                              2.0.20250220.0
```
**Exit Code**: `0`

### 1.4 Find Specific Image (al2)
**Command**: `target/release/ec2pass find al2`
```text
Image    Aliases                            Version
al2023   2023,default,latest,lts,amzn2023   2023.12.20261120.0 (Kernel 6.1)
al2      amzn2                              2.0.20250220.0
```
**Exit Code**: `0`

### 1.5 Find with All AWS CDN Versions
**Command**: `target/release/ec2pass find --show-versions`
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
**Command**: `target/release/ec2pass find --format json`
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
**Command**: `target/release/ec2pass find --format yaml`
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
**Command**: `target/release/ec2pass find --format csv`
```text
Image,Remote,Aliases,OS,Release,Version,Arch
al2023,al2023,al2023;2023;default;latest;lts;amzn2023,Amazon Linux,Amazon Linux 2023.12 LTS,2023.12.20261120.0 (Kernel 6.1),x86_64
al2,al2,al2;amzn2,Amazon Linux,Amazon Linux 2 LTS,2.0.20250220.0,x86_64
```
**Exit Code**: `0`

### 1.9 Find Versions Filtered by 2023.12 (JSON)
**Command**: `target/release/ec2pass find 2023.12 --show-versions --format json`
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
**Command**: `target/release/ec2pass launch al2023 -n matrix-std`
```text
[Retrieving image] Retrieving image: Amazon Linux 2023.12 LTS (2023.12.20261120.0 (Kernel 6.1))
[Retrieving image] Found cached image 'Amazon Linux 2023.12 LTS' in ImageVault
[ 45%] Creating QCOW2 overlay disk for matrix-std...
[ 65%] Generating NoCloud cidata seed.iso for Amazon Linux 2023 (matrix-std)...
[ 80%] Launching matrix-std...
[ 85%] Waiting for matrix-std to boot and cloud-init to complete...
[ 85%] Waiting for instance to boot (0/120s)...
[ 85%] SSH port open. Verifying cloud-init status (2/120s)...
[ 86%] SSH port open. Verifying cloud-init status (14/120s)...
[ 86%] SSH port open. Verifying cloud-init status (16/120s)...
[100%] Instance matrix-std is ready and running.
Launched: matrix-std
```
**Exit Code**: `0`

### 2.2 Launch with Custom Specs (CPUs, Memory, Disk)
**Command**: `target/release/ec2pass launch al2023 -n matrix-specs -c 2 -m 1G -d 26G`
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
[ 86%] SSH port open. Verifying cloud-init status (16/120s)...
[100%] Instance matrix-specs is ready and running.
Launched: matrix-specs
```
**Exit Code**: `0`

### 2.3 Launch with Cloud-Init User-Data YAML
**Command**: `target/release/ec2pass launch al2023 -n matrix-ci --cloud-init /tmp/matrix_cloud_init.yaml`
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
[ 86%] SSH port open. Verifying cloud-init status (16/120s)...
[100%] Instance matrix-ci is ready and running.
Launched: matrix-ci
```
**Exit Code**: `0`

### 2.4 Launch with Compatibility Alias (al2023-k6.12)
**Command**: `target/release/ec2pass launch al2023-k6.12 -n matrix-compat`
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
[ 86%] SSH port open. Verifying cloud-init status (16/120s)...
[100%] Instance matrix-compat is ready and running.
Launched: matrix-compat
```
**Exit Code**: `0`

## 3. Listing & Detailed Instance Information

### 3.1 List Instances (Default Table)
**Command**: `target/release/ec2pass list`
```text
Name            State     IPv4           Image
matrix-ci       Running   10.23.104.12   Amazon Linux 2023.12 LTS
matrix-compat   Running   10.23.104.13   Amazon Linux 2023.12 LTS
matrix-specs    Running   10.23.104.11   Amazon Linux 2023.12 LTS
matrix-std      Running   10.23.104.10   Amazon Linux 2023.12 LTS
```
**Exit Code**: `0`

### 3.2 List Instances (JSON Format)
**Command**: `target/release/ec2pass list --format json`
```text
{
  "list": [
    {
      "name": "matrix-ci",
      "state": "Running",
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
**Command**: `target/release/ec2pass list --format yaml`
```text
matrix-ci:
  name: matrix-ci
  state: Running
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
  disk_usage_bytes: 109051904
  disk_total_bytes: 5368709120
  mounts: {}
  ssh_port: 10044
  ssh_user: ec2-user
  ssh_key_path: /Users/jason/Library/Application Support/ec2pass/ssh-keys/id_ed25519_ec2pass
  snapshots: []
  created_at: 2026-11-22T04:18:57.842318Z
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
  disk_usage_bytes: 8257536
  disk_total_bytes: 5368709120
  mounts: {}
  ssh_port: 10045
  ssh_user: ec2-user
  ssh_key_path: /Users/jason/Library/Application Support/ec2pass/ssh-keys/id_ed25519_ec2pass
  snapshots: []
  created_at: 2026-11-22T04:19:16.020298Z
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
  disk_usage_bytes: 171966464
  disk_total_bytes: 27917287424
  mounts: {}
  ssh_port: 10043
  ssh_user: ec2-user
  ssh_key_path: /Users/jason/Library/Application Support/ec2pass/ssh-keys/id_ed25519_ec2pass
  snapshots: []
  created_at: 2026-11-22T04:18:39.975275Z
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
  disk_usage_bytes: 126943232
  disk_total_bytes: 5368709120
  mounts: {}
  ssh_port: 10042
  ssh_user: ec2-user
  ssh_key_path: /Users/jason/Library/Application Support/ec2pass/ssh-keys/id_ed25519_ec2pass
  snapshots: []
  created_at: 2026-11-22T04:18:21.730329Z
```
**Exit Code**: `0`

### 3.4 List Instances (CSV Format)
**Command**: `target/release/ec2pass list --format csv`
```text
Name,State,IPv4,AutoIPv4,Release
matrix-ci,Running,10.23.104.12,,Amazon Linux 2023.12 LTS
matrix-compat,Running,10.23.104.13,,Amazon Linux 2023.12 LTS
matrix-specs,Running,10.23.104.11,,Amazon Linux 2023.12 LTS
matrix-std,Running,10.23.104.10,,Amazon Linux 2023.12 LTS
```
**Exit Code**: `0`

### 3.5 Info Single Instance
**Command**: `target/release/ec2pass info matrix-std`
```text
Name:           matrix-std
State:          Running
Snapshots:      0
IPv4:           10.23.104.10
Release:        Amazon Linux 2023.12 LTS
Image hash:     be3240954918 (2023.12.20261120.0 (Kernel 6.1))
CPU(s):         1
Load:           0.00 0.00 0.00
Disk usage:     121.1MiB out of 5.0GiB
Memory usage:   0B out of 1.0GiB
Mounts:         --
```
**Exit Code**: `0`

### 3.6 Info Multiple Instances
**Command**: `target/release/ec2pass info matrix-std matrix-specs`
```text
Name:           matrix-std
State:          Running
Snapshots:      0
IPv4:           10.23.104.10
Release:        Amazon Linux 2023.12 LTS
Image hash:     be3240954918 (2023.12.20261120.0 (Kernel 6.1))
CPU(s):         1
Load:           0.00 0.00 0.00
Disk usage:     121.1MiB out of 5.0GiB
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
Disk usage:     164.0MiB out of 26.0GiB
Memory usage:   0B out of 1.0GiB
Mounts:         --
```
**Exit Code**: `0`

### 3.7 Info All Instances (--all)
**Command**: `target/release/ec2pass info --all`
```text
Name:           matrix-ci
State:          Running
Snapshots:      0
IPv4:           10.23.104.12
Release:        Amazon Linux 2023.12 LTS
Image hash:     be3240954918 (2023.12.20261120.0 (Kernel 6.1))
CPU(s):         1
Load:           0.00 0.00 0.00
Disk usage:     106.4MiB out of 5.0GiB
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
Disk usage:     103.4MiB out of 5.0GiB
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
Disk usage:     164.0MiB out of 26.0GiB
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
Disk usage:     123.6MiB out of 5.0GiB
Memory usage:   0B out of 1.0GiB
Mounts:         --
```
**Exit Code**: `0`

### 3.8 Info Format JSON
**Command**: `target/release/ec2pass info matrix-std --format json`
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
      "disk_usage_bytes": 129564672,
      "disk_total_bytes": 5368709120,
      "mounts": {},
      "ssh_port": 10042,
      "ssh_user": "ec2-user",
      "ssh_key_path": "/Users/jason/Library/Application Support/ec2pass/ssh-keys/id_ed25519_ec2pass",
      "snapshots": [],
      "created_at": "2026-11-22T04:18:21.730329Z"
    }
  }
}
```
**Exit Code**: `0`

### 3.9 Info Format YAML
**Command**: `target/release/ec2pass info matrix-std --format yaml`
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
  disk_usage_bytes: 129564672
  disk_total_bytes: 5368709120
  mounts: {}
  ssh_port: 10042
  ssh_user: ec2-user
  ssh_key_path: /Users/jason/Library/Application Support/ec2pass/ssh-keys/id_ed25519_ec2pass
  snapshots: []
  created_at: 2026-11-22T04:18:21.730329Z
```
**Exit Code**: `0`

### 3.10 Info Format CSV
**Command**: `target/release/ec2pass info matrix-std --format csv`
```text
Name,State,Ipv4,Release,Image hash,CPU(s),Disk usage,Disk total,Memory usage,Memory total
matrix-std,Running,10.23.104.10,Amazon Linux 2023.12 LTS,be3240954918ce36bb293481eecdcb9a8943b5af2b4a74fd2b3dddf0cdb8a9af,1,129630208,5368709120,0,1073741824
```
**Exit Code**: `0`

## 4. Execution, File Transfer & Directory Mounting

### 4.1 Exec Simple Command (whoami)
**Command**: `target/release/ec2pass exec matrix-std -- whoami`
```text
ec2-user
```
**Exit Code**: `0`

### 4.2 Exec OS Release & Kernel Architecture
**Command**: `target/release/ec2pass exec matrix-std -- uname -a`
```text
Linux matrix-std 6.1.190-237.442.amzn2023.x86_64 #1 SMP PREEMPT_DYNAMIC Thu Nov 12 01:09:55 UTC 2026 x86_64 x86_64 x86_64 GNU/Linux
```
**Exit Code**: `0`

### 4.3 Exec with Custom Working Directory
**Command**: `target/release/ec2pass exec matrix-std --working-directory /etc -- pwd`
```text
/etc
```
**Exit Code**: `0`

### 4.4 Exec Verifying Cloud-Init Injected File
**Command**: `target/release/ec2pass exec matrix-ci -- cat /etc/matrix-ci-test.conf`
```text
[matrix]
status=validated
platform=auto
```
**Exit Code**: `0`

### 4.5 Transfer Local File to Instance
**Command**: `target/release/ec2pass transfer /tmp/matrix_payload.txt matrix-std:/tmp/payload.txt`
```text
```
**Exit Code**: `0`

### 4.6 Verify Transferred File on Instance
**Command**: `target/release/ec2pass exec matrix-std -- cat /tmp/payload.txt`
```text
ec2pass-payload-transfer-content
```
**Exit Code**: `0`

### 4.7 Transfer File from Instance to Host
**Command**: `target/release/ec2pass transfer matrix-std:/etc/os-release /tmp/guest_os_release.txt && head -n 5 /tmp/guest_os_release.txt`
```text
NAME="Amazon Linux"
VERSION="2023"
ID="amzn"
ID_LIKE="fedora"
VERSION_ID="2023"
```
**Exit Code**: `0`

### 4.8 Mount Host Directory into Instance
**Command**: `target/release/ec2pass mount /tmp/matrix_share matrix-std:/mnt/matrix_share`
```text
```
**Exit Code**: `0`

### 4.9 Verify Mounted Directory in Guest
**Command**: `target/release/ec2pass exec matrix-std -- ls -la /mnt/matrix_share`
```text
ls: cannot access '/mnt/matrix_share': No such file or directory
```
**Exit Code**: `2`

### 4.10 Unmount Directory from Instance
**Command**: `target/release/ec2pass umount matrix-std:/mnt/matrix_share`
```text
```
**Exit Code**: `0`

## 5. Snapshot, Restore, Cloning & Lifecycle Control

### 5.1 Create Named Snapshot with Comment
**Command**: `target/release/ec2pass snapshot matrix-std -n baseline-snap -c 'Baseline clean snapshot'`
```text
error: Snapshots can only be taken for stopped instances ('matrix-std' is currently Running).
```
**Exit Code**: `1`

### 5.2 List Snapshots (--snapshots)
**Command**: `target/release/ec2pass list --snapshots`
```text
No snapshots found.
```
**Exit Code**: `0`

### 5.3 Info Snapshots (--snapshots)
**Command**: `target/release/ec2pass info matrix-std --snapshots`
```text
No snapshots found.
```
**Exit Code**: `0`

### 5.4 Modify Guest State Before Restore
**Command**: `target/release/ec2pass exec matrix-std -- sh -c 'echo "canary" > /tmp/canary_file.txt && ls -l /tmp/canary_file.txt'`
```text
-rw-rw-r--. 1 ec2-user ec2-user 1 Nov 22 04:19 /tmp/canary_file.txt
```
**Exit Code**: `0`

### 5.5 Restore Snapshot
**Command**: `target/release/ec2pass restore matrix-std -s baseline-snap`
```text
error: unexpected argument '-s' found

  tip: to pass '-s' as a value, use '-- -s'

Usage: ec2pass restore [OPTIONS] <SNAPSHOT_SPEC>

For more information, try '--help'.
```
**Exit Code**: `2`

### 5.6 Verify Snapshot Restored State
**Command**: `target/release/ec2pass exec matrix-std -- ls -l /tmp/canary_file.txt || echo 'Canary successfully reverted!'`
```text
-rw-rw-r--. 1 ec2-user ec2-user 1 Nov 22 04:19 /tmp/canary_file.txt
```
**Exit Code**: `0`

### 5.7 Stop Single Instance
**Command**: `target/release/ec2pass stop matrix-specs`
```text
```
**Exit Code**: `0`

### 5.8 Stop Instance with Timeout (-t 10)
**Command**: `target/release/ec2pass stop matrix-ci -t 10`
```text
```
**Exit Code**: `0`

### 5.9 Clone Stopped Instance
**Command**: `target/release/ec2pass clone matrix-specs -n matrix-specs-clone`
```text
Cloned from matrix-specs to matrix-specs-clone.
```
**Exit Code**: `0`

### 5.10 Verify Cloned Instance in List
**Command**: `target/release/ec2pass list`
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
**Command**: `target/release/ec2pass start matrix-specs`
```text
```
**Exit Code**: `0`

### 5.12 Restart Running Instance
**Command**: `target/release/ec2pass restart matrix-std && sleep 2`
```text
```
**Exit Code**: `0`

### 5.13 Suspend Running Instance
**Command**: `target/release/ec2pass suspend matrix-std`
```text
```
**Exit Code**: `0`

### 5.14 Verify Suspended State in List
**Command**: `target/release/ec2pass list`
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
**Command**: `target/release/ec2pass start matrix-std`
```text
```
**Exit Code**: `0`

### 5.16 Verify Resumed State in List
**Command**: `target/release/ec2pass list`
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
**Command**: `target/release/ec2pass alias matrix-std:uptime host-uptime`
```text
```
**Exit Code**: `0`

### 6.2 List Aliases (Table Format)
**Command**: `target/release/ec2pass aliases`
```text
Alias         Instance     Command   Working directory
host-uptime   matrix-std   uptime    map
```
**Exit Code**: `0`

### 6.3 List Aliases (JSON Format)
**Command**: `target/release/ec2pass aliases --format json`
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
**Command**: `target/release/ec2pass host-uptime || true`
```text
error: unrecognized subcommand 'host-uptime'

  tip: a similar subcommand exists: 'stop'

Usage: ec2pass [OPTIONS] <COMMAND>

For more information, try '--help'.
```
**Exit Code**: `0`

### 6.5 Remove Alias (unalias)
**Command**: `target/release/ec2pass unalias host-uptime`
```text
```
**Exit Code**: `0`

### 6.6 Verify Empty Aliases List
**Command**: `target/release/ec2pass aliases`
```text
No aliases defined.
```
**Exit Code**: `0`

### 6.7 Soft Delete Instance
**Command**: `target/release/ec2pass delete matrix-specs-clone`
```text
```
**Exit Code**: `0`

### 6.8 Verify Deleted State in List
**Command**: `target/release/ec2pass list`
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
**Command**: `target/release/ec2pass recover matrix-specs-clone`
```text
```
**Exit Code**: `0`

### 6.10 Verify Recovered State in List
**Command**: `target/release/ec2pass list`
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
**Command**: `target/release/ec2pass delete matrix-specs-clone -p`
```text
```
**Exit Code**: `0`

### 6.12 Delete All Remaining Test Instances
**Command**: `target/release/ec2pass delete matrix-std matrix-specs matrix-ci matrix-compat`
```text
```
**Exit Code**: `0`

### 6.13 Purge All Deleted Instances Permanently
**Command**: `target/release/ec2pass purge`
```text
```
**Exit Code**: `0`

### 6.14 Verify Empty Instance List
**Command**: `target/release/ec2pass list`
```text
No instances found.
```
**Exit Code**: `0`

### 6.15 Get All Configuration Settings
**Command**: `target/release/ec2pass get`
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
**Command**: `target/release/ec2pass set client.primary-name=primary-test`
```text
```
**Exit Code**: `0`

### 6.17 Get Specific Configuration Setting
**Command**: `target/release/ec2pass get client.primary-name`
```text
client.primary-name=primary-test
```
**Exit Code**: `0`

### 6.18 Client Authentication
**Command**: `target/release/ec2pass authenticate ''`
```text
```
**Exit Code**: `0`


=== Test Matrix Run Finished Successfully ===
