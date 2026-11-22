# nightlight

> A CLI for configuring "Night Shift" on macOS (and night light on Linux) 🌕🌖🌗🌘🌑

This crate also doubles as a Rust library. 🦀

### Why?

The "Night Shift" feature on macOS is a convenient, built-in feature
that can theoretically accomplish most of what third-party alternatives
(like [f.lux®](https://justgetflux.com/)) are capable of. However, as
of now, there is no way to programmatically configure Night Shift (without
entering the system preferences GUI), making its current usage more limited.

This `nightlight` CLI aims to enable such access via a few simple commands.
(Or, alternatively, via library access for other Rust tools.)

## Installing

### via Homebrew

```
brew install smudge/smudge/nightlight
```

### via prebuilt binaries

Download the archive for your platform from the
[latest release](https://github.com/smudge/nightlight/releases/latest)
(`aarch64-apple-darwin` for Apple Silicon Macs, `x86_64-apple-darwin` for Intel
Macs, or `x86_64`/`aarch64-unknown-linux-musl` for Linux), extract it, and move
the `nightlight` binary somewhere on your `PATH`, e.g.:

```
tar -xzf nightlight-v*.tar.gz
sudo mv nightlight-v*/nightlight /usr/local/bin/
```

The Linux binaries are statically linked and run on any distribution.

### via Cargo

[Set up Rust/Cargo](https://doc.rust-lang.org/book/ch01-01-installation.html)
and install from crates.io by running:

```
cargo install nightlight
```

The Linux build has no system library dependencies: it talks to whatever
already manages night light in your session (see below) at runtime.

## Platform Support

On **macOS**, make sure you are running macOS Sierra or newer.

On **Linux**, Wayland has no standard night light interface, so `nightlight`
drives the component that owns it in your session, detected in this order:

| Backend | Detected when | Notes |
|---|---|---|
| `gnome` | `XDG_CURRENT_DESKTOP` includes `GNOME` or `Pantheon` (e.g. Ubuntu, elementary OS) | Uses `gsettings` |
| `noctalia` | A [Noctalia](https://noctalia.dev) (v5+) shell is running, on any compositor | Uses `noctalia msg` |
| `hyprsunset` | [hyprsunset](https://wiki.hypr.land/Hypr-Ecosystem/hyprsunset/) is listening (e.g. Hyprland/Omarchy) | Talks to its socket directly |
| `kde` | `XDG_CURRENT_DESKTOP` includes `KDE` (Plasma 6) | Uses `kreadconfig6`/`kwriteconfig6` |

Outside a graphical session (e.g. cron), `gnome` is used if its settings
schema is installed. Set `NIGHTLIGHT_BACKEND` to one of the names above to
skip detection.

A few backend differences to be aware of:

* On Linux, `on`/`off` enable or disable night light on its schedule (rather
  than overriding the schedule until the next transition, as on macOS), and
  `schedule stop` turns night light off.
* **Noctalia** and **KDE** (Plasma 6.5+) share their day/night schedule with
  automatic theme switching, so a custom schedule changes that as well.
  Noctalia changes are saved to its GUI-managed `settings.toml`; your
  hand-written config is never modified.
* **hyprsunset** has no sunset/sunrise mode, and its time-based profiles live
  in `hyprsunset.conf`, so `schedule` commands aren't supported yet (`status`
  reports a `null` schedule). Reading the on/off state, and changing the
  temperature (which is only allowed while it's on), require hyprsunset
  0.4.0 or newer.
* **KDE**'s "always on" mode has no schedule, so `status` reports a `null`
  schedule in that mode.

## Usage

### Command-Line Interface

#### Manual On/Off:

Turn Night Shift on (until tomorrow/sunrise):

```
nightlight on
```

Turn Night Shift off:

```
nightlight off
```

View current on/off status:

```
nightlight status
```

For machine-readable output, add `--json` to **any command**, before, within, or after the command arguments:

```bash
nightlight --json status
nightlight temp --json
nightlight --json temp 70
nightlight schedule 7:45pm 6am --json
```

Reads return typed JSON objects: `status` includes a `status` string and `schedule` object; `temp` returns `{"temperature":70}` (on Linux, it also includes the desktop's configured night temperature, e.g. `"kelvin":4250`); and `schedule` returns its configuration. Custom schedules have `from` and `to` in 24-hour `HH:MM` format. Successful updates return an object with `ok: true`, `command`, and `value`. Errors return JSON on stderr with `ok: false` and `error`, and retain a nonzero exit status.

Without `--json`, commands retain their usual human-readable output.

Toggle on or off based on current status:

```
nightlight toggle
nightlight <none>
```

#### Controlling the Temperature:

View current temperature setting:

```
nightlight temp
```

Set color temperature (a number from 0 to 100):

```
nightlight temp 70
```

#### Scheduling:

View the current schedule:

```
nightlight schedule
```

Start schedule from sunset to sunrise:

```
nightlight schedule start
```

Start a custom schedule (in 12 or 24-hour time format):

```
nightlight schedule 19:45 6:00
nightlight schedule 7:45pm 6am
```

Stop the current schedule:

```
nightlight schedule off
```

### Rust API

In addition to a CLI, `nightlight` can be pulled-in as a dependency for other Rust crates:

```
nightlight = "1.1.0"
```

Here's an example `fn` that toggles Night Shift off,
changes the schedule and color temperature preference,
and then toggles the feature back on:

```rust
extern crate nightlight;

use nightlight::{NightLight, Schedule};

fn main() {
    let night_light = NightLight::new();

    if night_light.status().is_on() {
        night_light.off().unwrap(),
    }

    println!("Setting schedule and temperature...");
    night_light.set_schedule(Schedule::SunsetToSunrise).unwrap();
    night_light.set_temp(70).unwrap();

    println!("Turning Night Shift on...");
    night_light.on().unwrap();
}
```

## Todo:

- [X] Ability to see current status of Night Shift
- [X] Ability to enable/disable sunrise/sundown schedule
- [X] Ability to enable/disable custom schedules
- [X] Ensure that changing schedule doesn't affect on/off state.
- [X] Make time display as properly-formatted 12-hour time
- [X] Use system config for time parse/format (12 vs 24).
- [X] API improvements
- [X] Other release mechanisms (like `brew`)
- [ ] Consider command outputs: concise, human-readable, machine-parsable.
- [ ] Full lib documentation
- [ ] Test coverage of schedule/time parsing.
- [ ] Tests that use fake/stub ObjC library.
- [ ] Support for "Automatically adjust brightness" feature.
- [X] Linux support (GNOME, KDE Plasma, Noctalia, hyprsunset)
- [ ] hyprsunset schedules
- [ ] Windows support (see [sem-reg-rs](https://github.com/Enyium/sem-reg-rs) in the meantime)

## Contributing

* Check the issue tracker and consider creating a new issue.
* Fork the project and create a new branch for your contribution.
* Write, commit, and push your contribution to your branch.
* Make sure the project builds (`cargo build`) and functionality still works as expected.
* Submit a pull request.

## Thanks To:

* The team at Apple for introducing this feature in macOS Sierra
* GitHub user `jenghis` for the (now archived) [nshift](https://github.com/jenghis/nshift) repo/CLI
* The maintainers of the Rust [objc crate](https://github.com/SSheldon/rust-objc)
* Carol Nichols and Steve Klabnik for the [official book](https://doc.rust-lang.org/book/) on Rust
* ABH, AF, CB, JP, and MK for brainstorming crate names with me

## License

`nightlight` is released under the [MIT License](LICENSE).
