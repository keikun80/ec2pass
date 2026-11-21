//! User-friendly hypervisor installation and troubleshooting guides for Linux, macOS, and Windows.

/// Returns a structured, actionable installation guide for the current platform's hypervisor.
pub fn get_hypervisor_install_guide() -> &'static str {
    if cfg!(target_os = "linux") {
        LINUX_GUIDE
    } else if cfg!(target_os = "macos") {
        MACOS_GUIDE
    } else if cfg!(target_os = "windows") {
        WINDOWS_GUIDE
    } else {
        GENERIC_GUIDE
    }
}

pub const LINUX_GUIDE: &str = r#"
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
"#;

pub const MACOS_GUIDE: &str = r#"
================================================================================
[PREREQUISITE MISSING] macOS virtualization engine (QEMU) was not detected.
ec2pass requires QEMU (qemu-system-x86_64 or qemu-system-aarch64) to leverage
the macOS kernel Hypervisor.framework (HVF) hardware acceleration.
(Simulation/mock mode is not supported).

▶ Resolution steps:
  Install QEMU via Homebrew:
    brew install qemu

  Verify installation:
    qemu-img --version
================================================================================
"#;

pub const WINDOWS_GUIDE: &str = r#"
================================================================================
[PREREQUISITE MISSING] Windows hypervisor (Hyper-V / WHPX) was not detected.
ec2pass requires Hyper-V or WHPX hardware acceleration on Windows.
(Simulation/mock mode is not supported).

▶ Resolution steps (choose one option below):
  1. [Recommended] Enable Hyper-V (Windows Pro / Enterprise):
     Open an Administrator PowerShell and run:
       Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V -All

  2. [Alternative] Enable Windows Hypervisor Platform (WHPX):
     Open an Administrator PowerShell and run:
       Enable-WindowsOptionalFeature -Online -FeatureName HypervisorPlatform -All

  3. Install QEMU for Windows:
     winget install SoftwareFreedomConservancy.QEMU
================================================================================
"#;

pub const GENERIC_GUIDE: &str = r#"
================================================================================
[PREREQUISITE MISSING] Supported hypervisor (KVM, HVF, Hyper-V) was not found.
ec2pass requires hardware virtualization support. Please install the appropriate
virtualization engine for your host operating system.
================================================================================
"#;
