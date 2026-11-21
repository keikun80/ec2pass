# InhausLab ec2pass (`ec2pass` / `multipass`)
# 플랫폼별 컴파일, 테스트 및 배포 실행 가이드 (`platform.md`)

본 문서는 **GitHub Actions 등 외부 CI/CD에 의존하지 않고**, 각 플랫폼(Linux, macOS, Windows) 환경에서 개발자가 **직접 컴파일, 자동화 테스트 및 배포용 패키징을 로컬에서 독립적으로 실행**할 수 있는 표준 매뉴얼입니다.

---

## 1. 플랫폼별 지원 매트릭스

| 플랫폼 (OS) | 하이퍼바이저 백엔드 | 지원 아키텍처 | 기본 패키지 산출물 | 배포 서비스 형태 |
| :--- | :--- | :--- | :--- | :--- |
| **Linux** (Ubuntu, RHEL, Fedora, AL2023) | **KVM (`/dev/kvm`)** | `x86_64`, `aarch64` | `ec2pass-v1.14.0-linux-<arch>.tar.gz` | `systemd` (`ec2passd.service`) |
| **macOS** (Sequoia, Sonoma) | **Hypervisor.framework (`HVF`)** | `aarch64` (Apple Silicon M-series), `x86_64` (Intel) | `ec2pass-v1.14.0-darwin-<arch>.tar.gz` | `launchd` (`com.amazonlinux.ec2passd.plist`) |
| **Windows** (10, 11 Pro/Enterprise) | **Hyper-V / WHPX** | `x86_64` | `ec2pass-setup.exe` (독립 실행 설치 파일), `ec2pass-v1.14.0-windows-x86_64.zip` | Background Process / Start Menu / User PATH |

---

## 2. Linux 환경 실행 방법 (Ubuntu, Debian, RHEL, Fedora, Amazon Linux 2023)

### 2.1 사전 환경 준비 (1회)
```bash
# 1. Rust 툴체인 설치 (미설치 시)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 2. QEMU 및 KVM 가속기 설치
# [Ubuntu / Debian 계열]
sudo apt-get update && sudo apt-get install -y qemu-system-x86 qemu-utils libvirt-daemon-system

# [RHEL / Fedora / Amazon Linux 2023 계열]
sudo dnf install -y qemu-kvm qemu-img

# 3. 현재 사용자를 kvm 그룹에 추가 (재로그인 필요)
sudo usermod -aG kvm $USER
```

### 2.2 컴파일 (Compile)
```bash
# 릴리스 바이너리 빌드 (최적화)
cargo build --workspace --release
```
* 산출물: `target/release/ec2pass`, `target/release/multipass`, `target/release/ec2passd`

### 2.3 테스트 (Automated E2E Testing)
```bash
# 7단계 생명주기 자동 검증 스크립트 실행
chmod +x ./tests/e2e_test.sh
./tests/e2e_test.sh
```
* **검증 단계**: 버전 확인 $\rightarrow$ CDN 카탈로그 검색 $\rightarrow$ 네트워크 조회 $\rightarrow$ 인스턴스 런치 $\rightarrow$ 상태 및 실시간 스펙 조회 $\rightarrow$ 게스트 내부 명령 실행(`whoami`, `os-release`) $\rightarrow$ 스냅샷/삭제/영구정리(Purge).

### 2.4 배포용 패키징 (Local Release Packaging)
```bash
# 독립 배포용 아카이브 및 SHA-256 체크섬 자동 생성
chmod +x ./packaging/build_release.sh
./packaging/build_release.sh
```
* **결과물 생성**:
  * `ec2pass-v1.14.0-linux-x86_64.tar.gz` (또는 `aarch64`)
  * `ec2pass-v1.14.0-linux-x86_64.tar.gz.sha256`

### 2.5 대상 시스템 설치 (Installation)
```bash
# 바이너리를 /usr/local/bin에 복사하고 systemd 서비스를 자동 등록 및 시작
sudo ./packaging/linux/install.sh

# 설치 확인
ec2pass version
ec2pass find
```

---

## 3. macOS 환경 실행 방법 (Apple Silicon M1~M4 & Intel Mac)

### 3.1 사전 환경 준비 (1회)
```bash
# 1. Homebrew를 통해 QEMU 설치 (macOS는 HVF 가속 기본 탑재)
brew install qemu

# 2. Rust 툴체인 확인
rustc --version && cargo --version
```

### 3.2 컴파일 (Compile)
* **Apple Silicon (M1/M2/M3/M4) 네이티브 빌드**:
  ```bash
  cargo build --workspace --release
  ```
* **(선택 사항) Intel과 Apple Silicon을 합친 Universal Binary 빌드**:
  ```bash
  rustup target add aarch64-apple-darwin x86_64-apple-darwin
  cargo build --workspace --release --target aarch64-apple-darwin
  cargo build --workspace --release --target x86_64-apple-darwin

  mkdir -p target/universal
  lipo -create -output target/universal/ec2pass target/aarch64-apple-darwin/release/ec2pass target/x86_64-apple-darwin/release/ec2pass
  lipo -create -output target/universal/ec2passd target/aarch64-apple-darwin/release/ec2passd target/x86_64-apple-darwin/release/ec2passd
  ln -sf ec2pass target/universal/multipass
  ```

