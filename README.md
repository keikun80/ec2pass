# InhausLab ec2pass (`ec2pass` / `multipass`)

**InhausLab ec2pass**는 Canonical Multipass의 아키텍처를 분석하여, **Amazon Linux 2023 (AL2023)**을 100% 동일한 CLI/데몬 구조로 지원하도록 Rust로 구현한 경량 VM 오케스트레이터입니다.

---

## 1. 주요 특징

* **100% Canonical Multipass CLI 호환**:
  * `launch`, `start`, `stop`, `restart`, `suspend`, `delete`, `recover`, `purge`
  * `shell`, `exec`, `info`, `list`, `find`, `mount`, `umount`, `transfer`
  * `snapshot`, `restore`, `clone`, `get`, `set`, `alias`, `unalias`, `aliases`, `networks`, `version`
* **Amazon Linux 2023 공식 KVM 이미지 지원**:
  * AWS 공식 CDN (`https://cdn.amazonlinux.com/al2023/os-images/latest/kvm/`) 자동 탐색 및 캐싱
  * 기본 커널 6.1 LTS (`al2023`, `latest`, `lts`), 커널 6.12 (`next`), Minimal 경량 이미지 (`minimal`), AL2 (`amzn2`) 지원
  * 원본 베이스 이미지를 보호하고 인스턴스마다 CoW (Copy-on-Write) QCOW2 오버레이 디스크 생성
* **순수 Rust NoCloud `cloud-init` ISO9660 (`cidata`) 생성기**:
  * 외부 `mkisofs`나 `genisoimage` 설치 없이 순수 Rust 코드로 볼륨 레이블 `cidata`와 Rock Ridge 확장(`NM`, `PX`)을 포함한 `seed.iso` 바이너리를 직접 생성
  * Amazon Linux 2023의 기본 사용자 `ec2-user`, 데몬 전용 SSH 키페어, 패스워드 없는 sudo, `growpart`/`xfs_growfs` 자동 주입
* **Client-Daemon 아키텍처**:
  * 백그라운드 데몬 (`ec2passd`)과 CLI (`ec2pass` 및 `multipass`) 간 스트리밍 IPC
  * 데몬 미실행 시 CLI가 자동으로 `ec2passd` 백그라운드 프로세스를 구동
  * Linux(KVM), macOS(HVF+QEMU), Windows(Hyper-V/WHPX) 네이티브 하이퍼바이저 필수 지원 (부재 시 맞춤형 설치 가이드 자동 안내)
* **멀티 플랫폼 공식 배포 지원**:
  * **Windows**: 독립 실행 설치 파일 (`ec2pass-setup.exe`)
  * **macOS**: 유니버설 아카이브 (`ec2pass-v1.0.0-darwin-<arch>.tar.gz`)
  * **Linux**: 독립 아카이브 (`ec2pass-v1.0.0-linux-<arch>.tar.gz`)

---

## 2. 워크스페이스 구조

```text
c:\ec2_multipath\
├── Cargo.toml                  # Workspace 매니페스트
├── crates/
│   ├── ec2ps-iso9660/             # 순수 Rust ISO9660 (cidata) 생성 라이브러리
│   ├── ec2ps-protocol/            # IPC 프로토콜, 상태 DTO, Table/JSON/YAML/CSV 포맷터
│   ├── ec2ps-core/                # ImageVault (CDN 캐시), Cloud-Init, QEMU 드라이버, SSH, State Store
│   ├── ec2ps-daemon/              # 백그라운드 데몬 바이너리 (ec2passd)
│   └── ec2ps-cli/                 # CLI 바이너리 (ec2pass, multipass)
└── README.md
```

---

## 3. 빠른 시작 가이드

### 빌드
```bash
cargo build --release
```
빌드 완료 후 `target/release/` 디렉터리에 아래 실행 파일들이 생성됩니다:
* `ec2passd.exe` (백그라운드 데몬)
* `ec2pass.exe` (Amazon Linux 2023 전용 CLI)
* `multipass.exe` (Canonical Multipass 100% 호환 심볼릭/바이너리 별칭)

### 기본 사용법

