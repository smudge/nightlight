use crate::kelvin;
use crate::backend::Backend;
use crate::{Schedule, Status};
use std::io::{Read, Write};
use std::net::Shutdown;
use std::os::unix::fs::MetadataExt;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

const DEFAULT_KELVIN: u32 = 4000;
const NEUTRAL_KELVIN: u32 = 6000;

const UNSUPPORTED_SCHEDULE: &str = "Schedules aren't supported by the hyprsunset backend yet; \
     configure profiles in ~/.config/hypr/hyprsunset.conf instead";

pub(crate) trait Socket {
    fn send(&self, request: &str) -> Result<String, String>;
}

pub(crate) struct UnixSocket {
    path: PathBuf,
}

impl Socket for UnixSocket {
    fn send(&self, request: &str) -> Result<String, String> {
        let error = |error: std::io::Error| format!("Unable to reach hyprsunset: {}", error);
        let mut stream = UnixStream::connect(&self.path).map_err(error)?;
        stream.set_read_timeout(Some(Duration::from_secs(2))).map_err(error)?;
        stream.set_write_timeout(Some(Duration::from_secs(2))).map_err(error)?;
        stream.write_all(request.as_bytes()).map_err(error)?;
        // hyprsunset holds the connection open until it reads EOF.
        stream.shutdown(Shutdown::Write).map_err(error)?;
        let mut reply = String::new();
        stream.read_to_string(&mut reply).map_err(error)?;
        Ok(reply.trim().to_string())
    }
}

pub(crate) fn socket_path(var: &dyn Fn(&str) -> Option<String>) -> PathBuf {
    let runtime = var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let uid = std::fs::metadata("/proc/self").map(|m| m.uid()).unwrap_or(0);
            PathBuf::from(format!("/run/user/{}", uid))
        });
    let hypr = runtime.join("hypr");
    match var("HYPRLAND_INSTANCE_SIGNATURE").filter(|sig| !sig.is_empty()) {
        Some(signature) => hypr.join(signature).join(".hyprsunset.sock"),
        None => hypr.join(".hyprsunset.sock"),
    }
}

pub(crate) fn running(var: &dyn Fn(&str) -> Option<String>) -> bool {
    UnixStream::connect(socket_path(var)).is_ok()
}

pub(crate) struct Hyprsunset<S: Socket = UnixSocket> {
    socket: S,
}

impl Hyprsunset {
    pub fn new() -> Hyprsunset {
        Hyprsunset {
            socket: UnixSocket {
                path: socket_path(&|name| std::env::var(name).ok()),
            },
        }
    }
}

impl<S: Socket> Hyprsunset<S> {
    fn command(&self, request: &str) -> Result<(), String> {
        match self.socket.send(request)?.as_str() {
            "ok" => Ok(()),
            reply => Err(format!("hyprsunset rejected '{}': {}", request, reply)),
        }
    }

    fn live_kelvin(&self) -> Result<u32, String> {
        let reply = self.socket.send("temperature")?;
        reply
            .parse()
            .map_err(|_| format!("Unexpected hyprsunset temperature: {}", reply))
    }

    // Before v0.4.0, any "identity ..." request (even "identity get") turns the
    // filter off. "reset <bogus>" is harmless on every version and tells them apart.
    fn identity(&self) -> Result<bool, String> {
        if self.socket.send("reset nightlight-version-probe")? == "invalid command" {
            return Err(
                "Reading hyprsunset's on/off state requires hyprsunset 0.4.0 or newer".into(),
            );
        }
        match self.socket.send("identity get")?.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            reply => Err(format!("Unexpected hyprsunset identity: {}", reply)),
        }
    }
}

