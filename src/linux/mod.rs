use crate::backend::Backend;
use crate::{Schedule, Status};
use command::{Runner, System};

mod command;
mod gnome;
mod hyprsunset;
mod kde;
mod locale;
mod noctalia;
mod transaction;

pub use self::locale::Locale;

const OVERRIDE: &str = "NIGHTLIGHT_BACKEND";

const BACKENDS: &[&str] = &["gnome", "kde", "noctalia", "hyprsunset"];

pub(crate) fn backend() -> Box<dyn Backend> {
    let session = Session {
        var: &|name| std::env::var(name).ok(),
        runner: &System,
    };
    match session.select() {
        Ok(name) => build(name),
        Err(reason) => Box::new(Unavailable(reason)),
    }
}

pub(crate) fn backend_named(name: &str) -> Result<Box<dyn Backend>, String> {
    match BACKENDS.iter().find(|known| **known == name) {
        Some(name) => Ok(build(name)),
        None => Err(unknown(name)),
    }
}

fn build(name: &str) -> Box<dyn Backend> {
    match name {
        "gnome" => Box::new(gnome::Gnome::new()),
        "noctalia" => Box::new(noctalia::Noctalia::new()),
        "hyprsunset" => Box::new(hyprsunset::Hyprsunset::new()),
        "kde" => Box::new(kde::Kde::new()),
        _ => unreachable!("unknown backend {}", name),
    }
}

fn unknown(name: &str) -> String {
    format!(
        "Unknown night light backend '{}'. Supported backends: {}",
        name,
        BACKENDS.join(", ")
    )
}

struct Session<'a> {
    var: &'a dyn Fn(&str) -> Option<String>,
    runner: &'a dyn Runner,
}

impl Session<'_> {
    fn select(&self) -> Result<&'static str, String> {
        if let Some(name) = (self.var)(OVERRIDE).filter(|name| !name.is_empty()) {
            return BACKENDS
                .iter()
                .find(|known| **known == name)
                .copied()
                .ok_or_else(|| unknown(&name));
        }

        if self.desktop_is("GNOME") || self.desktop_is("PANTHEON") {
            return Ok("gnome");
        }

        // Noctalia can run on desktops that have their own night light (e.g. KDE).
        if noctalia::running(self.runner) {
            return Ok("noctalia");
        }

        if hyprsunset::running(self.var) {
            return Ok("hyprsunset");
        }

        if self.desktop_is("KDE") {
            return Ok("kde");
        }

        if self.desktops().is_empty() && gnome::installed(self.runner) {
            return Ok("gnome");
        }

        Err(format!(
            "No supported night light found for this session (supported: {}). \
             Set {} to choose one explicitly.",
            BACKENDS.join(", "),
            OVERRIDE
        ))
    }

    fn desktops(&self) -> Vec<String> {
        (self.var)("XDG_CURRENT_DESKTOP")
            .unwrap_or_default()
            .split(':')
            .filter(|desktop| !desktop.is_empty())
            .map(str::to_ascii_uppercase)
            .collect()
    }

    fn desktop_is(&self, desktop: &str) -> bool {
        self.desktops().iter().any(|name| name == desktop)
    }
}

struct Unavailable(String);

