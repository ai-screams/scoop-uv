//! Common test utilities
//!
//! 테스트용 픽스처와 격리된 환경을 제공한다.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test fixture for scoop tests
///
/// 기존 테스트 호환성을 위해 유지. 단순한 SCOOP_HOME 설정만 필요할 때 사용.
pub struct TestFixture {
    /// Temporary directory
    pub temp_dir: TempDir,
    /// SCOOP_HOME path
    pub scoop_home: PathBuf,
}

impl TestFixture {
    /// Create a new test fixture
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let scoop_home = temp_dir.path().join(".scoop");

        // Set SCOOP_HOME for tests
        // SAFETY: 테스트 환경에서 단일 스레드로 실행됨
        unsafe {
            std::env::set_var("SCOOP_HOME", &scoop_home);
        }

        Self {
            temp_dir,
            scoop_home,
        }
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        // Clean up environment
        // SAFETY: 테스트 환경에서 단일 스레드로 실행됨
        unsafe {
            std::env::remove_var("SCOOP_HOME");
        }
    }
}

impl Default for TestFixture {
    fn default() -> Self {
        Self::new()
    }
}

/// 격리된 테스트 환경
///
/// pyenv, virtualenvwrapper, conda 등 다양한 Python 환경 관리 도구를
/// 시뮬레이션하기 위한 완전 격리된 테스트 환경을 제공한다.
///
/// # 특징
///
/// - 모든 관련 환경 변수를 백업하고 테스트용 값으로 설정
/// - Drop 시 원래 환경 변수로 복원
/// - mock pyenv, virtualenvwrapper, conda 디렉토리 구조 생성 지원
///
/// # Example
///
/// ```ignore
/// let env = IsolatedTestEnv::new();
/// env.setup_mock_pyenv(&["3.11.0", "3.12.0"]);
/// env.setup_mock_pyenv_virtualenv("myenv", "3.11.0");
/// // 테스트 수행...
/// // Drop 시 자동으로 환경 복원
/// ```
pub struct IsolatedTestEnv {
    /// 임시 홈 디렉토리 (테스트 종료 시 자동 삭제)
    pub home: TempDir,
    /// SCOOP_HOME 경로
    pub scoop_home: PathBuf,
    /// PYENV_ROOT 경로
    pub pyenv_root: PathBuf,
    /// WORKON_HOME 경로 (virtualenvwrapper)
    pub workon_home: PathBuf,
    /// CONDA_PREFIX 경로
    pub conda_prefix: PathBuf,
    /// 원래 환경 변수 백업 (None = 원래 설정 안 됨)
    original_env: HashMap<String, Option<String>>,
}

/// 백업할 환경 변수 목록
const ENV_VARS_TO_BACKUP: &[&str] = &[
    "HOME",
    "SCOOP_HOME",
    "PYENV_ROOT",
    "WORKON_HOME",
    "CONDA_PREFIX",
    "VIRTUAL_ENV",
    "PATH",
];

