#!/usr/bin/env bash
# Provisioning Script for ec2pass Amazon Linux / EL9 Vagrant Environment
set -euo pipefail

echo "================================================================"
echo "  Provisioning Amazon Linux / EL9 Environment for ec2pass"
echo "================================================================"

# 1. Update and install development tools
echo "[1/4] Installing C/C++ toolchains, kernel headers, and build tools..."
dnf install -y epel-release 2>/dev/null || true
dnf groupinstall -y "Development Tools"
dnf install -y \
    gcc \
    gcc-c++ \
    make \
    pkg-config \
    openssl-devel \
    git \
    curl \
    wget \
    tar \
    which \
    ca-certificates

# 2. Install QEMU, KVM, and virtualization packages
echo "[2/4] Installing QEMU, KVM, and image utilities..."
dnf install -y \
    qemu-kvm \
    qemu-img \
    libvirt \
    libvirt-client \
    openssh-clients \
    genisoimage \
    xorriso || true

# 3. Configure KVM permissions and nested virtualization
echo "[3/4] Configuring KVM access and permissions..."
if [ -e /dev/kvm ]; then
    usermod -aG kvm vagrant || true
    chmod 666 /dev/kvm
    echo "  [OK] /dev/kvm detected and read/write access granted to vagrant user."
else
    echo "  [INFO] /dev/kvm not found. (ec2pass fallback simulation mode will be used if needed)."
fi

# 4. Install Rust Toolchain for vagrant user
echo "[4/4] Installing Rust Toolchain (stable) for vagrant user..."
if ! su - vagrant -c "command -v cargo" >/dev/null 2>&1; then
    su - vagrant -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable"
fi

# Configure system-wide Rust PATH
cat << 'EOF' > /etc/profile.d/rust.sh
export PATH="$HOME/.cargo/bin:/home/vagrant/.cargo/bin:$PATH"
EOF
chmod 644 /etc/profile.d/rust.sh

# Verify tools
echo "================================================================"
echo "  Provisioning Completed Successfully!"
echo "================================================================"
su - vagrant -c "rustc --version" || true
su - vagrant -c "cargo --version" || true
qemu-img --version | head -n 1 || true
echo "Ready to build and test ec2pass."
