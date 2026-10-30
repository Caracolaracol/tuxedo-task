//! Smoke-test the actual executable's identity, aliases and upgrade guidance.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn stdout_at(exe: &Path, arg: &str) -> String {
    // Linux can transiently return ETXTBSY for a freshly copied executable
    // while parallel test children finish inheriting/closing file handles.
    // Retry that specific condition briefly, never other execution failures.
    let mut retries = 0;
    let output = loop {
        match Command::new(exe).arg(arg).output() {
            Ok(output) => break output,
            Err(error)
                if error.kind() == std::io::ErrorKind::ExecutableFileBusy && retries < 20 =>
            {
                retries += 1;
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
            Err(error) => panic!("run tuxedo: {error}"),
        }
    };
    assert!(output.status.success(), "{output:?}");
    String::from_utf8(output.stdout).expect("UTF-8 output")
}

fn stdout(arg: &str) -> String {
    stdout_at(Path::new(env!("CARGO_BIN_EXE_tuxedo")), arg)
}

struct TestInstallation {
    root: PathBuf,
    original: PathBuf,
    alias: PathBuf,
}

impl TestInstallation {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("tuxedo-dist-{}-{stamp}", std::process::id()));
        let bin = root.join("Cellar/tuxedotask/test/bin");
        std::fs::create_dir_all(&bin).expect("fixture bin directory");
        let original = bin.join(format!("tuxedo{}", std::env::consts::EXE_SUFFIX));
        let alias = bin.join(format!("tuxedotask{}", std::env::consts::EXE_SUFFIX));
        std::fs::copy(env!("CARGO_BIN_EXE_tuxedo"), &original).expect("copy fixture executable");
        #[cfg(unix)]
        std::os::unix::fs::symlink("tuxedo", &alias).expect("fixture alias");
        #[cfg(not(unix))]
        std::fs::copy(&original, &alias).expect("copy fixture alias");
        Self {
            root,
            original,
            alias,
        }
    }
}

impl Drop for TestInstallation {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn version_identifies_the_fork() {
    assert_eq!(
        stdout("--version").trim(),
        concat!("Tuxedo Task ", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn help_uses_the_compatible_executable_name() {
    let output = stdout("--help");
    assert!(output.contains("usage: tuxedo [FILE]"));
    assert!(output.contains("`tuxedotask` as an alias"));
}

#[test]
fn source_upgrade_instructions_install_the_fork_not_upstream() {
    let output = stdout("update");
    if cfg!(windows) {
        assert!(output.contains("Setup.exe"));
        assert!(output.contains("https://github.com/Caracolaracol/tuxedo-task/releases"));
    } else {
        assert!(output.contains(
            "cargo install --git https://github.com/Caracolaracol/tuxedo-task --locked --force"
        ));
    }
    assert!(!output.contains("github.com/webstonehq"));
    assert!(!output.contains("brew upgrade"));
}

#[test]
fn packaged_alias_preserves_the_original_command() {
    let installation = TestInstallation::new();
    assert_eq!(
        stdout_at(&installation.original, "--version"),
        stdout_at(&installation.alias, "--version")
    );
    assert_eq!(
        stdout_at(&installation.original, "--help"),
        stdout_at(&installation.alias, "--help")
    );
}

#[test]
#[cfg(unix)]
fn homebrew_upgrade_uses_the_forks_formula() {
    let installation = TestInstallation::new();
    let output = stdout_at(&installation.alias, "update");
    assert!(output.contains("brew update && brew upgrade caracolaracol/tap/tuxedotask"));
    assert!(!output.contains("cargo install"));
    assert!(!output.contains("webstonehq"));
}
