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
fn invalid_command_uses_json_errors_when_requested() {
    for arguments in [
        &["--json", "bogus"][..],
        &["on", "--json", "unexpected"][..],
        &["schedule", "bad", "--json"][..],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_nightlight"))
            .args(arguments)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1), "{:?}", arguments);
        let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(error["ok"], false);
        assert!(error["error"].as_str().unwrap().contains("Invalid arguments"));
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn help_includes_global_json() {
    let output = Command::new(env!("CARGO_BIN_EXE_nightlight"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("--json"));
}

#[test]
fn global_json_help_is_valid_json() {
    for arguments in [
        &["--json", "--help"][..],
        &["--help", "--json"][..],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_nightlight"))
            .args(arguments)
            .output()
            .unwrap();
        assert!(output.status.success());
        let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(response["ok"], true);
        assert!(response["help"].as_str().unwrap().contains("Available Commands"));
    }
}

#[test]
fn invalid_temperature_input_returns_json_without_a_backend() {
    let output = Command::new(env!("CARGO_BIN_EXE_nightlight"))
        .args(["temp", "not-a-temperature", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["ok"], false);
    assert!(error["error"].as_str().unwrap().contains("Color temperature"));
}