impl<S: Socket> Backend for Hyprsunset<S> {
    fn name(&self) -> &'static str {
        "hyprsunset"
    }

    fn status(&self) -> Result<Status, String> {
        Ok(Status::from_bool(!self.identity()?))
    }

    fn set_status(&self, status: Status) -> Result<(), String> {
        match status {
            Status::Off => self.command("identity"),
            Status::On => {
                let kelvin = match self.live_kelvin()? {
                    kelvin if kelvin >= NEUTRAL_KELVIN => DEFAULT_KELVIN,
                    kelvin => kelvin,
                };
                self.command(&format!("temperature {}", kelvin))
            }
        }
    }

    fn has_schedule(&self) -> bool {
        false
    }

    fn schedule(&self) -> Result<Schedule, String> {
        Err(UNSUPPORTED_SCHEDULE.into())
    }

    fn set_schedule(&self, _: Schedule) -> Result<(), String> {
        Err(UNSUPPORTED_SCHEDULE.into())
    }

    fn strength(&self) -> Result<i32, String> {
        Ok(kelvin::to_strength(self.live_kelvin()? as f64))
    }

    fn kelvin(&self) -> Result<Option<u32>, String> {
        self.live_kelvin().map(Some)
    }

    fn set_strength(&self, strength: i32) -> Result<(), String> {
        if self.identity()? {
            return Err(
                "hyprsunset is off; turn night light on before changing its temperature".into(),
            );
        }
        self.command(&format!("temperature {}", kelvin::from_strength(strength)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    struct FakeHyprsunset {
        modern: bool,
        kelvin: RefCell<u32>,
        identity: RefCell<bool>,
        requests: RefCell<Vec<String>>,
    }

    impl FakeHyprsunset {
        fn new(modern: bool, kelvin: u32, identity: bool) -> Hyprsunset<FakeHyprsunset> {
            Hyprsunset {
                socket: FakeHyprsunset {
                    modern,
                    kelvin: RefCell::new(kelvin),
                    identity: RefCell::new(identity),
                    requests: RefCell::new(vec![]),
                },
            }
        }
    }

    impl Socket for FakeHyprsunset {
        fn send(&self, request: &str) -> Result<String, String> {
            self.requests.borrow_mut().push(request.to_string());
            let mut words = request.split(' ');
            let reply = match (words.next(), words.next(), self.modern) {
                (Some("temperature"), None, _) => self.kelvin.borrow().to_string(),
                (Some("temperature"), Some(value), _) => match value.parse::<u32>() {
                    Ok(kelvin) if (1000..=20000).contains(&kelvin) => {
                        *self.kelvin.borrow_mut() = kelvin;
                        *self.identity.borrow_mut() = false;
                        "ok".into()
                    }
                    _ => "Invalid temperature (should be an integer in range 1000-20000)".into(),
                },
                (Some("identity"), _, false) | (Some("identity"), None, true) => {
                    *self.identity.borrow_mut() = true;
                    "ok".into()
                }
                (Some("identity"), Some("get"), true) => self.identity.borrow().to_string(),
                (Some("reset"), Some(_), true) => "No profile is currently loaded".into(),
                _ => "invalid command".into(),
            };
            Ok(reply)
        }
    }

    #[test]
    fn status_reflects_identity() {
        assert_eq!(FakeHyprsunset::new(true, 4000, false).status(), Ok(Status::On));
        assert_eq!(FakeHyprsunset::new(true, 4000, true).status(), Ok(Status::Off));
    }

    #[test]
    fn off_bypasses_the_filter_and_on_restores_the_temperature() {
        let hyprsunset = FakeHyprsunset::new(true, 4250, false);
        hyprsunset.set_status(Status::Off).unwrap();
        assert_eq!(hyprsunset.status(), Ok(Status::Off));
        hyprsunset.set_status(Status::On).unwrap();
        assert_eq!(hyprsunset.status(), Ok(Status::On));
        assert_eq!(*hyprsunset.socket.kelvin.borrow(), 4250);
    }

    #[test]
    fn on_from_neutral_uses_a_warm_default() {
        let hyprsunset = FakeHyprsunset::new(true, 6000, true);
        hyprsunset.set_status(Status::On).unwrap();
        assert_eq!(*hyprsunset.socket.kelvin.borrow(), DEFAULT_KELVIN);
    }

    #[test]
    fn old_versions_are_never_sent_identity_get() {
        let hyprsunset = FakeHyprsunset::new(false, 4000, false);
        let error = hyprsunset.status().unwrap_err();
        assert!(error.contains("0.4.0"), "{}", error);
        assert!(hyprsunset.set_strength(50).is_err());
        assert!(!*hyprsunset.socket.identity.borrow());
        assert!(!hyprsunset
            .socket
            .requests
            .borrow()
            .iter()
            .any(|request| request.starts_with("identity")));
    }

    #[test]
    fn old_versions_still_toggle_and_report_temperature() {
        let hyprsunset = FakeHyprsunset::new(false, 4750, false);
        assert_eq!(hyprsunset.strength(), Ok(50));
        assert_eq!(hyprsunset.kelvin(), Ok(Some(4750)));
        hyprsunset.set_status(Status::Off).unwrap();
        assert!(*hyprsunset.socket.identity.borrow());
        hyprsunset.set_status(Status::On).unwrap();
        assert!(!*hyprsunset.socket.identity.borrow());
    }

    #[test]
    fn strength_changes_the_live_temperature_only_while_on() {
        let hyprsunset = FakeHyprsunset::new(true, 4000, false);
        hyprsunset.set_strength(100).unwrap();
        assert_eq!(*hyprsunset.socket.kelvin.borrow(), 3500);
        assert_eq!(hyprsunset.strength(), Ok(100));

        let off = FakeHyprsunset::new(true, 4000, true);
        assert!(off.set_strength(100).unwrap_err().contains("turn night light on"));
        assert_eq!(*off.socket.kelvin.borrow(), 4000);
        assert!(*off.socket.identity.borrow());
    }

    #[test]
    fn schedules_are_unsupported() {
        let hyprsunset = FakeHyprsunset::new(true, 4000, false);
        assert!(!hyprsunset.has_schedule());
        assert!(hyprsunset.schedule().is_err());
        assert!(hyprsunset.set_schedule(Schedule::SunsetToSunrise).is_err());
    }

    #[test]
    fn rejected_requests_are_errors() {
        struct Rejecting;
        impl Socket for Rejecting {
            fn send(&self, _: &str) -> Result<String, String> {
                Ok("invalid command".into())
            }
        }
        let hyprsunset = Hyprsunset { socket: Rejecting };
        assert!(hyprsunset.set_status(Status::Off).unwrap_err().contains("rejected"));
    }

    #[test]
    fn socket_path_follows_hyprsunset() {
        let vars: HashMap<&str, &str> = [
            ("XDG_RUNTIME_DIR", "/run/user/1000"),
            ("HYPRLAND_INSTANCE_SIGNATURE", "abc_123"),
        ]
        .into_iter()
        .collect();
        let var = |name: &str| vars.get(name).map(|v| v.to_string());
        assert_eq!(
            socket_path(&var),
            PathBuf::from("/run/user/1000/hypr/abc_123/.hyprsunset.sock")
        );
        let var = |name: &str| (name == "XDG_RUNTIME_DIR").then(|| "/tmp/rt".to_string());
        assert_eq!(socket_path(&var), PathBuf::from("/tmp/rt/hypr/.hyprsunset.sock"));
    }

    #[test]
    fn unix_socket_round_trip() {
        use std::os::unix::net::UnixListener;
        let path = std::env::temp_dir().join(format!("nightlight-hs-{}.sock", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path).unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0u8; 1024];
            loop {
                let size = stream.read(&mut buffer).unwrap();
                if size == 0 {
                    break;
                }
                let request = String::from_utf8_lossy(&buffer[..size]).to_string();
                stream.write_all(format!("got {}", request).as_bytes()).unwrap();
            }
        });
        let socket = UnixSocket { path: path.clone() };
        assert_eq!(socket.send("temperature").unwrap(), "got temperature");
        server.join().unwrap();
        let _ = std::fs::remove_file(&path);
    }
}
