# InhausLab ec2pass (`ec2pass` / `multipass`) CLI 매뉴얼

본 매뉴얼은 Canonical Multipass와 100% 명령어 호환성을 제공하는 **InhausLab ec2pass (Amazon Linux 2023 가상화 오케스트레이터)**의 모든 서브커맨드, 옵션, 출력 포맷 및 실전 예제를 다룹니다.

> **[고지]** `InhausLab ec2pass`는 Canonical Ltd.와 무관한 독립 프로젝트입니다. `multipass` 실행 파일은 기존 스크립트 호환성을 위한 기능적 별칭(Drop-in Alias)으로 제공됩니다. 상세 법적 권리 및 상표권 성명은 [4. 법적 고지 및 상표권 성명](#4-법적-고지-및-상표권-성명-legal-notice--trademark-disclaimer)을 참조하십시오.

---

## 1. 전역 옵션 (Global Options)

```bash
ec2pass [OPTIONS] <COMMAND>
```

| 옵션 | 환경변수 | 기본값 | 설명 |
| :--- | :--- | :--- | :--- |
| `--daemon-addr <ADDR>` | `MP_DAEMON_ADDR` | `127.0.0.1:50051` | 백그라운드 데몬(`ec2passd`)의 IPC 소켓 주소 |
| `-v, --verbose` | - | 0 | 출력 상세도 증가 (`-v`, `-vv`, `-vvv`) |
| `-h, --help` | - | - | 도움말 출력 |
| `-V, --version` | - | - | 버전 정보 출력 |

---

## 2. 전체 서브커맨드 레퍼런스

### 2.1 `find` - 사용 가능한 이미지 카탈로그 검색
공식 AWS Amazon Linux 2023 CDN에서 지원하는 OS 이미지 목록을 조회합니다.

```bash
ec2pass find [SEARCH_STRING] [--show-unsupported] [--force-update] [--format <FORMAT>]
```
* **인자/옵션**:
  * `[SEARCH_STRING]`: 검색 키워드 (예: `minimal`, `kernel`, `al2`)
  * `--force-update`: AWS CDN 최신 릴리스 디렉터리를 강제로 다시 스크래핑
  * `--format <FORMAT>`: 출력 포맷 (`table`, `json`, `yaml`, `csv`)
* **예제**:
  ```bash
  # 전체 AL2023 이미지 목록 조회
  ec2pass find

  # JSON 포맷으로 필터링
  ec2pass find minimal --format json
  ```

---

### 2.2 `launch` - 새 인스턴스 생성 및 부팅
Amazon Linux 2023 베이스 이미지를 기반으로 CoW QCOW2 오버레이 디스크와 NoCloud `seed.iso`를 생성하고 인스턴스를 즉시 부팅합니다.

```bash
ec2pass launch [IMAGE] [OPTIONS]
```
* **옵션**:
  * `[IMAGE]`: 이미지 별칭 (기본값: `al2023` / `default` / `latest`). 지원 별칭: `al2023`, `al2023-k6.12`, `al2023-minimal`, `al2`, 또는 커스텀 파일 경로/URL
  * `-n, --name <NAME>`: 인스턴스 이름 (미지정 시 `bouncy-lynx` 등 고유 petname 자동 생성)
  * `-c, --cpus <COUNT>`: vCPU 할당 수 (기본값: `1`)
  * `-m, --memory <SIZE>`: 메모리 크기 (기본값: `1G`, 예: `512M`, `2G`, `4GiB`)
  * `-d, --disk <SIZE>`: 루트 디스크 크기 (기본값: `5G`, 예: `10G`, `20GiB`)
  * `--cloud-init <FILE|- >`: 주입할 cloud-config YAML 파일 경로 (또는 표준입력 `-`)
  * `--mount <LOCAL:GUEST>`: 인스턴스 생성과 동시에 호스트 디렉터리 마운트
  * `--timeout <SECS>`: 최대 완료 대기 시간 (초 단위, 기본값: `300`)
* **예제**:
  ```bash
  # 기본 스펙으로 인스턴스 빠른 실행
  ec2pass launch

  # 커스텀 사양 및 이름 지정
  ec2pass launch al2023 -n ec2-dev --cpus 2 --memory 4G --disk 15G

  # cloud-init YAML 파일과 호스트 디렉터리 동시 연결
  ec2pass launch al2023 -n web-srv --cloud-init ./init.yaml --mount C:\my-web:/var/www/html
  ```

---

### 2.3 `list` - 인스턴스 또는 스냅샷 목록 조회
현재 관리 중인 인스턴스의 실행 상태, IPv4 주소, 이미지 정보 또는 스냅샷 목록을 확인합니다.

```bash
ec2pass list [--format <FORMAT>] [--snapshots]
```
* **옵션**:
  * `--format <FORMAT>`: `table` (기본값), `json`, `yaml`, `csv`
  * `--snapshots`: 인스턴스 대신 생성된 모든 스냅샷 트리 출력
* **예제**:
  ```bash
  ec2pass list
  ec2pass list --format json
  ec2pass list --snapshots
  ```

---

### 2.4 `info` - 인스턴스 상세 정보 조회
vCPU 수, 실시간 메모리/디스크 사용량, 부하율(Load Average), 마운트 목록, 스냅샷 정보를 확인합니다.

```bash
ec2pass info [NAMES...] [--all] [--format <FORMAT>]
```
* **예제**:
  ```bash
  ec2pass info ec2-dev
  ec2pass info --all --format yaml
  ```

---

### 2.5 `exec` - 인스턴스 내부 명령 실행
실행 중인 인스턴스에 SSH 또는 내부 세션을 통해 명령을 전달하고 표준 출력/에러 및 종료 코드를 반환합니다.

```bash
ec2pass exec <NAME> [-d <DIR>] -- <COMMAND...>
```
* **옵션**:
  * `-d, --working-directory <DIR>`: 실행 전 게스트 내부 작업 디렉터리 지정
* **예제**:
  ```bash
  # Amazon Linux 2023 OS 릴리스 확인
  ec2pass exec ec2-dev -- cat /etc/os-release

  # 패키지 업데이트 및 설치
  ec2pass exec ec2-dev -- sudo dnf install -y htop

  # 특정 디렉터리에서 명령 실행
  ec2pass exec ec2-dev -d /var/log -- ls -la
  ```

---

### 2.6 `shell` - 대화형 셸 접속
대상 인스턴스에 `ec2-user`로 인터랙티브 터미널 세션을 연결합니다.

```bash
ec2pass shell [NAME]
```
* 대상 이름 생략 시 기본 인스턴스(`primary`)에 접속하며, `primary`가 없을 경우 Amazon Linux 2023으로 자동 런치 후 진입합니다.

---

### 2.7 `start` / `stop` / `restart` / `suspend` - 생명주기 제어

```bash
# 인스턴스 시작
ec2pass start [NAMES...] [--all]

# 인스턴스 정상 종료 (ACPI system_powerdown)
ec2pass stop [NAMES...] [--all]
ec2pass stop ec2-dev --force           # 강제 즉각 종료
ec2pass stop ec2-dev -t 10             # 10분 후 예약 종료
ec2pass stop ec2-dev -c                # 예약 종료 취소

# 인스턴스 재부팅
ec2pass restart [NAMES...] [--all]

# 인스턴스 일시 정지 (메모리 상태 동결)
ec2pass suspend [NAMES...] [--all]
```

---

### 2.8 `transfer` - 호스트 $\leftrightarrow$ 게스트 간 파일 전송
호스트와 인스턴스 내부 경로(`instance:path`) 간 파일이나 디렉터리를 고속으로 전송합니다.

```bash
ec2pass transfer [OPTIONS] <SRC...> <DST>
```
* **옵션**:
  * `-r, --recursive`: 디렉터리 재귀 복사
  * `-p, --parents`: 대상 경로의 부모 디렉터리 자동 생성
* **예제**:
  ```bash
  # 호스트 -> 인스턴스 파일 복사
  ec2pass transfer ./app.jar ec2-dev:/home/ec2-user/app.jar

  # 인스턴스 -> 호스트 파일 다운로드
  ec2pass transfer ec2-dev:/var/log/audit.log ./audit.log

  # 폴더 전체 전송
  ec2pass transfer -r ./src-dir ec2-dev:/home/ec2-user/project
  ```

---

### 2.9 `mount` & `umount` - 실시간 디렉터리 마운트
호스트의 로컬 폴더를 게스트 내부 경로에 마운트하여 실시간으로 코드와 파일을 공유합니다.

```bash
# 마운트 등록
ec2pass mount <HOST_PATH> <INSTANCE>[:<GUEST_PATH>]

# 마운트 해제
ec2pass umount <INSTANCE>[:<GUEST_PATH>]
```
* **예제**:
  ```bash
  ec2pass mount C:\workspace ec2-dev:/home/ec2-user/workspace
  ec2pass umount ec2-dev:/home/ec2-user/workspace
  ```

---

### 2.10 `snapshot` & `restore` - 스냅샷 및 롤백
중지된(`Stopped`) 인스턴스의 디스크 상태와 하드웨어 구성을 보존하고 원하는 시점으로 복원합니다.

```bash
# 스냅샷 생성
ec2pass snapshot <INSTANCE> [-n <NAME>] [-c <COMMENT>]

# 스냅샷으로 복원
ec2pass restore <INSTANCE>.<SNAPSHOT_NAME>
```
* **예제**:
  ```bash
  ec2pass stop ec2-dev
  ec2pass snapshot ec2-dev --name pre-upgrade --comment "커널 업데이트 전 백업"
  ec2pass restore ec2-dev.pre-upgrade
  ```

---

### 2.11 `clone` - 인스턴스 복제
기존에 중지된 인스턴스의 디스크와 설정을 독립된 새 인스턴스로 복제합니다.

```bash
ec2pass clone <SOURCE_NAME> [-n <NEW_NAME>]
```
* **예제**:
  ```bash
  ec2pass stop ec2-dev
  ec2pass clone ec2-dev -n ec2-staging
  ```

---

### 2.12 `delete` / `recover` / `purge` - 휴지통 및 영구 삭제
실수로 인한 데이터 유실을 방지하는 2단계 삭제 모델을 제공합니다.

```bash
# 1. 인스턴스를 Deleted (휴지통) 상태로 변경
ec2pass delete <NAMES...> [--all]

# 즉시 영구 삭제 옵션
ec2pass delete ec2-dev --purge

# 2. Deleted 상태의 인스턴스 복원
ec2pass recover <NAMES...> [--all]

# 3. Deleted 상태의 모든 인스턴스를 디스크에서 영구 제거
ec2pass purge
```

---

### 2.13 `get` & `set` - 동적 설정 및 리소스 크기 조정
Multipass의 계층형 설정 트리를 조회하고, 중지된 인스턴스의 리소스(vCPU, RAM, 디스크)를 동적으로 변경합니다.

```bash
# 사용 가능한 전체 설정 키 목록 조회
ec2pass get --keys

# 현재 설정값 조회
ec2pass get local.ec2-dev.cpus
ec2pass get local.ec2-dev.memory

# 중지된 VM의 vCPU 및 메모리/디스크 크기 변경
ec2pass stop ec2-dev
ec2pass set local.ec2-dev.cpus=4
ec2pass set local.ec2-dev.memory=8G
ec2pass set local.ec2-dev.disk=30G
ec2pass start ec2-dev
```

---

### 2.14 `alias` / `unalias` / `aliases` - 게스트 명령어 별칭
인스턴스 내부의 명령어를 호스트 환경에서 마치 로컬 프로그램처럼 실행할 수 있도록 연결합니다.

```bash
# ec2-dev 인스턴스의 dnf 명령어를 호스트 별칭으로 등록
ec2pass alias ec2-dev:dnf

# 등록된 별칭 목록 조회
ec2pass aliases

# 별칭 제거
ec2pass unalias dnf
```

---

### 2.15 `networks` - 가상 네트워크 인터페이스 조회
호스트의 브리지 및 NAT 가상 네트워크 상태를 확인합니다.

```bash
ec2pass networks [--format <FORMAT>]
```

---

## 3. 실전 활용 레시피 (Cookbook)

### 레시피 1: AWS EC2 배포 전 로컬에서 `cloud-init` 검증
```yaml
# cloud-config.yaml
#cloud-config
packages:
  - nginx
  - git
write_files:
  - path: /var/www/html/index.html
    content: "<h1>Amazon Linux 2023 on Local Multipass</h1>"
runcmd:
  - systemctl enable --now nginx
```

```bash
# 위 설정을 주입하여 인스턴스 런치
ec2pass launch al2023 -n ec2-web --cloud-init cloud-config.yaml
ec2pass exec ec2-web -- curl -s http://localhost
```

### 레시피 2: 로컬 IDE와 연동된 실시간 개발 환경 구축
```bash
# 인스턴스 생성 및 호스트 소스 코드 디렉터리 마운트
ec2pass launch al2023 -n dev-env --cpus 2 --memory 4G
ec2pass mount C:\Projects\MyService dev-env:/home/ec2-user/service

# 인스턴스 안에서 빌드 및 테스트 수행
ec2pass exec dev-env -d /home/ec2-user/service -- cargo test
```

---

## 4. 법적 고지 및 상표권 성명 (Legal Notice & Trademark Disclaimer)

### 4.1 상표권 및 비제휴 성명 (Trademark Notice & Non-Affiliation)
* **Multipass** 및 **Ubuntu**는 Canonical Ltd.의 등록 상표 또는 상표입니다.
* 본 프로젝트(**`InhausLab ec2pass`**)는 **Canonical Ltd.와 어떠한 제휴, 후원, 보증 또는 공식적인 인가 관계도 없는 독립적인 서드파티 오픈소스 소프트웨어**입니다.
* 본 문서 및 소프트웨어 전반에서 언급되는 "Multipass", "Ubuntu" 등의 상표는 오직 **명령어 체계 호환성 및 상호운용성(Interoperability) 대상을 식별·설명하기 위한 '명칭적 공정 이용(Nominative Fair Use)' 목적**으로만 사용됩니다.

### 4.2 호환성 명령어 별칭(Drop-in Alias)에 관한 성명
* 배포 패키지에 포함되거나 심볼릭 링크로 생성되는 `multipass` 실행 파일은 **독립된 제품 브랜드를 주장하기 위한 것이 아니며**, 기존 Canonical Multipass 사용자의 쉘 스크립트 마이그레이션과 워크플로 연속성을 지원하기 위한 **순수 기능적 명령어 별칭(Drop-in Compatibility Alias)**입니다.
* 본 프로젝트의 공식적인 독립 제품 고유 명칭은 **`InhausLab ec2pass`**입니다.

### 4.3 저작권 및 클린룸 독자 구현 (Copyright & Clean-Room Implementation)
* 본 소프트웨어는 Canonical의 Multipass C++ 소스코드를 복제·수정하거나 포함하지 않았으며, 공개된 CLI 규격 및 프로토콜 인터페이스를 바탕으로 **Rust 언어로 밑바닥부터 100% 독자 작성된 독립 저작물(Clean-room Reimplementation)**입니다.
* 명령어 인터페이스(CLI 문법, 옵션 체계)는 글로벌 판례(*Google LLC v. Oracle America, Inc.*) 및 국제 저작권법상 기능적 인터페이스 및 상호운용성 원칙에 따라 저작권 침해 대상이 아닌 공정 이용 영역에 해당합니다.

### 4.4 Amazon Linux 관련 상표권 고지
* **Amazon Linux**, **Amazon Web Services**, **AWS**, **EC2**는 Amazon.com, Inc. 또는 그 계열사의 상표입니다.
* 본 프로젝트는 AWS 공식 CDN에서 공개 배포되는 정식 KVM 이미지를 다운로드 및 오케스트레이션하는 기능을 제공하며, Amazon과의 공식 제휴 또는 보증을 의미하지 않습니다.

---

## 5. 설치 및 언인스톨 가이드 (Installation & Uninstallation Guide)

### 5.1 Windows 환경

#### 설치 (Installation)
1. **독립 설치 관리자 (권장)**: `ec2pass-setup.exe`를 더블 클릭하여 실행하면 자동으로 `%LOCALAPPDATA%\Programs\ec2pass`에 파일이 복사되고 사용자 `PATH`에 등록됩니다.
2. **배치/스크립트 설치**:
   ```cmd
   Setup.bat
   ```
   또는 PowerShell:
   ```powershell
   powershell -ExecutionPolicy Bypass -File install.ps1
   ```

#### 언인스톨 (Uninstallation)
`ec2pass`는 깔끔하고 안전한 언인스톨 기능을 완벽히 지원합니다. 다음 세 가지 방법 중 하나로 제거할 수 있습니다:

1. **Windows 설정 / 제어판에서 제거 (표준 GUI 방식)**:
   * **Windows 설정** (`Win + I`) > **앱** > **설치된 앱** (또는 제어판 > **프로그램 추가/제거**)으로 이동합니다.
   * 목록에서 **`InhausLab ec2pass`**를 찾아 **[제거]**를 클릭합니다.
2. **설치 폴더의 배치 파일 실행**:
   * 설치 디렉터리(`%LOCALAPPDATA%\Programs\ec2pass`) 내의 **`Uninstall.bat`** 파일을 더블 클릭합니다.
3. **PowerShell 스크립트 실행 (데이터 영구 삭제 옵션 지원)**:
   ```powershell
   # 1) 프로그램 및 환경변수만 제거 (가상머신 데이터는 보존)
   powershell -ExecutionPolicy Bypass -File uninstall.ps1

   # 2) 가상머신 이미지 및 캐시 데이터까지 모두 영구 삭제
   powershell -ExecutionPolicy Bypass -File uninstall.ps1 -PurgeData
   ```

* **언인스톨러 동작 내용**:
  * 실행 중인 `ec2passd` 백그라운드 데몬 및 프로세스 즉시 종료
  * 시스템 및 사용자 환경변수(`PATH`)에서 `ec2pass` 경로 자동 제거
  * 시작 메뉴 바로가기 아이콘 제거
  * Windows 레지스트리(설치된 앱 목록) 항목 제거
  * 설치 디렉터리 및 바이너리 파일 영구 삭제

---

### 5.2 Linux & macOS 환경

#### 언인스톨 (Uninstallation)
Linux와 macOS에서도 제공되는 `uninstall.sh` 스크립트를 통해 원클릭으로 완벽히 제거할 수 있습니다:

```bash
# 기본 제거 (데몬 중지/등록 해제 및 바이너리 삭제, 인스턴스 데이터 보존)
sudo ./packaging/linux/uninstall.sh    # Linux
sudo ./packaging/macos/uninstall.sh    # macOS

# 가상머신 디스크 및 캐시까지 완전히 삭제하고자 하는 경우
sudo ./packaging/linux/uninstall.sh --purge-data
sudo ./packaging/macos/uninstall.sh --purge-data
```

---

## 6. 설치 디렉터리 및 데이터 저장소 구조 (Directory & Storage Architecture)

`InhausLab ec2pass`는 실행 바이너리와 영구 가상머신 데이터가 명확히 분리된 **엔터프라이즈급 2계층 저장소 아키텍처**를 가집니다. 프로그램을 업데이트하거나 재설치하더라도 기존 가상머신 데이터와 SSH 키페어는 안전하게 보존됩니다.

```
┌────────────────────────────────────────────────────────┐
│  1. 실행 바이너리 디렉터리 (Program Install Directory)  │
│  - ec2pass.exe, ec2passd.exe, multipass.exe            │
│  - 언인스톨러 스크립트, 공식 문서                       │
└────────────────────────────────────────────────────────┘
                           ▲
                           │  호출 및 오케스트레이션
                           ▼
┌────────────────────────────────────────────────────────┐
│  2. 데이터 및 가상머신 저장소 (Data & Instance Store)   │
│  - AWS AL2023 원본 베이스 KVM 이미지 캐시              │
│  - 인스턴스별 CoW 오버레이 QCOW2 가상 디스크           │
│  - cloud-init seed.iso 및 전용 SSH 키페어               │
│  - 인스턴스 상태 메타데이터 (state.json)               │
└────────────────────────────────────────────────────────┘
```

---

### 6.1 프로그램 실행 바이너리 디렉터리 (Program Directory)

실행 파일과 언인스톨 스크립트가 위치하며, 시스템 `PATH` 환경변수에 자동 등록되는 디렉터리입니다.

| OS 환경 | 구분 | 디렉터리 경로 |
| :--- | :--- | :--- |
| **Windows** | 일반 사용자 (기본) | `%LOCALAPPDATA%\Programs\ec2pass`<br>*(예: `C:\Users\<사용자명>\AppData\Local\Programs\ec2pass`)* |
| **Windows** | 관리자 권한 설치 시 | `%ProgramFiles%\ec2pass`<br>*(예: `C:\Program Files\ec2pass`)* |
| **Linux** | 시스템 전역 | `/usr/local/bin/` |
| **macOS** | 시스템 전역 | `/usr/local/bin/` |

#### Windows 설치 디렉터리 내부 구조
```text
%LOCALAPPDATA%\Programs\ec2pass\
├── ec2pass.exe        # InhausLab ec2pass 메인 CLI 클라이언트
├── ec2passd.exe       # 백그라운드 가상화 관리 데몬
├── multipass.exe      # 기존 Canonical Multipass 명령 100% 호환용 드롭인 별칭
├── Uninstall.bat      # 원클릭 마우스 더블클릭 안전 제거 배치 파일
├── uninstall.ps1      # 파워셸 언인스톨러 (-PurgeData 지원)
├── README.md          # 제품 개요 및 법적 고지문
└── MANUAL.md          # 100% 상세 CLI 전체 매뉴얼
```

---

### 6.2 데이터 및 가상머신 저장소 (Data & Storage Directory)

가상머신 인스턴스, 스냅샷, 다운로드된 AWS 공식 Amazon Linux 2023 KVM 이미지, 암호화 키페어가 영구 저장되는 위치입니다.

| OS 환경 | 기본 데이터 디렉터리 경로 |
| :--- | :--- |
| **Windows** | `%LOCALAPPDATA%\ec2pass`<br>*(예: `C:\Users\<사용자명>\AppData\Local\ec2pass`)* |
| **Linux** | `/var/lib/ec2pass` (루트 데몬) 또는 `~/.local/share/ec2pass` |
| **macOS** | `~/Library/Application Support/ec2pass`<br>*(Launchd 루트 데몬 시: `/var/root/Library/Application Support/ec2pass`)* |

> **[팁: 데이터 저장 경로 사용자 정의]**  
> 용량이 큰 드라이브(예: `D:\ec2pass-data`)에 가상머신을 저장하고자 하는 경우, 환경 변수 **`EC2PASS_HOME`**을 설정하면 해당 경로가 데이터 디렉터리로 최우선 적용됩니다.

#### 데이터 디렉터리 세부 구조 및 파일 역할
```text
%LOCALAPPDATA%\ec2pass\
│
├── cache\
│   └── images\
│       ├── al2023-kvm-2023.6.20250128.0-kernel-6.1-x86_64.x86_64.qcow2
│       │   # AWS 공식 CDN에서 다운로드한 무변형 정식 KVM 원본 베이스 이미지 (읽기 전용 보호)
│       └── ...
│
├── instances\
│   ├── dev-env\
│   │   ├── disk.qcow2    # 원본 베이스 이미지를 부모(backing file)로 참조하는 CoW 델타 오버레이 디스크
│   │   └── seed.iso      # 순수 Rust로 직접 생성된 NoCloud cloud-init 시드 이미지 (볼륨명: cidata)
│   │
│   └── test-kernel612\
│       ├── disk.qcow2
│       └── seed.iso
│
├── ssh-keys\
│   ├── id_ed25519_ec2pass      # 데몬이 인스턴스에 비밀번호 없이 안전 접속하기 위한 ED25519 개인키
│   └── id_ed25519_ec2pass.pub  # 인스턴스 생성 시 seed.iso의 authorized_keys에 자동 주입되는 공개키
│
└── state.json
    # 등록된 인스턴스 상태(RUNNING, STOPPED, DELETED), IP, 리소스(vCPU, RAM, Disk), 마운트 정보가 저장되는 원자적 JSON DB
```


