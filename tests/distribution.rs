//! Smoke-test the actual executable's public identity and upgrade guidance.

use std::process::Command;

fn stdout(arg: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_tuxedo"))
        .arg(arg)
        .output()
        .expect("run tuxedo");
    assert!(output.status.success(), "{output:?}");
    String::from_utf8(output.stdout).expect("UTF-8 output")
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
    assert!(stdout("--help").contains("usage: tuxedo [FILE]"));
}

#[test]
fn upgrade_instructions_install_the_fork_not_upstream() {
    let output = stdout("update");
    assert!(output.contains(
        "cargo install --git https://github.com/Caracolaracol/tuxedo-task --locked --force"
    ));
    assert!(!output.contains("github.com/webstonehq"));
    assert!(!output.contains("brew upgrade"));
}
