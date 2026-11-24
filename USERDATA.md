# Testing Shell Scripts and cloud-init user-data

This document describes how to test and validate custom shell scripts and `cloud-init` user-data configurations in Amazon Linux 2023 using **InhausLab ec2pass**.

---

## 1. Overview of cloud-init `user-data.txt`

When launching an EC2 instance in AWS or an instance in `ec2pass`, `cloud-init` reads configuration from a `user-data` source. In `ec2pass`, this is injected via a purely generated NoCloud `seed.iso` (labeled `cidata`).

A `user-data` file typically takes one of two forms:
1. **Shell Script (`#!/bin/bash`)**: Executed once during the final `cloud-final` boot phase.
2. **Cloud-Config YAML (`#cloud-config`)**: Declarative configuration parsed by cloud-init modules (creating users, packages, write_files, runcmd, etc.).

`ec2pass launch --cloud-init <file>` decides the format from the first line, like EC2 does:
* `#cloud-config` (or no header at all) is validated as a YAML mapping before boot. A file without a header gets `#cloud-config` added.
* `#!` scripts, MIME multipart (`Content-Type:`), `#include`, `#cloud-boothook`, `#part-handler`, `#cloud-config-archive` and `## template: jinja` are passed to cloud-init unchanged.
* Binary (gzip-compressed) user-data is not supported; pass the uncompressed text.
* If a `#cloud-config` file defines top-level `users:`, ec2pass keeps its own `ec2-user` login in that list (cloud-init lets user-data replace vendor-data keys, which would otherwise lock ec2pass out). An existing `ec2-user` entry only gets the ec2pass SSH key added; your settings for it are kept. The YAML is re-serialized in this case, so comments are dropped.
* Limitation: `users:` inside a MIME multipart or `#include` payload is not adjusted. Include an `ec2-user` entry with the key from `<ec2pass data dir>/ssh-keys/id_ed25519_ec2pass.pub` yourself, or `ec2pass exec`/`shell` will not be able to log in.

---

## 2. Four Testing Strategies

Depending on what stage of development you are in, choose the most effective approach below:

```text
┌────────────────────────────────────────────────────────────────────────┐
│ Method A: End-to-End Boot Test (Full cloud-init execution)             │
│  - Command: ec2pass launch al2023 --cloud-init ./user-data.txt        │
│  - Best for: Testing official EC2 launch behavior from cold start      │
├────────────────────────────────────────────────────────────────────────┤
│ Method B: Interactive / Piped Execution (Fast script iteration)         │
│  - Command: ec2pass exec my-box -- bash < ./test_script.sh             │
│  - Best for: Rapidly modifying and testing shell scripts on running VM │
├────────────────────────────────────────────────────────────────────────┤
│ Method C: Host Mount Testing (Live development)                        │
│  - Command: ec2pass mount ./scripts my-box:/scripts                    │
│  - Best for: Editing scripts in local IDE, executing inside VM         │
├────────────────────────────────────────────────────────────────────────┤
│ Method D: Snapshot & Rollback (Idempotency and retry testing)          │
│  - Command: ec2pass snapshot my-box --name clean-state                 │
│  - Best for: Testing failure conditions and rollbacks                  │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Detailed Step-by-Step Instructions

### Method A: End-to-End Boot Injection (`--cloud-init`)

Create your `user-data.txt` file:

```bash
cat <<'EOF' > user-data.txt
#!/bin/bash
set -e
echo "=== Starting custom initialization ===" > /var/log/my-init.log
dnf install -y nginx jq
systemctl enable --now nginx
echo "Initialization completed at $(date)" >> /var/log/my-init.log
EOF
```

Launch a fresh instance with the script injected:
```bash
ec2pass launch al2023 -n test-bootstrap --cloud-init ./user-data.txt
```

Verify the results:
```bash
# Check cloud-init execution logs
ec2pass exec test-bootstrap -- cat /var/log/cloud-init-output.log

# Check custom script output log
ec2pass exec test-bootstrap -- cat /var/log/my-init.log

# Verify installed service
ec2pass exec test-bootstrap -- systemctl status nginx
```

---

### Method B: Pipe Script directly to a Running Instance (Fastest)

If you have a running instance and want to test a local shell script without restarting or recreating the VM:

```bash
# Execute local script inside guest VM via standard input
ec2pass exec test-bootstrap -- bash -s < ./my_script.sh

# Pass arguments to the script:
ec2pass exec test-bootstrap -- bash -s arg1 arg2 < ./my_script.sh
```

---

### Method C: Live Host Directory Mounting

Mount your local scripts directory into the instance for real-time testing:

```bash
# 1. Mount local folder into guest VM
ec2pass mount ./my-scripts test-bootstrap:/home/ec2-user/scripts

# 2. Execute script directly inside guest
ec2pass exec test-bootstrap -d /home/ec2-user/scripts -- bash ./setup.sh
```

---

### Method D: Snapshot & Rollback for Idempotency Testing

To verify that your installation script is idempotent or test failure recovery:

```bash
# 1. Stop the clean instance and take a snapshot
ec2pass stop test-bootstrap
ec2pass snapshot test-bootstrap -n base-clean -c "Clean base state"

# 2. Start the instance and run your script
ec2pass start test-bootstrap
ec2pass exec test-bootstrap -- bash -s < ./deploy.sh

# 3. Roll back to the clean snapshot to test again
ec2pass stop test-bootstrap
ec2pass restore test-bootstrap -s base-clean
ec2pass start test-bootstrap
```

---

## 4. Diagnosing and Debugging cloud-init in Amazon Linux 2023

If your script fails or does not execute as expected, inspect the following files inside the instance:

1. **Check cloud-init execution status**:
   ```bash
   ec2pass exec test-bootstrap -- cloud-init status --long
   ```
2. **Read complete console output**:
   ```bash
   ec2pass exec test-bootstrap -- cat /var/log/cloud-init-output.log
   ```
3. **Read detailed module execution logs**:
   ```bash
   ec2pass exec test-bootstrap -- cat /var/log/cloud-init.log
   ```
4. **Inspect the resolved user-data script**:
   ```bash
   ec2pass exec test-bootstrap -- sudo cat /var/lib/cloud/instance/user-data.txt
   ```
5. **Re-run cloud-init manually inside the VM**:
   ```bash
   ec2pass exec test-bootstrap -- sudo cloud-init clean --logs
   ec2pass exec test-bootstrap -- sudo cloud-init init
   ec2pass exec test-bootstrap -- sudo cloud-init modules --mode=final
   ```