impl IsolatedTestEnv {
    /// 새로운 격리 테스트 환경을 생성한다.
    ///
    /// 1. 현재 환경 변수를 백업
    /// 2. 임시 디렉토리 생성
    /// 3. 테스트용 환경 변수 설정
    ///
    /// # Panics
    ///
    /// 임시 디렉토리 생성 실패 시 패닉.
    pub fn new() -> Self {
        // 1. 환경 변수 백업
        let mut original_env = HashMap::new();
        for var in ENV_VARS_TO_BACKUP {
            original_env.insert((*var).to_string(), std::env::var(*var).ok());
        }

        // 2. 임시 디렉토리 생성
        let home = TempDir::new().expect("Failed to create temp home directory");
        let home_path = home.path();

        let scoop_home = home_path.join(".scoop");
        let pyenv_root = home_path.join(".pyenv");
        let workon_home = home_path.join(".virtualenvs");
        let conda_prefix = home_path.join(".conda");

        // 3. 디렉토리 생성
        fs::create_dir_all(&scoop_home).expect("Failed to create scoop_home");
        fs::create_dir_all(&pyenv_root).expect("Failed to create pyenv_root");
        fs::create_dir_all(&workon_home).expect("Failed to create workon_home");
        fs::create_dir_all(&conda_prefix).expect("Failed to create conda_prefix");

        // 4. 환경 변수 설정
        // SAFETY: 테스트 환경에서 단일 스레드로 실행되며, Drop에서 복원됨
        unsafe {
            std::env::set_var("HOME", home_path);
            std::env::set_var("SCOOP_HOME", &scoop_home);
            std::env::set_var("PYENV_ROOT", &pyenv_root);
            std::env::set_var("WORKON_HOME", &workon_home);
            std::env::set_var("CONDA_PREFIX", &conda_prefix);
            std::env::remove_var("VIRTUAL_ENV"); // 활성화된 venv 없는 상태로 시작
        }

        Self {
            home,
            scoop_home,
            pyenv_root,
            workon_home,
            conda_prefix,
            original_env,
        }
    }

    /// mock pyenv Python 버전들을 설치한다.
    ///
    /// `~/.pyenv/versions/{version}/bin/python` 구조를 생성.
    ///
    /// # Arguments
    ///
    /// * `versions` - 설치할 Python 버전 목록 (예: `["3.11.0", "3.12.0"]`)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let env = IsolatedTestEnv::new();
    /// env.setup_mock_pyenv(&["3.11.0", "3.12.0"]);
    /// // ~/.pyenv/versions/3.11.0/bin/python 생성됨
    /// ```
    pub fn setup_mock_pyenv(&self, versions: &[&str]) -> &Self {
        let versions_dir = self.pyenv_root.join("versions");
        fs::create_dir_all(&versions_dir).expect("Failed to create pyenv versions dir");

        for version in versions {
            let bin_dir = versions_dir.join(version).join("bin");
            fs::create_dir_all(&bin_dir).expect("Failed to create pyenv bin dir");

            // mock python executable (빈 파일)
            let python_path = bin_dir.join("python");
            fs::write(&python_path, "#!/bin/sh\n# mock python").expect("Failed to create mock python");

            // python3 심볼릭 링크 대신 복사본 생성 (크로스 플랫폼 호환)
            let python3_path = bin_dir.join("python3");
            fs::write(&python3_path, "#!/bin/sh\n# mock python3").expect("Failed to create mock python3");
        }

        self
    }

    /// mock pyenv-virtualenv 가상환경을 생성한다.
    ///
    /// pyenv-virtualenv는 `~/.pyenv/versions/{name}` 구조를 사용하며,
    /// pyvenv.cfg 파일로 가상환경임을 표시한다.
    ///
    /// # Arguments
    ///
    /// * `name` - 가상환경 이름
    /// * `python_version` - 기반 Python 버전 (예: "3.11.0")
    ///
    /// # Returns
    ///
    /// 생성된 가상환경 경로
    ///
    /// # Example
    ///
    /// ```ignore
    /// let env = IsolatedTestEnv::new();
    /// env.setup_mock_pyenv(&["3.11.0"]);
    /// let venv_path = env.setup_mock_pyenv_virtualenv("myenv", "3.11.0");
    /// ```
    pub fn setup_mock_pyenv_virtualenv(&self, name: &str, python_version: &str) -> PathBuf {
        let venv_dir = self.pyenv_root.join("versions").join(name);
        let bin_dir = venv_dir.join("bin");
        fs::create_dir_all(&bin_dir).expect("Failed to create virtualenv bin dir");

        // pyvenv.cfg 생성 (가상환경 표시자)
        let pyvenv_cfg = venv_dir.join("pyvenv.cfg");
        let base_path = self.pyenv_root.join("versions").join(python_version);
        let cfg_content = format!(
            "home = {}/bin\n\
             include-system-site-packages = false\n\
             version = {}\n",
            base_path.display(),
            python_version
        );
        fs::write(&pyvenv_cfg, cfg_content).expect("Failed to create pyvenv.cfg");

        // mock python executable
        let python_path = bin_dir.join("python");
        fs::write(&python_path, "#!/bin/sh\n# mock venv python").expect("Failed to create mock python");

        // activate 스크립트
        let activate_path = bin_dir.join("activate");
        let activate_content = format!(
            "# mock activate script\n\
             export VIRTUAL_ENV=\"{}\"\n",
            venv_dir.display()
        );
        fs::write(&activate_path, activate_content).expect("Failed to create activate script");

        venv_dir
    }

