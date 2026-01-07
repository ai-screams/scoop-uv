# scoop doctor 기능 설계 문서

> **Status**: 설계 완료, 구현 대기
> **Version**: v0.3.0 예정
> **Last Updated**: 2025-01

---

## 개요

`scoop doctor`는 scoop 설치 상태를 자가진단하고 문제 해결 방법을 안내하는 명령어다.

### 목적

1. **신규 사용자 온보딩 마찰 감소**: 설치 후 즉시 검증 가능
2. **지원 요청 전 자가진단**: GitHub Issue 생성 전 스스로 해결
3. **설정 오류 조기 발견**: 쉘 훅, 디렉토리 권한 등
4. **해결책 안내**: 복붙 가능한 Fix Hint 제공

---

## 경쟁 도구 분석

### 비교표

| 도구 | 명령어 | 주요 목적 | 체크 항목 |
|------|--------|----------|----------|
| pyenv-doctor | `pyenv doctor` | 빌드 환경 검증 | git, OpenSSL, SQLite3, 컴파일러 |
| brew doctor | `brew doctor` | 설치 상태 진단 | Xcode CLT, orphan kegs, unbrewed 파일 |
| npm doctor | `npm doctor` | 런타임 환경 검증 | Node.js, git, 레지스트리 연결, 권한, 캐시 |
| flutter doctor | `flutter doctor` | 개발환경 완전성 | SDK, Android/iOS 툴체인, IDE 플러그인 |
| rustup check | `rustup check` | 업데이트 확인 | 툴체인 버전, rustup 버전 |
| poetry | `poetry debug info` | 디버그 정보 | (doctor 없음) |
| uv | 없음 | - | (doctor 없음) |

### 참고 링크

