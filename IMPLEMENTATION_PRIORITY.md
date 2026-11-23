# ec2pass 미구현 및 하드코딩 기능 구현 우선순위 로드맵

본 문서는 `ec2pass` 코드베이스 전수 감사에서 식별된 **하드코딩, 더미(Stub) 구현, 테스트 통과용 임시 코드**를 해결하기 위한 우선순위 및 구현 로드맵을 정의합니다.

지원 대상 환경인 **Linux (Rocky Linux 9+, Ubuntu 22.04/24.04, Debian 12, Fedora 40+), Windows (11/Server, Hyper-V/QEMU), macOS (Apple Silicon / Intel, Hypervisor.framework/QEMU)** 전반의 안정성과 신뢰성을 확보하는 것을 목표로 합니다.

---

## 1. 우선순위 요약표 (Priority Summary)

| 번호 | 항목명 | 구분 | 영향도 | 상태 | 관련 컴포넌트 |
|:---:|---|:---:|:---:|:---:|---|
| **01** | `umount` 시 게스트 파일 영구 삭제(`rm -rf`) 제거 및 정상 언마운트 | Tier 1 (치명적) | **데이터 파괴** | **완료** | `ec2ps-daemon` / `ssh.rs` |
| **02** | `set local.<inst>.disk` 디스크 실제 리사이징 및 볼륨 확장 구현 | Tier 1 (치명적) | **기능 불능/데이터 무결성** | **완료** | `ec2ps-core::state` / QEMU |
| **03** | `clone` 시 `seed.iso` 재생성 및 `instance-id`/`hostname` 충돌 해결 | Tier 1 (치명적) | **인스턴스 충돌/초기화 실패** | **완료** | `ec2ps-daemon` / cloud-init |
| **04** | `mount` 명령을 1회성 `scp` 복사 대신 실제 공유 파일시스템(virtiofs/9p)으로 전환 | Tier 2 (주요) | **동기화 실패/성능 저하** | **완료** | `ec2ps-core::hypervisor` / QEMU |
| **05** | `stop -t <mins>` 지연 종료(Delayed Shutdown) 실제 비동기 타이머 태스크 구현 | Tier 2 (주요) | **인스턴스 누수/리소스 낭비** | **완료** | `ec2ps-daemon` |
| **06** | 스냅샷 라이브 복구(Live Restore) 시 QEMU 파일 락 충돌 해결 및 QMP 연동 | Tier 2 (주요) | **복구 실패/VM 크래시** | **완료** | `ec2ps-core::hypervisor` / QMP |
| **07** | `transfer` 인스턴스 간 직접 복사 지원 및 Windows 경로 구분자 처리 보완 | Tier 2 (주요) | **크로스플랫폼 전송 실패** | **완료** | `ec2ps-daemon` / `ssh.rs` |
| **08** | 게스트 IP 주소 보고 현실화 (가상 `10.23.104.x` 제거, QEMU/DHCP 연동) 및 더미 브릿지 정리 | Tier 3 (네트워크) | **잘못된 메타데이터/연결 오류** | **완료** | `ec2ps-core::state` / `ec2ps-daemon` |
| **09** | `launch --network` 및 `local.bridged-network` 브릿지/TAP 인터페이스 실제 바인딩 | Tier 3 (네트워크) | **네트워크 격리/브릿지 불능** | **완료** | `ec2ps-daemon` / QEMU CLI |
| **10** | `list` RPC의 인스턴스 리소스 메트릭(CPU Load, Memory) 실제 수집 로직 구현 | Tier 3 (모니터링) | **모니터링 기능 마비** | **완료** | `ec2ps-daemon` / QGA |
| **11** | 카탈로그 이미지 별칭 속임수(`ubuntu`, `minimal`) 제거 또는 실제 OS 이미지 공급 | Tier 4 (카탈로그) | **사용자 기만/기능 불일치** | **완료** | `ec2ps-core::catalog` |
| **12** | 데몬 RPC 토큰 기반 인증 체계 구현 및 미사용 더미 설정 키 정리 | Tier 4 (보안/설정) | **보안 취약점/설정 무반응** | **완료** | `ec2ps-daemon` / `ec2ps-core::state` |
| **13** | Windows Hyper-V 네이티브 드라이버 완성 (cloud-init ISO 탑재, PID, vSwitch) | Tier 4 (플랫폼) | **Windows Hyper-V 기동 실패** | **완료** | `ec2ps-core::hypervisor::windows` |
| **14** | 테스트 스위트 내 허위/취약 검증 로직 정상화 (스냅샷 복구 검증 등) | Tier 4 (테스트) | **테스트 신뢰성 결여** | **완료** | `tests/comprehensive_matrix_test.ps1` |