    /// mock virtualenvwrapper 가상환경을 생성한다.
    ///
    /// virtualenvwrapper는 `$WORKON_HOME/{name}` 구조를 사용.
    ///
    /// # Arguments
    ///
    /// * `name` - 가상환경 이름
    ///
    /// # Returns
    ///
    /// 생성된 가상환경 경로
    ///
    /// # Example
    ///
    /// ```ignore
    /// let env = IsolatedTestEnv::new();
    /// let venv_path = env.setup_mock_venvwrapper_env("myenv");
    /// ```
    pub fn setup_mock_venvwrapper_env(&self, name: &str) -> PathBuf {
        let venv_dir = self.workon_home.join(name);
        let bin_dir = venv_dir.join("bin");
        fs::create_dir_all(&bin_dir).expect("Failed to create venvwrapper bin dir");

        // pyvenv.cfg 생성
        let pyvenv_cfg = venv_dir.join("pyvenv.cfg");
        let cfg_content = "home = /usr/bin\n\
             include-system-site-packages = false\n\
             version = 3.11.0\n";
        fs::write(&pyvenv_cfg, cfg_content).expect("Failed to create pyvenv.cfg");

        // mock executables
        let python_path = bin_dir.join("python");
        fs::write(&python_path, "#!/bin/sh\n# mock venvwrapper python")
            .expect("Failed to create mock python");

        let activate_path = bin_dir.join("activate");
        let activate_content = format!(
            "# mock activate script\n\
             export VIRTUAL_ENV=\"{}\"\n",
            venv_dir.display()
        );
        fs::write(&activate_path, activate_content).expect("Failed to create activate script");

        venv_dir
    }

    /// mock 쉘 설정 파일을 생성한다.
    ///
    /// # Arguments
    ///
    /// * `shell` - 쉘 종류 ("bash", "zsh", "fish")
    ///
    /// # Returns
    ///
    /// 생성된 설정 파일 경로
    ///
    /// # Example
    ///
    /// ```ignore
    /// let env = IsolatedTestEnv::new();
    /// let rc_path = env.setup_mock_shell_rc("bash");
    /// // ~/.bashrc 생성됨
    /// ```
    pub fn setup_mock_shell_rc(&self, shell: &str) -> PathBuf {
        let home_path = self.home.path();

        let (rc_path, content) = match shell {
            "bash" => {
                let path = home_path.join(".bashrc");
                let content = r#"# ~/.bashrc
# mock bashrc for testing

# pyenv init
if command -v pyenv 1>/dev/null 2>&1; then
    eval "$(pyenv init -)"
fi

# virtualenvwrapper
export WORKON_HOME="$HOME/.virtualenvs"
source /usr/local/bin/virtualenvwrapper.sh 2>/dev/null
"#;
                (path, content)
            }
            "zsh" => {
                let path = home_path.join(".zshrc");
                let content = r#"# ~/.zshrc
# mock zshrc for testing

# pyenv init
if command -v pyenv 1>/dev/null 2>&1; then
    eval "$(pyenv init -)"
fi

# virtualenvwrapper
export WORKON_HOME="$HOME/.virtualenvs"
source /usr/local/bin/virtualenvwrapper.sh 2>/dev/null
"#;
                (path, content)
            }
            "fish" => {
                // fish는 ~/.config/fish/config.fish 사용
                let config_dir = home_path.join(".config").join("fish");
                fs::create_dir_all(&config_dir).expect("Failed to create fish config dir");
                let path = config_dir.join("config.fish");
                let content = r#"# ~/.config/fish/config.fish
# mock fish config for testing

# pyenv init
if command -v pyenv > /dev/null
    pyenv init - | source
end

# virtualenvwrapper
set -gx WORKON_HOME "$HOME/.virtualenvs"
"#;
                (path, content)
            }
            other => panic!("Unsupported shell: {}", other),
        };

        fs::write(&rc_path, content).expect("Failed to create shell rc file");
        rc_path
    }

