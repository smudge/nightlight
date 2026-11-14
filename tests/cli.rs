use std::process::Command;

#[test]
fn help_exits_successfully_without_initializing_the_backend() {
    for option in ["help", "-h", "--help"] {
        let output = Command::new(env!("CARGO_BIN_EXE_nightlight"))
            .arg(option)
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", option);
        assert!(String::from_utf8_lossy(&output.stdout).contains("Available Commands"));
    }
}

#[test]
fn unknown_command_fails() {
    let output = Command::new(env!("CARGO_BIN_EXE_nightlight"))
        .arg("nonexistent-command")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Invalid arguments"));
}

#[test]
fn extra_arguments_fail() {
    let output = Command::new(env!("CARGO_BIN_EXE_nightlight"))
        .args(["on", "unexpected"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Invalid arguments"));
}
