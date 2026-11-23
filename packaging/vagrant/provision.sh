#!/usr/bin/env bash
# Provisioning Script for ec2pass Multi-Distribution Vagrant Environment
# Supports: Rocky Linux / EL9, Amazon Linux 2023, Fedora, Debian 12+, Ubuntu 22.04 / 24.04
set -euo pipefail

echo "================================================================"
echo "  Provisioning Linux Environment for ec2pass"
echo "================================================================"

# Detect distribution
if [ -f /etc/os-release ]; then
    . /etc/os-release
    DISTRO_ID="${ID:-unknown}"
    DISTRO_LIKE="${ID_LIKE:-unknown}"
else
    DISTRO_ID="unknown"
    DISTRO_LIKE="unknown"
fi

echo "Detected OS Distribution: $DISTRO_ID (family: $DISTRO_LIKE)"

# 1. Update and install development tools and virtualization packages
if [[ "$DISTRO_ID" =~ ^(rocky|almalinux|rhel|centos|fedora|amzn)$ ]] || [[ "$DISTRO_LIKE" =~ (rhel|fedora) ]]; then
    echo "[1/4] Installing tools and QEMU/KVM via dnf..."
    dnf install -y epel-release 2>/dev/null || true
    dnf groupinstall -y "Development Tools" 2>/dev/null || true
    dnf install -y \
        gcc \
        gcc-c++ \
        make \
        pkgconf \
        pkgconf-pkg-config \
        openssl-devel \
        git \
        curl \
        wget \
        tar \
        which \
        ca-certificates 2>/dev/null || true

    dnf install -y \
        qemu-kvm \
        qemu-img \
        libvirt \
        libvirt-client \
        openssh-clients \
        genisoimage \
        xorriso 2>/dev/null || true

    if [ -f /usr/libexec/qemu-kvm ] && [ ! -f /usr/bin/qemu-system-x86_64 ]; then
        ln -sf /usr/libexec/qemu-kvm /usr/bin/qemu-system-x86_64
    fi

elif [[ "$DISTRO_ID" =~ ^(debian|ubuntu)$ ]] || [[ "$DISTRO_LIKE" =~ debian ]]; then
    echo "[1/4] Installing tools and QEMU/KVM via apt-get..."
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -y
    apt-get install -y \
        build-essential \
        pkg-config \
        libssl-dev \
        git \
        curl \
        wget \
        tar \
        which \
        ca-certificates \
        openssh-client \
        qemu-system-x86 \
        qemu-utils \
        genisoimage \
        xorriso
else
    echo "[1/4] Warning: Unknown distribution $DISTRO_ID. Attempting package manager detection..."
    if command -v dnf >/dev/null 2>&1; then
        dnf install -y gcc gcc-c++ make pkgconf openssl-devel git curl wget tar which qemu-kvm qemu-img openssh-clients genisoimage || true
    elif command -v apt-get >/dev/null 2>&1; then
        apt-get update -y && apt-get install -y build-essential pkg-config libssl-dev git curl wget tar openssh-client qemu-system-x86 qemu-utils genisoimage || true
    fi
fi

# 2. Configure KVM permissions and nested virtualization
echo "[2/4] Configuring KVM access and permissions..."
if [ -e /dev/kvm ]; then
    groupadd -f kvm 2>/dev/null || true
    usermod -aG kvm vagrant 2>/dev/null || true
    chmod 666 /dev/kvm
    echo "  [OK] /dev/kvm detected and read/write access granted to vagrant user."
else
    echo "  [INFO] /dev/kvm not found in this VM (unaccelerated mode will be used if needed)."
fi

# 3. Install Rust Toolchain for vagrant user
echo "[3/4] Installing Rust Toolchain (stable) for vagrant user..."
if ! su - vagrant -c "command -v cargo" >/dev/null 2>&1; then
    su - vagrant -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable"
fi

# Configure system-wide Rust PATH
cat << 'EOF' > /etc/profile.d/rust.sh
export PATH="$HOME/.cargo/bin:/home/vagrant/.cargo/bin:$PATH"
EOF
chmod 644 /etc/profile.d/rust.sh

# 4. Verify tools
echo "[4/4] Verifying installed toolchains..."
su - vagrant -c "rustc --version" || true
su - vagrant -c "cargo --version" || true
qemu-img --version | head -n 1 || true

echo "================================================================"
echo "  Provisioning Completed Successfully for $DISTRO_ID!"
echo "================================================================"
