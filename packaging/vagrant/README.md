# InhausLab ec2pass - Vagrant 기반 EC2 (Amazon Linux 2023) 빌드 및 테스트 환경

본 디렉터리는 Windows 개발 환경에서 **Vagrant를 사용하여 Amazon Linux 2023 / Enterprise Linux 9 가상머신을 자동 프로비저닝하고, 그 안에서 `ec2pass`의 소스코드 빌드 및 Linux 네이티브 E2E 통합 테스트를 원클릭으로 수행**할 수 있는 도구 모음입니다.

---

## 1. 사전 준비 (Prerequisites)

Windows 호스트에서 터미널(PowerShell)을 열고 아래 명령어로 Vagrant와 가상화 프로바이더(VirtualBox)를 설치합니다:

```powershell
# 1. Vagrant CLI 설치
winget install HashiCorp.Vagrant

# 2. VirtualBox 가상화 프로바이더 설치
winget install Oracle.VirtualBox
```
> *설치 완료 후 새 PowerShell 창을 열어 환경변수를 갱신합니다.*

---

## 2. 원클릭 자동 실행 (PowerShell Runner)

프로젝트 루트에서 제공되는 호스트 자동화 스크립트를 실행하면 **가상머신 기동 -> 환경 세팅 -> 소스코드 컴파일 -> 유닛 테스트 -> E2E 통합 테스트 -> 자동 종료**까지 전자동으로 수행됩니다:

```powershell
# 기본 실행 (테스트 완료 후 VM 자동 일시정지)
powershell -ExecutionPolicy Bypass -File packaging\vagrant\run_vagrant_test.ps1

# 테스트 후 VM을 계속 띄워두고 직접 SSH로 디버깅하고 싶은 경우
powershell -ExecutionPolicy Bypass -File packaging\vagrant\run_vagrant_test.ps1 -KeepRunning

# 테스트 완료 즉시 가상머신을 완전히 파기(Clean-room CI 모드)
powershell -ExecutionPolicy Bypass -File packaging\vagrant\run_vagrant_test.ps1 -DestroyOnFinish
```

---

## 3. 수동 CLI 조작 가이드

직접 Vagrant 명령어를 제어하며 단계별로 작업하고 싶을 때 사용합니다:

### 3.1 가상머신 시작 및 프로비저닝
```bash
# 가상머신 생성 및 부팅 (초회 실행 시 박스 다운로드 및 Rust/QEMU 자동 설치)
vagrant up

# CPU/메모리를 늘려 실행하고자 하는 경우
$env:VM_CPUS = "4"
$env:VM_MEMORY = "8192"
vagrant up
```

### 3.2 가상머신 내부 SSH 접속 및 테스트 실행
```bash
# 게스트 SSH 접속
vagrant ssh

# 게스트 내부에서 원클릭 빌드 & 7단계 E2E 테스트 실행
bash /vagrant/packaging/vagrant/build_and_test.sh
```

### 3.3 가상머신 내부에서 수동 개발/디버깅
```bash
cd /vagrant

# 1. 릴리스 빌드
export CARGO_TARGET_DIR=~/target_cache
cargo build --workspace --release

# 2. 유닛 테스트
cargo test --workspace

# 3. 개발 중인 ec2pass 직접 실행
/usr/local/bin/ec2pass --version
/usr/local/bin/ec2pass find
/usr/local/bin/ec2pass launch al2023 -n dev-box
/usr/local/bin/ec2pass shell dev-box
```

### 3.4 환경 종료 및 리소스 해제
```bash
# 가상머신 중지 (상태 보존)
vagrant halt

# 가상머신 완전 삭제 (디스크 공간 회수)
vagrant destroy -f
```

---

## 4. 파일 구성 및 역할

| 파일명 | 역할 |
| :--- | :--- |
| **`Vagrantfile`** | 프로젝트 루트에 위치. vCPU/RAM 설정, 중첩 가상화(`--nested-hw-virt`) 활성화, `/vagrant` 공유 폴더 정의 |
| **`packaging/vagrant/provision.sh`** | 게스트 초기 부팅 시 1회 실행: DNF 빌드 도구, QEMU/KVM 패키지, Rust 툴체인 자동 설치 |
| **`packaging/vagrant/build_and_test.sh`** | 게스트 내부 실행: Rust 릴리스 컴파일, 유닛 테스트, 바이너리 배포, `tests/e2e_test.sh` 자동화 |
| **`packaging/vagrant/run_vagrant_test.ps1`** | Windows 호스트용 원클릭 전체 테스트 실행기 |