- [pyenv-doctor GitHub](https://github.com/pyenv/pyenv-doctor)
- [Homebrew Troubleshooting](https://docs.brew.sh/Troubleshooting)
- [npm doctor documentation](https://docs.npmjs.com/cli/v7/commands/npm-doctor/)
- [Flutter Doctor Guide](https://docs.flutter.dev/install/troubleshoot)
- [rustup basics](https://rust-lang.github.io/rustup/basics.html)

### 핵심 패턴

**pyenv-doctor**: 빌드 의존성 검증 (OpenSSL, SQLite3 헤더)
- scoop은 uv 사용하므로 빌드 불필요 → 해당 없음

**npm doctor**: 런타임 환경 + 네트워크 + 권한 검증
- 가장 유사한 모델
- 선택적 체크 인자 지원 (connection, versions, cache, permissions)

**flutter doctor**: 개발 환경 완전성 + 컬러 출력 + 상세 안내
- UX 참고 모델
- `-v` 상세 모드, 설치 안내 링크 제공

---

## 체크 항목 설계

### 카테고리 1: 핵심 의존성 (Critical)

```
[✓] uv: installed (0.5.14)
    Path: /Users/x/.cargo/bin/uv
```

**체크 로직**:
```rust
// which uv 또는 uv --version 실행
Command::new("uv").arg("--version").output()
```

**실패 시**:
```
[✗] uv: NOT FOUND
    Fix: curl -LsSf https://astral.sh/uv/install.sh | sh
```

---

### 카테고리 2: 쉘 통합 (Shell Integration)

```
[✓] Shell: zsh
[✓] Shell hook: installed
    Source: eval "$(scoop init zsh)" in ~/.zshrc
```

**체크 로직**:
1. 현재 쉘 감지: `$SHELL` 환경변수
2. 쉘 설정 파일에서 `scoop init` 패턴 검색:
   - bash: `~/.bashrc`, `~/.bash_profile`
   - zsh: `~/.zshrc`, `~/.zprofile`
3. `eval` 포함 여부 확인 (흔한 실수)

**실패 시**:
```
[✗] Shell hook: NOT installed
    Fix: Add to ~/.zshrc:
      eval "$(scoop init zsh)"
```

**경고 (eval 누락)**:
```
[!] Shell hook: found but missing 'eval'
    Current: scoop init zsh
    Fix: eval "$(scoop init zsh)"
```

---

### 카테고리 3: 디렉토리 구조 (Directory Structure)

```
[✓] SCOOP_HOME: ~/.scoop
[✓] virtualenvs/: exists, writable
[✓] version (global): exists → myenv
```

**체크 로직**:
```rust
// SCOOP_HOME 환경변수 또는 기본값
let scoop_home = env::var("SCOOP_HOME")
    .unwrap_or_else(|_| dirs::home_dir().join(".scoop"));

// 디렉토리 존재 및 쓰기 권한
fs::metadata(&scoop_home)?.permissions().readonly() == false
```

---

### 카테고리 4: 현재 환경 상태 (Current Environment)

```
[✓] Local env: myproject (from .scoop-version)
[✓] Environment 'myproject' exists
    Path: ~/.scoop/virtualenvs/myproject
    Python: 3.12.1
[✓] Active: myproject (SCOOP_ACTIVE matches)
```

**문제 상황**:
```
[!] Local env: old-project (from .scoop-version)
[✗] Environment 'old-project' NOT FOUND
    Fix: scoop create old-project 3.12
         or: rm .scoop-version
```

**체크 로직**:
1. `scoop resolve` 실행하여 현재 환경 이름 획득
2. 해당 환경 디렉토리 존재 확인
3. `SCOOP_ACTIVE` 환경변수와 일치 여부

---

### 카테고리 5: Python 버전 (Python Versions)

```
[✓] Installed Pythons: 3
    - 3.10.14
    - 3.11.9
    - 3.12.4
```

**체크 로직**:
```rust
// uv python list --installed 실행
Command::new("uv")
    .args(["python", "list", "--installed"])
    .output()
```

---

### 카테고리 6: 환경 무결성 (Environment Integrity)

```
[✓] Environments: 5 total
    - backend-api (3.12.1)
    - frontend (3.11.9)
    ...
[!] Orphaned: 1
    - old-test (Python 3.9 not installed)
```

**체크 로직**:
- 모든 환경 순회
- 각 환경의 Python 버전이 설치되어 있는지 확인
- 고아 환경 (Python 버전 누락) 경고

---

## 출력 형식

### 상태 아이콘

| 아이콘 | 의미 | 색상 |
|--------|------|------|
| `[✓]` | 성공 | 녹색 |
| `[✗]` | 실패 (필수) | 빨간색 |
| `[!]` | 경고 (권장) | 노란색 |
| `[?]` | 정보 (참고) | 파란색 |

### 전체 출력 예시 (성공)

```
$ scoop doctor

Checking scoop installation...

Dependencies
  [✓] uv 0.5.14 (/Users/x/.cargo/bin/uv)

Shell Integration
  [✓] Shell: zsh
  [✓] Hook installed in ~/.zshrc

Directories
  [✓] SCOOP_HOME: /Users/x/.scoop
  [✓] virtualenvs/: writable

Current Environment
  [✓] Local: myproject (from .scoop-version)
  [✓] Active: myproject

Python Versions
  [✓] 3 versions installed (3.10, 3.11, 3.12)

Environments
  [✓] 5 environments, all healthy

✓ Your scoop installation is healthy!
```

### 전체 출력 예시 (문제 있음)

```
$ scoop doctor

Checking scoop installation...

Dependencies
  [✓] uv 0.5.14 (/Users/x/.cargo/bin/uv)

Shell Integration
  [✗] Hook NOT installed
      Add to ~/.zshrc:
        eval "$(scoop init zsh)"

Current Environment
  [!] Local: deleted-env (from .scoop-version)
      Environment does not exist!
      Fix: scoop create deleted-env 3.12

Found 2 issue(s). See above for fixes.
```

---

## CLI 인터페이스

### 명령 구조

```bash
scoop doctor [OPTIONS]

OPTIONS:
    -v, --verbose     상세 출력 (경로, 버전 상세)
    -q, --quiet       요약만 출력 (CI용)
    --json            JSON 형식 출력
    --fix             자동 수정 가능한 문제 해결 시도
```

### 향후 확장 (Phase 3)

```bash
scoop doctor --check shell      # 쉘 통합만 검사
scoop doctor --check envs       # 환경 무결성만 검사
scoop doctor --check deps       # 의존성만 검사
```

### Exit Code

| 코드 | 의미 |
|------|------|
| 0 | 모든 검사 통과 |
| 1 | 경고 있음 (동작에 지장 없음) |
| 2 | 오류 있음 (동작에 지장 있음) |

### --fix 동작 범위

**자동 수정 가능**:
- `~/.scoop` 디렉토리 생성
- `virtualenvs/` 디렉토리 생성

**자동 수정 불가 (안내만)**:
- 쉘 설정 파일 수정 (사용자 승인 필요)
- uv 설치
- 환경 생성/삭제

---

## 구현 구조

### 파일 구조

```
src/cli/commands/
├── doctor.rs         # 메인 doctor 명령
└── doctor/
    ├── mod.rs        # 체크 모듈 통합
    ├── checks/
    │   ├── mod.rs
    │   ├── deps.rs       # 의존성 체크 (uv)
    │   ├── shell.rs      # 쉘 통합 체크
    │   ├── dirs.rs       # 디렉토리 구조 체크
    │   └── envs.rs       # 환경 무결성 체크
    └── report.rs     # 결과 포맷팅/출력
```

### 핵심 타입

```rust
/// 체크 결과 상태
pub enum CheckStatus {
    Pass,           // ✓
    Fail,           // ✗
    Warning,        // !
    Info,           // ?
}

/// 개별 체크 결과
pub struct CheckResult {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    pub fix_hint: Option<String>,
    pub details: Option<Vec<String>>,
}

/// 체크 카테고리 결과
pub struct CategoryResult {
    pub name: String,
    pub checks: Vec<CheckResult>,
}

/// 전체 진단 보고서
pub struct DoctorReport {
    pub categories: Vec<CategoryResult>,
    pub has_errors: bool,
    pub has_warnings: bool,
}
```

### 체크 트레이트

```rust
/// 건강 체크 인터페이스
pub trait HealthCheck {
    fn name(&self) -> &str;
    fn run(&self) -> CheckResult;
}

// 구현 예시
pub struct UvCheck;

impl HealthCheck for UvCheck {
    fn name(&self) -> &str {
        "uv"
    }

    fn run(&self) -> CheckResult {
        match Command::new("uv").arg("--version").output() {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                CheckResult::pass(format!("uv {}", version.trim()))
            }
            _ => CheckResult::fail(
                "uv not found",
                Some("Install: curl -LsSf https://astral.sh/uv/install.sh | sh"),
            ),
        }
    }
}
```

---

## 구현 로드맵

### Phase 1: MVP (v0.3.0) ⭐ 권장 시작점

**체크 항목** (4개):
1. uv 존재 확인 (`which uv` + `--version`)
2. SCOOP_HOME 디렉토리 확인
3. 쉘 훅 설치 여부 (설정 파일 검색)
4. 현재 환경 해석 가능 여부 (`scoop resolve`)

**예상 코드량**: ~300 LOC
**구현 난이도**: 🟢 낮음
**커버리지**: 지원 요청 80% 감소 예상

### Phase 2: 완성 (v0.4.0)

**추가 체크 항목**:
5. 쉘 설정 파일 상세 분석 (`eval` 누락 감지)
6. 설치된 Python 버전 목록
7. 환경 무결성 검사 (고아 환경)

**추가 기능**:
8. `--verbose` 옵션
9. `--quiet` 옵션
10. `--json` 옵션
11. 컬러 출력 및 아이콘

**예상 코드량**: ~500 LOC 추가
**구현 난이도**: 🟡 중간

### Phase 3: 고급 (v0.5.0+)

**추가 기능**:
12. `--fix` 자동 수정
13. `--check <category>` 선택적 체크
14. CI 모드 (`--ci`)
15. Fish/PowerShell 쉘 감지

**구현 난이도**: 🟠 높음

---

## 가장 흔한 문제 (우선 해결 대상)

1. **uv 미설치**
   - 증상: 환경 생성 실패
   - 해결: 설치 명령 안내

2. **쉘 훅 미설정**
   - 증상: 자동 활성화 안됨
   - 해결: 복붙 가능한 설정 라인 제공

3. **삭제된 환경 참조**
   - 증상: cd 시 에러 발생
   - 해결: `scoop create` 또는 `.scoop-version` 삭제 안내

---

## 테스트 시나리오

### 정상 케이스
- [ ] uv 설치됨, 쉘 훅 설정됨, 환경 정상 → 모두 통과

### 실패 케이스
- [ ] uv 미설치 → 설치 안내
- [ ] SCOOP_HOME 없음 → 생성 안내
- [ ] 쉘 훅 미설정 → 설정 안내
- [ ] eval 누락 → 수정 안내
- [ ] .scoop-version이 존재하지 않는 환경 참조 → 생성/삭제 안내
- [ ] 고아 환경 존재 → 경고

### 옵션 테스트
- [ ] `--verbose`: 상세 경로/버전 출력
- [ ] `--quiet`: 요약만 출력
- [ ] `--json`: JSON 형식 출력
- [ ] `--fix`: 디렉토리 자동 생성

---

## 참고 자료

- [pyenv-doctor 소스코드](https://github.com/pyenv/pyenv-doctor/blob/master/bin/pyenv-doctor)
- [npm doctor 구현](https://github.com/npm/cli/tree/latest/lib/commands/doctor.js)
- [flutter doctor 구현](https://github.com/flutter/flutter/tree/master/packages/flutter_tools/lib/src/doctor.dart)