    /// 홈 디렉토리 경로를 반환한다.
    pub fn home_path(&self) -> &std::path::Path {
        self.home.path()
    }
}

impl Default for IsolatedTestEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for IsolatedTestEnv {
    fn drop(&mut self) {
        // 원래 환경 변수로 복원
        // SAFETY: 테스트 환경에서 단일 스레드로 실행됨
        unsafe {
            for (var, original_value) in &self.original_env {
                match original_value {
                    Some(value) => std::env::set_var(var, value),
                    None => std::env::remove_var(var),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isolated_env_creates_directories() {
        let env = IsolatedTestEnv::new();

        assert!(env.scoop_home.exists());
        assert!(env.pyenv_root.exists());
        assert!(env.workon_home.exists());
        assert!(env.conda_prefix.exists());
    }

    #[test]
    fn test_isolated_env_sets_env_vars() {
        let env = IsolatedTestEnv::new();

        assert_eq!(
            std::env::var("SCOOP_HOME").ok(),
            Some(env.scoop_home.to_string_lossy().to_string())
        );
        assert_eq!(
            std::env::var("PYENV_ROOT").ok(),
            Some(env.pyenv_root.to_string_lossy().to_string())
        );
    }

    #[test]
    fn test_mock_pyenv_creates_structure() {
        let env = IsolatedTestEnv::new();
        env.setup_mock_pyenv(&["3.11.0", "3.12.0"]);

        let python_311 = env.pyenv_root.join("versions/3.11.0/bin/python");
        let python_312 = env.pyenv_root.join("versions/3.12.0/bin/python");

        assert!(python_311.exists());
        assert!(python_312.exists());
    }

    #[test]
    fn test_mock_pyenv_virtualenv_creates_pyvenv_cfg() {
        let env = IsolatedTestEnv::new();
        env.setup_mock_pyenv(&["3.11.0"]);
        let venv_path = env.setup_mock_pyenv_virtualenv("myenv", "3.11.0");

        assert!(venv_path.join("pyvenv.cfg").exists());
        assert!(venv_path.join("bin/python").exists());
        assert!(venv_path.join("bin/activate").exists());
    }

    #[test]
    fn test_mock_venvwrapper_creates_structure() {
        let env = IsolatedTestEnv::new();
        let venv_path = env.setup_mock_venvwrapper_env("myenv");

        assert!(venv_path.join("pyvenv.cfg").exists());
        assert!(venv_path.join("bin/python").exists());
    }

    #[test]
    fn test_mock_shell_rc_bash() {
        let env = IsolatedTestEnv::new();
        let rc_path = env.setup_mock_shell_rc("bash");

        assert!(rc_path.exists());
        assert!(rc_path.ends_with(".bashrc"));
    }

    #[test]
    fn test_mock_shell_rc_fish() {
        let env = IsolatedTestEnv::new();
        let rc_path = env.setup_mock_shell_rc("fish");

        assert!(rc_path.exists());
        assert!(rc_path.ends_with("config.fish"));
    }
}
