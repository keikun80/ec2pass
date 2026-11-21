# InhausLab ec2pass (`ec2pass` / `multipass`)
## 크로스 플랫폼 빌드, 테스트 및 배포 가이드

본 문서는 Linux, macOS, Windows에서 `ec2pass` 및 `multipass`를 직접 컴파일하고, 테스트를 수행하며, 패키징하여 배포하는 표준 절차를 안내합니다.

---

## 1. 사전 요구사항 (Prerequisites)

### A. 공통 요구사항
* **Rust Toolchain**: 1.75 이상 (`rustup default stable`)
* **Git**

### B. OS별 가상화 및 하이퍼바이저 요구사항
* **Linux (Ubuntu / Debian / RHEL / Fedora / AL2023)**:
  * QEMU 및 KVM 모듈:
    ```bash
    # Ubuntu / Debian
    sudo apt-get update && sudo apt-get install -y qemu-system-x86 qemu-utils libvirt-daemon-system

    # RHEL / Fedora / Amazon Linux 2023
    sudo dnf install -y qemu-kvm qemu-img
    ```
  * 현재 사용자를 `kvm` 그룹에 추가:
    ```bash
    sudo usermod -aG kvm $USER
    ```
* **macOS (Apple Silicon M1~M4 & Intel)**:
  * Homebrew를 통해 QEMU 설치:
    ```bash
    brew install qemu
    ```
  * macOS는 커널 기본 `Hypervisor.framework (HVF)`를 사용하므로 별도의 커널 모듈 설정 불필요.
* **Windows (10 / 11 Pro, Enterprise)**:
  * Hyper-V 기능 활성화 또는 QEMU 설치.

---

## 2. 컴파일 (Compilation)

### A. 개발 빌드 (Debug)
```bash
cargo build --workspace
```

### B. 프로덕션 릴리스 빌드 (Release)
```bash
cargo build --workspace --release
```
빌드가 완료되면 `target/release/` 디렉터리에 다음 바이너리가 생성됩니다:
* `ec2pass` (또는 `ec2pass.exe`) - 메인 CLI
* `multipass` (또는 `multipass.exe`) - Canonical Multipass 호환 CLI 별칭
* `ec2passd` (또는 `ec2passd.exe`) - 가상 머신 백그라운드 오케스트레이터 데몬

---

## 3. 테스트 (Testing & E2E Verification)

### A. Rust 단위 및 통합 테스트
```bash
cargo test --workspace
```

### B. 전체 생명주기 E2E 통합 테스트 (자동 검증)
* **Linux / macOS**:
  ```bash
  chmod +x ./tests/e2e_test.sh
  ./tests/e2e_test.sh
  ```
* **Windows (PowerShell)**:
  ```powershell
  .\tests\e2e_test.ps1
  ```
7단계 생명주기(`version` $\rightarrow$ `find` $\rightarrow$ `networks` $\rightarrow$ `launch` $\rightarrow$ `list/info` $\rightarrow$ `exec (whoami/os-release)` $\rightarrow$ `stop/snapshot/delete/purge`)가 자동으로 실행되고 검증됩니다.

---

## 4. 배포용 패키징 (Packaging for Distribution)

### A. Linux 배포 패키징
1. **원클릭 인스톨러 사용**:
   ```bash
   sudo ./packaging/linux/install.sh
   ```
   바이너리를 `/usr/local/bin`에 설치하고 systemd 서비스(`ec2passd.service`)를 자동 등록 및 활성화합니다.
2. **독립 실행 압축본 생성 (`.tar.gz`)**:
   ```bash
   mkdir -p dist/ec2pass-linux-x86_64
   cp target/release/ec2pass target/release/multipass target/release/ec2passd dist/ec2pass-linux-x86_64/
   cp packaging/linux/ec2passd.service packaging/linux/install.sh README.md MANUAL.md dist/ec2pass-linux-x86_64/
   tar -czvf ec2pass-v1.14.0-linux-x86_64.tar.gz -C dist/ec2pass-linux-x86_64 .
   ```

### B. macOS 배포 패키징
1. **Apple Silicon & Intel Universal Binary 생성 (선택 사항)**:
   ```bash
   lipo -create -output ec2pass target/aarch64-apple-darwin/release/ec2pass target/x86_64-apple-darwin/release/ec2pass
   lipo -create -output ec2passd target/aarch64-apple-darwin/release/ec2passd target/x86_64-apple-darwin/release/ec2passd
   ln -s ec2pass multipass
   ```
2. **Homebrew 배포**:
   `packaging/macos/ec2pass.rb`를 자신의 Homebrew tap 저장소(`homebrew-ec2pass`)에 등록하여 `brew install ec2pass`로 원클릭 배포 가능.

### C. Windows 배포 패키징
```powershell
.\packaging\windows\install.ps1
```
`ec2pass.exe`, `multipass.exe`, `ec2passd.exe`를 Program Files(또는 LocalAppData)에 복사하고 사용자 환경변수 `PATH`에 자동 등록합니다.

---

## 5. GitHub Actions를 통한 완전 자동화 배포

프로젝트 루트의 `.github/workflows/` 디렉터리에 다음 워크플로가 포함되어 있습니다:
* **`ci.yml`**: PR 및 푸시 시 `ubuntu-latest`, `macos-14`(M-series), `macos-13`(Intel), `windows-latest` 4개 환경에서 병렬 컴파일 및 단위 테스트 자동 수행.
* **`release.yml`**: `git tag v1.14.0` 푸시 시 4개 플랫폼용 아카이브(`.tar.gz`, `.zip`)와 SHA-256 체크섬을 빌드하여 GitHub Release에 자동 배포.