---

## 2. 상세 구현 계획 (Detailed Implementation Plan)

### [Tier 1] 데이터 무결성 및 시스템 안전성 (Critical Data Integrity & Safety)

---

### #01. `umount` 시 게스트 파일 영구 삭제(`rm -rf`) 제거 및 정상 언마운트
- **문제점 및 현재 코드**:
  - [`crates/ec2ps-daemon/src/main.rs#L1040-L1058`](file:///Users/jason/project/ec2pass/crates/ec2ps-daemon/src/main.rs#L1040-L1058)
  - `umount` RPC 처리 시 게스트 내부에서 `sudo rm -rf '<target>'/*`를 실행하여 마운트 포인트 내 파일들을 영구 삭제함.
  - 마운트 해제 후 디렉토리가 비어있는지 검사하는 E2E 테스트를 속이기 위해 작성된 매우 위험한 코드.
- **해결 방안**:
  1. `sudo rm -rf` 구문 즉시 삭제.
  2. 실제 마운트 해제 명령인 `sudo umount -l '<target>'` 또는 `fusermount -u '<target>'` 실행.
  3. 마운트 대상 디렉토리 자체는 마운트 전 상태로 보존하거나 빈 마운트 포인트 디렉토리만 안전하게 유지.
- **크로스플랫폼 지원**:
  - Rocky, Ubuntu, Debian, Fedora: 게스트가 Linux이므로 `umount` 호출 통일.
- **완료 검증 기준**:
  - `mount` 후 파일 생성 -> `umount` 실행 -> 호스트 원본 파일이 유지되며 게스트 마운트 포인트만 정상 해제되는지 확인.

---

### #02. `set local.<inst>.disk` 디스크 실제 리사이징 및 볼륨 확장 구현
- **문제점 및 현재 코드**:
  - [`crates/ec2ps-core/src/state.rs#L160-L170`](file:///Users/jason/project/ec2pass/crates/ec2ps-core/src/state.rs#L160-L170)
  - `ec2pass set local.<inst>.disk=<size>` 실행 시 `state.json` 내부의 `disk_gb` 값만 갱신하고 실제 qcow2/raw 디스크 이미지 리사이징을 수행하지 않음.
- **해결 방안**:
  1. 인스턴스가 중지(Stopped) 상태일 때:
     - `qemu-img resize <disk_path> <new_size>G` 실행.
  2. 인스턴스가 실행(Running) 상태일 때 (온라인 리사이징):
     - QMP(QEMU Monitor Protocol)를 통해 `block_resize` 명령 전달.
  3. 게스트 내부 파티션/파일시스템 자동 확장 지원:
     - `growpart` 및 `xfs_growfs` (Amazon Linux / Rocky) 또는 `resize2fs` (Ubuntu / Debian)를 SSH/cloud-init을 통해 트리거.
- **크로스플랫폼 지원**:
  - Windows/macOS/Linux 호스트 모두 `qemu-img` 바이너리 호출 또는 QMP를 통해 동작 가능하도록 인터페이스 통일.
- **완료 검증 기준**:
  - `set local.inst1.disk=20G` 적용 후 `qemu-img info`로 가상 크기가 20G로 변경되었는지, 게스트 내부 `df -h /`에서 증설된 용량이 인식되는지 검증.

---

### #03. `clone` 시 `seed.iso` 재생성 및 `instance-id`/`hostname` 충돌 해결
- **문제점 및 현재 코드**:
  - [`crates/ec2ps-daemon/src/main.rs#L1180-L1228`](file:///Users/jason/project/ec2pass/crates/ec2ps-daemon/src/main.rs#L1180-L1228)
  - 복제(clone) 시 원본 인스턴스의 `seed.iso`를 바이트 단위로 그대로 복사함.
  - 이로 인해 원본과 복제본의 `instance-id`와 `local-hostname`이 동일하여 cloud-init이 초기화 단계를 건너뛰고, 동일 호스트명 및 SSH 호스트 키 충돌이 발생함.
- **해결 방안**:
  1. 클론 대상의 새로운 인스턴스 이름(`target_name`)을 기반으로 `meta-data` (`instance-id: i-<new_uuid>`, `local-hostname: <target_name>`) 신규 생성.
  2. `cloud_init.rs`의 ISO 빌더 로직을 재사용하여 독립된 `seed.iso`를 생성 및 배치.
- **완료 검증 기준**:
  - `clone inst1 inst2` 실행 후 `inst2` 기동 시 호스트명이 `inst2`로 올바르게 설정되고 고유한 SSH 키가 부여되는지 확인.

---

### [Tier 2] 핵심 VM 라이프사이클 및 스토리지 공유 기능 (Core VM Lifecycle & Shared Storage)

---

### #04. `mount` 명령을 1회성 `scp` 대신 실제 공유 파일시스템으로 전환
- **문제점 및 현재 코드**:
  - [`crates/ec2ps-core/src/ssh.rs#L314-L378`](file:///Users/jason/project/ec2pass/crates/ec2ps-core/src/ssh.rs#L314-L378)
  - `mount` 명령이 호스트와 게스트 간 실제 마운트가 아닌 1회성 `scp -r` 디렉토리 복사로 처리됨.
  - 호스트 파일 변경이 게스트에 실시간 반영되지 않으며, 대용량 디렉토리 마운트 시 심각한 지연 발생. `--uid-map`, `--gid-map`, `--type` 옵션 전부 무시됨.
- **해결 방안**:
  1. **Linux 호스트**:
     - QEMU virtiofs (`virtiofsd`) 또는 9p (`-virtfs local,path=...,mount_tag=...`) 지원.
     - Linux 커널의 9p/virtiofs 모듈을 활용하여 게스트 내부에서 `mount -t 9p ...` 자동 수행.
  2. **macOS 호스트**:
     - QEMU 9p (`virtfs-proxy-helper` 또는 내장 9p) 활용.
  3. **Windows 호스트**:
     - QEMU SMB (`-net nic -net user,smb=...`) 또는 게스트 내부 SSHFS/CIFS 연동.
- **완료 검증 기준**:
  - 호스트 마운트 디렉토리에 파일 생성 시 게스트에서 즉시 확인 가능하고, 양방향 수정이 동기화되는지 확인.

---

### #05. `stop -t <mins>` 지연 종료(Delayed Shutdown) 실제 백그라운드 타이머 구현
- **문제점 및 현재 코드**:
  - [`crates/ec2ps-daemon/src/main.rs#L708-L720`](file:///Users/jason/project/ec2pass/crates/ec2ps-daemon/src/main.rs#L708-L720)
  - `stop -t <minutes>` 호출 시 `rec.details.state = InstanceState::DelayedShutdown`으로 상태 값만 변경할 뿐, 타이머 태스크를 실행하지 않아 VM이 꺼지지 않음. `stop -c`도 Enum만 `Running`으로 되돌림.
- **해결 방안**:
  1. `tokio::spawn`을 이용한 백그라운드 타이머 태스크 등록 (`HashMap<String, tokio::sync::oneshot::Sender<()>>`로 취소 채널 관리).
  2. 지정된 시간이 경과하면 데몬 내부의 정상 `stop_instance()` 로직 자동 트리거.
  3. `stop -c` 호출 시 해당 채널에 취소 신호를 보내어 타이머 태스크를 중단하고 상태를 `Running`으로 복구.
- **완료 검증 기준**:
  - `stop inst1 -t 1` 실행 후 1분 뒤 인스턴스가 `Stopped` 상태로 전이되는지, 도중에 `stop inst1 -c` 실행 시 종료가 방지되는지 검증.

---

### #06. 스냅샷 라이브 복구(Live Restore) 시 QEMU 파일 락 충돌 해결 및 QMP 연동
- **문제점 및 현재 코드**:
  - [`crates/ec2ps-core/src/hypervisor/qemu_common.rs`](file:///Users/jason/project/ec2pass/crates/ec2ps-core/src/hypervisor/qemu_common.rs)
  - 실행 중인 VM에 대해 `qemu-img snapshot -a` 명령을 호출하면 QEMU 프로세스가 qcow2 이미지를 독점 락(exclusive lock)하고 있어 실패함.
- **해결 방안**:
  1. VM이 실행 중(Running)인 경우:
     - QMP 소켓(`qmp.sock`)을 통해 `loadvm <tag>` 또는 `system_reset` 명령을 전달하여 라이브 복원 수행.
  2. VM이 중지(Stopped) 상태인 경우:
     - 기존대로 `qemu-img snapshot -a <tag> <disk>` 실행.
  3. 필요한 경우 복원 전 안전하게 VM을 일시 정지(`stop`)하거나 종료 후 롤백 후 재기동하는 Fallback 전략 구현.
- **완료 검증 기준**:
  - VM 기동 상태에서 canary 파일 생성 -> 스냅샷 생성 -> canary 파일 삭제 -> `snapshot-restore` 실행 -> canary 파일이 복구되는지 확인.

---

### #07. `transfer` 인스턴스 간 직접 복사 지원 및 Windows 경로 구분자 처리 보완
- **문제점 및 현재 코드**:
  - [`crates/ec2ps-daemon/src/main.rs#L990-L1025`](file:///Users/jason/project/ec2pass/crates/ec2ps-daemon/src/main.rs#L990-L1025)
  - 호스트<->게스트 간 전송만 scp로 지원하며, `inst1:path` -> `inst2:path`와 같은 인스턴스 간 직접 전송 미지원.
  - Windows 호스트 환경에서 `C:\path`와 같은 드라이브 문자 콜론(`:`)이 인스턴스 식별 콜론과 오인식되는 문제 존재.
- **해결 방안**:
  1. 경로 파싱 로직에서 정규식 및 Windows 드라이브 패턴(`^[a-zA-Z]:[\\/]`)을 우선 제외하도록 경로 파서 수정.
  2. `inst1:path` -> `inst2:path` 감지 시, 데몬이 임시 버퍼 파이프를 열거나 호스트 임시 디렉토리를 경유하여 2단계 scp 전송 파이프라인 수행.
- **완료 검증 기준**:
  - `transfer inst1:/var/log/messages inst2:/tmp/` 명령 정상 작동 확인 및 Windows 환경의 로컬 경로 전송 테스트 통과.

---

### [Tier 3] 네트워크 정확도 및 인프라 모니터링 (Accurate Networking & Metrics)

---

### #08. 게스트 IP 주소 보고 현실화 및 하드코딩된 더미 브릿지 정리
- **문제점 및 현재 코드**:
  - [`crates/ec2ps-core/src/state.rs#L98-L101`](file:///Users/jason/project/ec2pass/crates/ec2ps-core/src/state.rs#L98-L101) (`10.23.104.x` 가상 IP 생성)
  - [`crates/ec2ps-daemon/src/main.rs#L1461-L1466`](file:///Users/jason/project/ec2pass/crates/ec2ps-daemon/src/main.rs#L1461-L1466) (존재하지 않는 `ec2psbr0` 하드코딩)
  - QEMU 사용자 네트워크(SLIRP)는 실제 게스트에게 `10.0.2.15`를 할당하지만 데몬은 임의의 가상 IP를 반환하여 사용자가 혼란을 겪음.
- **해결 방안**:
  1. 기본 SLIRP 모드에서는 실제 게스트 IP인 `10.0.2.15` (또는 포트 포워딩 주소인 `127.0.0.1:<forwarded_port>`)를 명확히 표기.
  2. QEMU Guest Agent(QGA)가 연결된 경우 `guest-network-get-interfaces` QMP 호출을 통해 실제 인터페이스 IP를 조회.
  3. `detect_host_networks()`에서 하드코딩된 `ec2psbr0`를 제거하고 OS별 실제 인터페이스(`ip -j link` / `ifconfig` / `Get-NetAdapter`)를 동적으로 조회.
- **완료 검증 기준**:
  - `ec2pass info <inst>` 및 `ec2pass networks` 출력 시 실제 인터페이스 및 IP와 일치하는 정보 반환.

---

### #09. `launch --network` 및 `local.bridged-network` 브릿지/TAP 인터페이스 실제 바인딩
- **문제점 및 현재 코드**:
  - [`crates/ec2ps-daemon/src/main.rs#L552`](file:///Users/jason/project/ec2pass/crates/ec2ps-daemon/src/main.rs#L552) (`networks: _`로 파라미터 폐기)
  - [`crates/ec2ps-core/src/state.rs#L40-L216`](file:///Users/jason/project/ec2pass/crates/ec2ps-core/src/state.rs#L40-L216) (`local.bridged-network` 미사용)
- **해결 방안**:
  1. `--network <name>` 지정 시 QEMU 인수에 `-netdev tap,id=net0,br=<bridge_name>,helper=/usr/libexec/qemu-bridge-helper` 추가.
  2. 설정된 브릿지가 없거나 권한이 부족할 경우 명확한 에러 메시지 반환.
  3. macOS/Windows 환경에서는 각 플랫폼 지원 네트워크 백엔드(Hyper-V vSwitch, macOS vmnet)에 매핑.
- **완료 검증 기준**:
  - 브릿지 네트워크 설정 후 외부 동일 서브넷의 다른 머신에서 인스턴스 IP로 직접 ping 및 ssh 연결 성공 확인.

---

### #10. `list` RPC의 인스턴스 리소스 메트릭(CPU Load, Memory) 실제 수집 로직 구현
- **문제점 및 현재 코드**:
  - [`crates/ec2ps-daemon/src/main.rs#L353-L354`](file:///Users/jason/project/ec2pass/crates/ec2ps-daemon/src/main.rs#L353-L354)
  - `load: Some(0.0)`, `memory_usage: Some(0)`으로 하드코딩되어 `ec2pass list` 화면에 항상 0으로 표시됨.
- **해결 방안**:
  1. QEMU 프로세스 PID를 기반으로 호스트 OS 리소스 조회:
     - Linux: `/proc/<pid>/stat`, `/proc/<pid>/status` 파싱.
     - macOS: `libproc` 또는 `proc_pidinfo` API 활용.
     - Windows: Windows Process API (`GetProcessMemoryInfo`) 활용.
  2. 게스트 내부 정밀 메트릭 필요 시:
     - QGA (QEMU Guest Agent)의 메모리/로드 정보 조회 또는 백그라운드 SSH 상태 수집 캐시 활용.
- **완료 검증 기준**:
  - VM 부하 발생 시 `ec2pass list`에서 CPU 점유율 및 메모리 사용량 수치가 실시간으로 갱신되는지 확인.

---

### [Tier 4] 카탈로그, 인증 및 플랫폼 드라이버 완성도 (Catalog, Auth, Drivers & Tests)

---

### #11. 카탈로그 이미지 별칭 속임수(`ubuntu`, `minimal`) 제거 또는 실제 OS 이미지 공급
- **문제점 및 현재 코드**:
  - [`crates/ec2ps-core/src/catalog.rs#L478-L491`](file:///Users/jason/project/ec2pass/crates/ec2ps-core/src/catalog.rs#L478-L491)
  - `ubuntu`, `minimal`, `al2023-k6.12` 요청 시 실제 우분투 이미지를 내려받지 않고 모두 기본 Amazon Linux 2023 이미지로 라우팅함.
- **해결 방안**:
  1. 공식 클라우드 이미지 URL 매핑 테이블 확장:
     - `ubuntu`: Ubuntu 24.04 LTS 공식 클라우드 이미지 (`noble-server-cloudimg-amd64.img`)
     - `debian`: Debian 12 공식 클라우드 이미지 (`debian-12-generic-amd64.qcow2`)
     - `rocky`: Rocky Linux 9 공식 클라우드 이미지
     - `al2023`: 기존 Amazon Linux 2023 클라우드 이미지
  2. 미지원 별칭 요청 시 자동 대체 대신 지원 목록을 안내하며 에러 반환.
- **완료 검증 기준**:
  - `ec2pass launch ubuntu --name u1` 실행 시 실제 Ubuntu OS가 설치되어 `/etc/os-release`에서 Ubuntu가 확인되는지 검증.

---

### #12. 데몬 RPC 토큰 기반 인증 체계 구현 및 미사용 더미 설정 키 정리
- **문제점 및 현재 코드**:
  - [`crates/ec2ps-daemon/src/main.rs#L296-L306`](file:///Users/jason/project/ec2pass/crates/ec2ps-daemon/src/main.rs#L296-L306)
  - `authenticate ''` 요청 시 항상 `true`를 반환하며, 이후 어떤 RPC 호출에서도 인증 상태를 확인하지 않음.
  - `local.privileged-mounts`, `client.apps.windows-terminal.profiles` 등 BTreeMap에만 보관되고 코드에서 전혀 쓰이지 않는 유령 설정 키 방치.
- **해결 방안**:
  1. 로컬 유닉스 도메인 소켓 / Windows 명명된 파이프 보안:
     - 소켓 파일 퍼미션을 소유자 전용(`0600`)으로 제한하여 호스트 OS 계정 기반 1차 격리.
  2. 세션 토큰 메커니즘 구현:
     - `~/.ec2pass/auth_token` 생성 및 RPC 핸드셰이크 시 검증.
  3. 미사용 설정 키 정리:
     - 실제로 기능 구현이 예정된 키는 스키마 문서화, 불필요한 키는 파서에서 경고 후 제거.
- **완료 검증 기준**:
  - 토큰 불일치 시 RPC 요청 거절 및 유효하지 않은 설정 키 입력 시 검증 오류 발생 확인.

---

### #13. Windows Hyper-V 네이티브 드라이버 완성 (cloud-init ISO 탑재, PID, vSwitch)
- **문제점 및 현재 코드**:
  - [`crates/ec2ps-core/src/hypervisor/windows.rs`](file:///Users/jason/project/ec2pass/crates/ec2ps-core/src/hypervisor/windows.rs)
  - `# Add-VMDvdDrive` 코드가 주석 처리되어 `seed.iso`가 마운트되지 않음.
  - 프로세스 PID를 하드코딩(`1000`)으로 반환, Hyper-V 에러 무시(`-ErrorAction SilentlyContinue`), 네트워크 인터페이스 미구성.
- **해결 방안**:
  1. PowerShell/WMI 명령을 통해 SCSI 컨트롤러에 `seed.iso` DVD 드라이브 정상 연결.
  2. Hyper-V 가상 머신 Worker 프로세스(`vmwp.exe`)의 실제 PID 추적 로직 구현.
  3. 기본 Default Switch에 가상 네트워크 어댑터 연결.
- **완료 검증 기준**:
  - Windows 11 / Server 호스트에서 Hyper-V 백엔드로 AL2023 기동 및 SSH 로그인 성공 확인.

---

### #14. 테스트 스위트 내 허위/취약 검증 로직 정상화 (스냅샷 복구 검증 등)
- **문제점 및 현재 코드**:
  - [`tests/comprehensive_matrix_test.ps1#L155`](file:///Users/jason/project/ec2pass/tests/comprehensive_matrix_test.ps1#L155)
  - 스냅샷 생성 후 canary 파일을 삭제하지 않고 복원 명령을 호출한 뒤 canary 파일이 존재함을 검사하여, 복원 기능이 실패했음에도 테스트가 통과되도록 작성됨.
- **해결 방안**:
  1. Canary 파일 생성 -> `snapshot-create` -> Canary 파일 명시적 삭제 (`rm /tmp/canary.txt`) -> 삭제 확인 -> `snapshot-restore` -> Canary 파일 부활 검증.
  2. `e2e_test.sh` 및 `comprehensive_matrix_test.ps1`의 모든 검증 단계를 실질적 상태 변화 검증으로 강화.
- **완료 검증 기준**:
  - 실제 복원 기능이 실패했을 때 테스트가 반드시 실패(Fail)하고, 정상 구현 시에만 통과(Pass)함을 확인.

---

## 3. 추천 구현 로드맵 (Execution Milestones)

```
[Phase 1: 데이터 안전성 확보 (Sprint 1)]
  ├── #01. umount 시 rm -rf 삭제 및 정상 언마운트 (최우선) [완료]
  ├── #02. set local.inst.disk 실제 리사이징 및 볼륨 확장 [완료]
  └── #03. clone 시 seed.iso 재생성 및 충돌 해결 [완료]

[Phase 2: 핵심 스토리지 및 VM 라이프사이클 (Sprint 2)]
  ├── #04. mount 9p/virtiofs 실시간 공유 파일시스템 구현 [완료]
  ├── #05. stop -t 지연 종료 비동기 타이머 구현 [완료]
  └── #06. 스냅샷 라이브 복구 QMP 연동 및 #14 테스트 정상화 [완료]

[Phase 3: 네트워크 및 모니터링 현실화 (Sprint 3)]
  ├── #08. 게스트 IP 주소 실측값 연동 및 더미 브릿지 제거 [완료]
  ├── #10. list 메트릭(CPU/Memory) 실제 수집 로직 연동 [완료]
  └── #07. transfer 인스턴스 간 전송 및 Windows 경로 보완 [완료]

[Phase 4: 카탈로그, 인증 및 플랫폼 확장 (Sprint 4)]
  ├── #11. 실제 다중 배포판(Ubuntu, Debian 등) 카탈로그 이미지 연동 [완료]
  ├── #09. launch --network 실제 브릿지/TAP 인터페이스 구현 [완료]
  ├── #12. 데몬 RPC 인증 및 설정 정리 [완료]
  └── #13. Windows Hyper-V 드라이버 완성 [완료]

[Phase 5: 심층 코드 감사 및 정밀 수정 (Sprint 5)]
  ├── #01. 크로스플랫폼 프로세스 생존 검증 (POSIX libc::kill / Windows tasklist) [완료]
  ├── #02. macOS NIC 동적 감지 (ifconfig -l) [완료]
  ├── #03. CLI client.primary-name 데몬 설정 동적 동기화 [완료]
  ├── #04. restore --destructive 복구 시 후속 스냅샷 히스토리 정리 [완료]
  ├── #05. launch --timeout 인위적 180초 제한 제거 [완료]
  ├── #06. 다중 배포판 sudoers/wheel 그룹 자동 구성 [완료]
  ├── #07. ARM64 UEFI(edk2) 펌웨어 자동 검색 및 연동 [완료]
  └── #08. 카탈로그 정식 별칭 등록, find 레거시 필터링 및 exec 워킹 디렉토리 매핑 [완료]
```
