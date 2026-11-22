#![cfg(target_os = "linux")]

use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const SCHEMA: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<schemalist>
  <schema id="org.gnome.settings-daemon.plugins.color" path="/org/gnome/settings-daemon/plugins/color/">
    <key name="night-light-enabled" type="b"><default>false</default></key>
    <key name="night-light-temperature" type="u"><default>2700</default></key>
    <key name="night-light-schedule-automatic" type="b"><default>true</default></key>
    <key name="night-light-schedule-from" type="d"><default>20.0</default></key>
    <key name="night-light-schedule-to" type="d"><default>6.0</default></key>
  </schema>
</schemalist>
"#;

struct Session {
    root: PathBuf,
}

impl Session {
    fn new(name: &str) -> Option<Session> {
        let root = std::env::temp_dir().join(format!("nightlight-gnome-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("schemas")).unwrap();
        fs::write(root.join("schemas/nightlight-test.gschema.xml"), SCHEMA).unwrap();
        let compiled = Command::new("glib-compile-schemas")
            .arg(root.join("schemas"))
            .status();
        match compiled {
            Ok(status) if status.success() && has_gsettings() => Some(Session { root }),
            _ => {
                eprintln!("skipping: glib-compile-schemas/gsettings unavailable");
                None
            }
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_nightlight"));
        self.configure(&mut command);
        command
    }

    fn configure(&self, command: &mut Command) {
        command
            .env("NIGHTLIGHT_BACKEND", "gnome")
            .env("GSETTINGS_SCHEMA_DIR", self.root.join("schemas"))
            .env("GSETTINGS_BACKEND", "keyfile")
            .env("XDG_CONFIG_HOME", self.root.join("config"));
    }

    fn run(&self, args: &[&str]) -> Value {
        let output = self.command().arg("--json").args(args).output().unwrap();
        assert!(
            output.status.success(),
            "{:?}: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap_or(Value::Null)
    }

    fn gsettings(&self, key: &str) -> String {
        let mut command = Command::new("gsettings");
        self.configure(&mut command);
        let output = command
            .args(["get", "org.gnome.settings-daemon.plugins.color", key])
            .output()
            .unwrap();
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

// `gsettings --version` exits non-zero when no schemas are installed.
fn has_gsettings() -> bool {
    Command::new("gsettings").arg("--version").output().is_ok()
}

#[test]
fn schedules_round_trip_through_gsettings() {
    let Some(session) = Session::new("schedule") else { return };

    assert_eq!(session.run(&["status"])["status"], "off");

    session.run(&["schedule", "19:45", "6:30"]);
    let status = session.run(&["status"]);
    assert_eq!(status["status"], "on");
    assert_eq!(status["schedule"]["type"], "custom");
    assert_eq!(status["schedule"]["from"], "19:45");
    assert_eq!(status["schedule"]["to"], "06:30");
    assert_eq!(session.gsettings("night-light-schedule-from"), "19.75");

    session.run(&["schedule", "start"]);
    assert_eq!(session.run(&["schedule"])["schedule"]["type"], "sunset-to-sunrise");

    session.run(&["schedule", "stop"]);
    assert_eq!(session.run(&["status"])["schedule"]["type"], "off");
    assert_eq!(session.gsettings("night-light-schedule-to"), "6.5");
}

#[test]
fn temperature_and_toggle_go_through_gsettings() {
    let Some(session) = Session::new("temp") else { return };

    session.run(&["temp", "4750K"]);
    assert_eq!(session.gsettings("night-light-temperature"), "uint32 4750");
    assert_eq!(session.run(&["temp"])["temperature"], 50);

    session.run(&["on"]);
    assert_eq!(session.gsettings("night-light-enabled"), "true");
    session.run(&["toggle"]);
    assert_eq!(session.gsettings("night-light-enabled"), "false");
}
