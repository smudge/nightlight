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

#[test]
fn unsupported_command_shapes_fail_without_accessing_settings() {
    for args in [
        &["schedule", "bogus"][..],
        &["schedule", "start", "extra", "argument"][..],
        &["temp", "70", "extra"][..],
        &["status", "extra"][..],
        &["--help", "extra"][..],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_nightlight"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1), "{args:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("Invalid arguments"),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn json_flag_only_applies_to_status() {
    let output = Command::new(env!("CARGO_BIN_EXE_nightlight"))
        .args(["on", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Invalid arguments"));
}

#[test]
fn help_includes_json_status() {
    let output = Command::new(env!("CARGO_BIN_EXE_nightlight"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("status --json"));
}