### 3.3 테스트 (Testing)
```bash
chmod +x ./tests/e2e_test.sh
./tests/e2e_test.sh
```

### 3.4 배포용 패키징 (Packaging)
```bash
chmod +x ./packaging/build_release.sh
./packaging/build_release.sh
```
* **결과물 생성**:
  * `ec2pass-v1.14.0-darwin-arm64.tar.gz` (또는 `x86_64`)
  * `ec2pass-v1.14.0-darwin-arm64.tar.gz.sha256`

### 3.5 대상 Mac 시스템 설치
```bash
# 바이너리를 /usr/local/bin에 설치하고 launchd 데몬 등록
sudo ./packaging/macos/install.sh

# 또는 Homebrew Formula를 통한 설치
# packaging/macos/ec2pass.rb 사용
```

---

## 4. Windows 환경 실행 방법 (Windows 10 / 11 Pro, Enterprise)

### 4.1 사전 환경 준비 (1회)
* **Hyper-V 활성화 (관리자 권한 PowerShell)**:
  ```powershell
  Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V -All
  ```
  *(재부팅 진행)*
* **Hyper-V 관리자 권한 부여**:
  ```powershell
  net localgroup "Hyper-V Administrators" $env:USERNAME /add
  ```

### 4.2 컴파일 (Compile)
```powershell
cargo build --workspace --release
```
* 산출물: `target\release\ec2pass.exe`, `target\release\multipass.exe`, `target\release\ec2passd.exe`

### 4.3 테스트 (Testing)
```powershell
.\tests\e2e_test.ps1
```
* 모든 7단계 수명주기 및 리눅스 게스트 응답 테스트 자동 검증 완료 확인.

### 4.4 배포용 설치 파일 및 패키징 빌드 (Installer & Packaging)
```powershell
.\packaging\build_windows_installer.ps1
```
* **생성되는 설치 파일 및 산출물 (`dist\` 디렉터리)**:
  * **`ec2pass-setup.exe`**: 더블 클릭으로 즉시 설치를 진행할 수 있는 **Windows 독립 실행형 단일 설치 파일 (Self-Extracting Setup Executable)**.
  * **`ec2pass-v1.14.0-windows-x86_64.zip`**: 무설치 포터블 압축 패키지 (1클릭 `Setup.bat` 및 `install.ps1` 내장).
  * **`*.sha256`**: 각 설치 파일의 SHA-256 무결성 검증 해시 파일.

### 4.5 대상 Windows PC 설치 및 제거 방법

#### 방법 A. GUI 단일 설치 파일 실행 (가장 간편한 방법)
1. `dist\ec2pass-setup.exe`를 더블 클릭하여 실행합니다.
2. 설치 진행 확인 대화상자에서 "예"를 누르면 자동으로 Hyper-V 환경 점검, 바이너리 배포, PATH 등록 및 시작 메뉴 바로가기 생성이 완료됩니다.

#### 방법 B. 1클릭 배치 스크립트 실행 (압축 패키지)
1. `dist\ec2pass-v1.14.0-windows-x86_64.zip`의 압축을 풉니다.
2. 폴더 내 `Setup.bat`을 더블 클릭(또는 관리자 권한으로 실행)합니다.

#### 방법 C. 무인(Silent) / 자동화 설치 (PowerShell)
```powershell
# 일반 사용자 설치 (%LOCALAPPDATA%\Programs\ec2pass)
powershell.exe -ExecutionPolicy Bypass -File .\packaging\windows\install.ps1 -Silent

# 특정 디렉터리 지정 설치
powershell.exe -ExecutionPolicy Bypass -File .\packaging\windows\install.ps1 -InstallDir "C:\Tools\ec2pass"
```

#### 프로그램 제거 (Uninstall)
* 설치 폴더 내 `Uninstall.bat` 더블 클릭 또는:
  ```powershell
  powershell.exe -ExecutionPolicy Bypass -File "%LOCALAPPDATA%\Programs\ec2pass\uninstall.ps1"
  ```

---

## 5. 배포 패키지 체크섬 무결성 검증 가이드

배포된 압축 파일을 사용자 시스템에 풀기 전, 패키지가 손상되지 않았는지 검증하는 방법:

* **Linux / macOS**:
  ```bash
  sha256sum -c ec2pass-v1.14.0-*.tar.gz.sha256
  # 또는
  shasum -a 256 -c ec2pass-v1.14.0-*.tar.gz.sha256
  ```
* **Windows (PowerShell)**:
  ```powershell
  $expected = (Get-Content .\ec2pass-v1.14.0-windows-x86_64.zip.sha256).Split(" ")[0]
  $actual = (Get-FileHash .\ec2pass-v1.14.0-windows-x86_64.zip -Algorithm SHA256).Hash.ToLower()
  if ($expected -eq $actual) { Write-Host "Checksum MATCHED!" -ForegroundColor Green }
  ```