```bash
# 1. 사용 가능한 Amazon Linux 2023 이미지 카탈로그 검색
ec2pass find

# 2. 기본 AL2023 인스턴스 생성 및 부팅 (이름 자동 생성)
ec2pass launch al2023

# 3. 사용자 정의 스펙 및 이름으로 인스턴스 생성
ec2pass launch al2023 -n my-al2023 --cpus 2 --memory 2G --disk 10G

# 4. 인스턴스 목록 및 상세 정보 조회
ec2pass list
ec2pass info my-al2023

# 5. 인스턴스 내부 명령 실행
ec2pass exec my-al2023 -- cat /etc/os-release
ec2pass exec my-al2023 -- uname -a

# 6. 호스트 디렉토리를 인스턴스에 마운트
ec2pass mount C:\my-project my-al2023:/home/ec2-user/project

# 7. 파일 전송 (호스트 <-> 게스트)
ec2pass transfer C:\local-file.txt my-al2023:/home/ec2-user/
ec2pass transfer my-al2023:/home/ec2-user/result.log C:\local-dir\

# 8. 인스턴스 스냅샷 및 롤백
ec2pass stop my-al2023
ec2pass snapshot my-al2023 --name snap-v1 --comment "설정 완료 후 백업"
ec2pass restore my-al2023.snap-v1

# 9. 인스턴스 삭제 및 영구 정리 (Soft-delete & Purge)
ec2pass delete my-al2023
ec2pass recover my-al2023   # 실수로 지운 경우 복원
ec2pass purge              # Deleted 상태의 인스턴스 영구 디스크 삭제
```

---

## 4. 설치 및 언인스톨 (Installation & Uninstallation)

### 설치 (Installation)
* **Windows**: `ec2pass-setup.exe` (1-Click 설치기) 실행 또는 `Setup.bat` 실행
* **Linux**: `sudo ./packaging/linux/install.sh`
* **macOS**: `sudo ./packaging/macos/install.sh`

### 언인스톨 (Uninstallation)
`ec2pass`는 시스템을 오염시키지 않는 안전하고 완전한 언인스톨을 지원합니다.
* **Windows GUI**: **Windows 설정 > 앱 > 설치된 앱**에서 `InhausLab ec2pass` 선택 후 **[제거]** 클릭
* **Windows CLI/배치**: 설치 폴더 내의 `Uninstall.bat` 실행 또는 `powershell -ExecutionPolicy Bypass -File uninstall.ps1 [-PurgeData]`
* **Linux**: `sudo ./packaging/linux/uninstall.sh [--purge-data]`
* **macOS**: `sudo ./packaging/macos/uninstall.sh [--purge-data]`

---

## 5. 법적 고지 및 상표권 성명 (Legal Notice & Trademark Disclaimer)

* **상표권 비제휴 성명**: **Multipass** 및 **Ubuntu**는 Canonical Ltd.의 등록 상표 또는 상표입니다. 본 프로젝트(**`InhausLab ec2pass`**)는 Canonical Ltd.와 어떠한 제휴, 후원 또는 공식 인가 관계도 없는 독립적인 서드파티 오픈소스 프로젝트입니다.
* **명칭적 공정 이용 (Nominative Fair Use)**: 본 프로젝트에서 "Multipass" 명칭은 오직 CLI 명령어 및 상호운용성(Interoperability) 호환 대상을 지칭하기 위한 설명 목적으로만 사용됩니다.
* **호환성 별칭 안내**: 패키지 내 `multipass` 실행 파일은 기존 사용자의 쉘 스크립트 마이그레이션을 위한 **기능적 별칭(Drop-in Compatibility Alias)**이며, 본 제품의 공식 고유 브랜드는 **`InhausLab ec2pass`**입니다.
* **클린룸 독자 구현**: 본 소프트웨어는 Canonical의 C++ 코드를 복제하지 않고, 공개된 규격에 따라 **Rust 언어로 밑바닥부터 100% 독자 작성된 독립 저작물(Clean-room Reimplementation)**입니다.
* **Amazon Linux 관련 고지**: **Amazon Linux**, **AWS**, **EC2**는 Amazon.com, Inc.의 상표이며, 본 프로젝트는 AWS 공식 CDN 공개 KVM 이미지 오케스트레이션 도구로서 Amazon과의 공식 제휴 관계가 없습니다.
