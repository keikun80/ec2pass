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
[필수 요구사항 미충족] Linux 하이퍼바이저(KVM)가 감지되지 않았습니다.
ec2pass는 안전한 격리 및 가상 머신 구동을 위해 KVM 하이퍼바이저가 필수입니다.
(시뮬레이션 모드는 지원되지 않습니다.)

▶ 해결 방법 (아래 명령어를 실행하세요):
  1. 배포판별 패키지 설치:
     • Ubuntu / Debian:
       sudo apt-get update && sudo apt-get install -y qemu-system-x86 qemu-utils libvirt-daemon-system
     • RHEL / Fedora / Amazon Linux 2023:
       sudo dnf install -y qemu-kvm qemu-img
     • Arch Linux:
       sudo pacman -S qemu-full

  2. 현재 사용자를 kvm 그룹에 추가:
     sudo usermod -aG kvm $USER
     newgrp kvm

  3. KVM 커널 모듈 활성화 확인:
     ls -l /dev/kvm
     (만약 /dev/kvm이 여전히 나타나지 않는다면, PC 부팅 시 BIOS/UEFI에서
      Intel VT-x 또는 AMD-V 가상화 지원이 활성화되어 있는지 확인하세요.)
================================================================================
"#;

pub const MACOS_GUIDE: &str = r#"
================================================================================
[필수 요구사항 미충족] macOS 가상화 엔진(QEMU)이 감지되지 않았습니다.
ec2pass는 macOS의 커널 Hypervisor.framework (HVF) 하드웨어 가속을 활용하기 위해
QEMU(qemu-system-x86_64 / qemu-system-aarch64)가 필수입니다.
(시뮬레이션 모드는 지원되지 않습니다.)

▶ 해결 방법 (아래 명령어를 터미널에서 실행하세요):
  Homebrew를 통해 QEMU를 설치해 주세요:
    brew install qemu

  설치 확인:
    qemu-img --version
================================================================================
"#;

pub const WINDOWS_GUIDE: &str = r#"
================================================================================
[필수 요구사항 미충족] Windows 하이퍼바이저(Hyper-V / WHPX)가 감지되지 않았습니다.
ec2pass는 Windows에서 하드웨어 수준의 가상화를 위해 Hyper-V 또는 WHPX가 필수입니다.
(시뮬레이션 모드는 지원되지 않습니다.)

▶ 해결 방법 (아래 중 하나를 진행하세요):
  1. [권장] Hyper-V 기능 활성화 (Windows Pro / Enterprise):
     관리자 권한 PowerShell을 열고 다음 명령 실행 후 재부팅:
       Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V -All

  2. [대안 1] Windows 하이퍼바이저 플랫폼 (WHPX) 활성화 (Windows Home 등):
     관리자 권한 PowerShell을 열고 다음 명령 실행 후 재부팅:
       Enable-WindowsOptionalFeature -Online -FeatureName HypervisorPlatform -All

  3. [대안 2] QEMU 설치 (WHPX 가속 연동):
     winget install SoftwareFreedomConservancy.QEMU
     (또는 choco install qemu)
================================================================================
"#;

pub const GENERIC_GUIDE: &str = r#"
================================================================================
[필수 요구사항 미충족] 지원되는 하이퍼바이저(KVM, HVF, Hyper-V)를 찾을 수 없습니다.
ec2pass는 하드웨어 가상화 지원이 필수입니다. 호스트 운영체제에 맞는 가상화 엔진을 설치해 주세요.
================================================================================
"#;
