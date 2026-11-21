# InhausLab ec2pass (`ec2pass` / `multipass`)
## 멀티 플랫폼 빌드, 테스트 및 배포 가이드 (`BUILD_AND_RELEASE.md`)

본 문서는 **Windows (.exe)**, **macOS (.tar.gz)**, **Linux (.tar.gz)** 환경에서 `ec2pass` 및 `multipass`를 빌드하고 배포 패키지를 생성하는 표준 규격서입니다.

---

## 1. 사전 요구사항 (Prerequisites)

### A. 공통 요구사항
* **Rust Toolchain**: 1.75 이상 (`rustc`, `cargo`)
* **Git**

### B. 필수 하이퍼바이저 (시뮬레이션 모드 금지)
* **Linux**: KVM (`/dev/kvm`), `qemu-kvm`, `qemu-img`
* **macOS**: `Hypervisor.framework (HVF)` 지원 CPU, `qemu` (`brew install qemu`)
* **Windows**: `Hyper-V` 또는 `HypervisorPlatform (WHPX)` 또는 `QEMU`

---

## 2. 컴파일 및 테스트

### A. 컴파일
```bash
cargo build --workspace --release
```
생성되는 바이너리:
* `ec2pass` (또는 `ec2pass.exe`) - CLI 도구
* `multipass` (또는 `multipass.exe`) - Multipass 호환 별칭
* `ec2passd` (또는 `ec2passd.exe`) - 오케스트레이터 데몬

### B. 단위 테스트
```bash
cargo test --workspace
```

---

## 3. 멀티 플랫폼 패키징 (Packaging for Distribution)

### A. Windows 배포 산출물: `ec2pass-setup.exe` (.exe)
```powershell
.\packaging\build_windows_installer.ps1
```
* 산출물: `dist\ec2pass-setup.exe`
* 독립 실행형 설치 마법사로, Hyper-V 상태를 자동 검사하고 사용자 환경변수 `PATH`를 등록합니다.

### B. macOS 배포 산출물: `ec2pass-v1.0.0-darwin-<arch>.tar.gz` (.tar.gz)
```bash
./packaging/build_release.sh
```
* 산출물: `dist/ec2pass-v1.0.0-darwin-<arch>.tar.gz`
* 포함 내용: 바이너리 3종, `install.sh` (HVF 사전점검 포함), Launchd 데몬 plist

### C. Linux 배포 산출물: `ec2pass-v1.0.0-linux-<arch>.tar.gz` (.tar.gz)
```bash
./packaging/build_release.sh
```
* 산출물: `dist/ec2pass-v1.0.0-linux-<arch>.tar.gz`
* 포함 내용: 바이너리 3종, `install.sh` (KVM 사전점검 포함), Systemd 서비스 유닛
