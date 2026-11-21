# InhausLab ec2pass (`ec2pass` / `multipass`)
# 멀티 플랫폼 컴파일, 테스트 및 배포 가이드 (`platform.md`)

본 문서는 **Windows, macOS, Linux** 환경에서 `ec2pass`를 컴파일하고, 단위/통합 테스트를 수행하며, 플랫폼별 표준 배포 패키지(**Windows .exe, macOS .tar.gz, Linux .tar.gz**)를 생성하는 공식 매뉴얼입니다.

---

## 1. 멀티 플랫폼 지원 및 배포 매트릭스

| 플랫폼 (OS) | 하이퍼바이저 백엔드 (필수) | 지원 아키텍처 | 공식 배포 산출물 | 배포 및 서비스 형태 |
| :--- | :--- | :--- | :--- | :--- |
| **Windows** (10, 11 Pro/Enterprise/Home) | **Hyper-V / WHPX** | `x86_64` | **`ec2pass-setup.exe`** (독립 실행 설치 파일) | User PATH 등록 / Windows Service |
| **macOS** (Sequoia, Sonoma 등) | **Hypervisor.framework (`HVF`) + QEMU** | `aarch64` (Apple Silicon), `x86_64` (Intel) | **`ec2pass-v1.0.0-darwin-<arch>.tar.gz`** | `launchd` (`com.inhauslab.ec2passd.plist`) |
| **Linux** (Ubuntu, RHEL, Fedora, AL2023) | **KVM (`/dev/kvm`) + QEMU** | `x86_64`, `aarch64` | **`ec2pass-v1.0.0-linux-<arch>.tar.gz`** | `systemd` (`ec2passd.service`) |

> [!IMPORTANT]
> **하이퍼바이저 필수화 정책 (시뮬레이션 모드 금지)**  
> `ec2pass`는 신뢰할 수 있는 격리 및 고성능 가상 머신 구동을 위해 **호스트의 실제 하드웨어 가상화 엔진(Hypervisor)을 필수로 요구**합니다.  
> 인스톨러 설치 시점 및 프로그램 실행 시점에 하이퍼바이저 가속 여부를 엄격히 점검하며, 미설치 시 친절한 OS별 원클릭 설치 가이드를 표시하고 안전하게 종료됩니다.

---

## 2. 소스 트리 아키텍처 (`ec2ps-core/hypervisor`)

멀티 플랫폼 추상화 및 하이퍼바이저 점검을 위해 소스 트리가 모듈화되어 있습니다:

```text
crates/
├── ec2ps-core/
│   └── src/
│       ├── hypervisor/               # 플랫폼별 가상화 추상화 & 사전 점검 모듈
│       │   ├── mod.rs                # VirtualMachineDriver trait 및 공통 점검 진입점
│       │   ├── guide.rs              # OS별 사용자 친화적 설치 가이드 템플릿
│       │   ├── linux.rs              # Linux KVM (/dev/kvm, kvm 그룹) 검증
│       │   ├── macos.rs              # macOS HVF (sysctl kern.hv_support, QEMU) 검증
│       │   ├── windows.rs            # Windows Hyper-V / WHPX 검증 및 드라이버
│       │   └── qemu_common.rs        # QMP 소켓 제어 및 QEMU 프로세스 관리
│       ├── catalog.rs                # AWS 공식 CDN AL2023 이미지 캐시
│       ├── cloud_init.rs             # NoCloud cidata 생성기
│       └── state.rs                  # 인스턴스 메타데이터 저장소
├── ec2ps-cli/                        # ec2pass / multipass CLI 도구
├── ec2ps-daemon/                     # ec2passd 백그라운드 오케스트레이터 데몬
├── ec2ps-iso9660/                    # 순수 Rust ISO9660 라이브러리
└── ec2ps-protocol/                   # IPC 통신 프로토콜 및 포맷터
```

---

## 3. 플랫폼별 사전 준비 및 하이퍼바이저 설치 가이드

### A. Linux (Ubuntu, Debian, RHEL, Fedora, Amazon Linux 2023)
```bash
# 1. KVM 및 QEMU 설치
# [Ubuntu / Debian]
sudo apt-get update && sudo apt-get install -y qemu-system-x86 qemu-utils libvirt-daemon-system

# [RHEL / Fedora / Amazon Linux 2023]
sudo dnf install -y qemu-kvm qemu-img

# 2. 현재 사용자를 kvm 그룹에 추가
sudo usermod -aG kvm $USER && newgrp kvm

# 3. KVM 장치 확인
ls -l /dev/kvm
```

### B. macOS (Apple Silicon M-series & Intel Mac)
```bash
# Homebrew를 통해 QEMU 설치 (HVF 커널 가속 기본 연동)
brew install qemu

# 설치 확인
qemu-img --version
```

### C. Windows (10, 11 Pro/Enterprise/Home)
관리자 권한 PowerShell에서 실행:
```powershell
# 1. Hyper-V 활성화 (Windows Pro / Enterprise)
Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V -All

# 2. 또는 Windows Hypervisor Platform (WHPX) 활성화 (Windows Home 등)
Enable-WindowsOptionalFeature -Online -FeatureName HypervisorPlatform -All

# 3. 시스템 재부팅 후 설치 진행
```

---

## 4. 컴파일 및 로컬 패키징 실행

### Linux / macOS
```bash
# 릴리스 빌드, 단위 테스트 검증, .tar.gz 패키징 일괄 수행
chmod +x ./packaging/build_release.sh
./packaging/build_release.sh
```
* **생성 산출물**: `dist/ec2pass-v1.0.0-<os>-<arch>.tar.gz` 및 `.sha256` 체크섬 파일

### Windows
PowerShell에서 실행:
```powershell
# 릴리스 빌드 및 ec2pass-setup.exe 설치 파일 일괄 생성
.\packaging\build_windows_installer.ps1
```
* **생성 산출물**: `dist\ec2pass-setup.exe` 및 `dist\ec2pass-v1.0.0-windows-x86_64.zip`