impl Backend for Unavailable {
    fn name(&self) -> &'static str {
        "none"
    }

    fn status(&self) -> Result<Status, String> {
        Err(self.0.clone())
    }

    fn set_status(&self, _: Status) -> Result<(), String> {
        Err(self.0.clone())
    }

    fn schedule(&self) -> Result<Schedule, String> {
        Err(self.0.clone())
    }

    fn set_schedule(&self, _: Schedule) -> Result<(), String> {
        Err(self.0.clone())
    }

    fn strength(&self) -> Result<i32, String> {
        Err(self.0.clone())
    }

    fn set_strength(&self, _: i32) -> Result<(), String> {
        Err(self.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::command::Runner;
    use super::Session;
    use std::collections::HashMap;

    struct FakeRunner {
        succeeds: Vec<&'static str>,
    }

    impl Runner for FakeRunner {
        fn run(&self, program: &str, args: &[&str]) -> Result<String, String> {
            let call = format!("{} {}", program, args.join(" "));
            match self.succeeds.iter().any(|prefix| call.starts_with(prefix)) {
                true => Ok(String::new()),
                false => Err(format!("{} failed", call)),
            }
        }
    }

    fn select(vars: &[(&str, &str)], succeeds: Vec<&'static str>) -> Result<&'static str, String> {
        let vars: HashMap<String, String> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let runner = FakeRunner { succeeds };
        Session {
            var: &|name| vars.get(name).cloned(),
            runner: &runner,
        }
        .select()
    }

    #[test]
    fn gnome_sessions_are_detected() {
        assert_eq!(select(&[("XDG_CURRENT_DESKTOP", "GNOME")], vec![]), Ok("gnome"));
        assert_eq!(select(&[("XDG_CURRENT_DESKTOP", "ubuntu:GNOME")], vec![]), Ok("gnome"));
        assert_eq!(select(&[("XDG_CURRENT_DESKTOP", "Pantheon")], vec![]), Ok("gnome"));
    }

    #[test]
    fn running_noctalia_is_detected_on_any_other_compositor() {
        let probe = vec!["noctalia msg config-get nightlight.enabled"];
        assert_eq!(select(&[("XDG_CURRENT_DESKTOP", "niri")], probe.clone()), Ok("noctalia"));
        assert_eq!(select(&[("XDG_CURRENT_DESKTOP", "Hyprland")], probe.clone()), Ok("noctalia"));
        assert_eq!(select(&[], probe.clone()), Ok("noctalia"));
        assert_eq!(select(&[("XDG_CURRENT_DESKTOP", "GNOME")], probe), Ok("gnome"));
        assert!(select(&[("XDG_CURRENT_DESKTOP", "niri")], vec![]).is_err());
    }

    #[test]
    fn kde_sessions_are_detected_unless_noctalia_runs_on_them() {
        assert_eq!(select(&[("XDG_CURRENT_DESKTOP", "KDE")], vec![]), Ok("kde"));
        let noctalia = vec!["noctalia msg config-get nightlight.enabled"];
        assert_eq!(select(&[("XDG_CURRENT_DESKTOP", "KDE")], noctalia), Ok("noctalia"));
    }

    #[test]
    fn listening_hyprsunset_is_detected_after_noctalia() {
        let runtime = std::env::temp_dir().join(format!("nightlight-detect-{}", std::process::id()));
        let socket = runtime.join("hypr/sig/.hyprsunset.sock");
        std::fs::create_dir_all(socket.parent().unwrap()).unwrap();
        let _ = std::fs::remove_file(&socket);
        let runtime_dir = runtime.to_str().unwrap();
        let vars = [
            ("XDG_CURRENT_DESKTOP", "Hyprland"),
            ("XDG_RUNTIME_DIR", runtime_dir),
            ("HYPRLAND_INSTANCE_SIGNATURE", "sig"),
        ];

        assert!(select(&vars, vec![]).is_err());
        let listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
        assert_eq!(select(&vars, vec![]), Ok("hyprsunset"));
        let noctalia = vec!["noctalia msg config-get nightlight.enabled"];
        assert_eq!(select(&vars, noctalia), Ok("noctalia"));

        drop(listener);
        assert!(select(&vars, vec![]).is_err(), "stale sockets are ignored");
        let _ = std::fs::remove_dir_all(&runtime);
    }

    #[test]
    fn override_wins_and_is_validated() {
        assert_eq!(
            select(&[("NIGHTLIGHT_BACKEND", "gnome"), ("XDG_CURRENT_DESKTOP", "niri")], vec![]),
            Ok("gnome")
        );
        let error = select(&[("NIGHTLIGHT_BACKEND", "bogus")], vec![]).unwrap_err();
        assert!(error.contains("Unknown night light backend 'bogus'"), "{}", error);
        assert!(error.contains("gnome"), "{}", error);
    }

    #[test]
    fn empty_override_is_ignored() {
        assert_eq!(
            select(&[("NIGHTLIGHT_BACKEND", ""), ("XDG_CURRENT_DESKTOP", "GNOME")], vec![]),
            Ok("gnome")
        );
    }

    #[test]
    fn non_graphical_sessions_fall_back_to_installed_gnome_schema() {
        assert_eq!(select(&[], vec!["gsettings get"]), Ok("gnome"));
        assert!(select(&[], vec![]).is_err());
    }

    #[test]
    fn other_desktops_do_not_fall_back_to_gnome() {
        let error = select(&[("XDG_CURRENT_DESKTOP", "sway")], vec!["gsettings get"]).unwrap_err();
        assert!(error.contains("NIGHTLIGHT_BACKEND"), "{}", error);
    }
}
