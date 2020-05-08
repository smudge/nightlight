# nightlight

> A CLI for configuring "Night Shift" on macOS 🌕🌖🌗🌘🌑

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
brew tap smudge/smudge
brew install nightlight
```

### via Cargo

[Set up Rust/Cargo](https://doc.rust-lang.org/book/ch01-01-installation.html)
and install from crates.io by running:

```
cargo install nightlight
```

## Usage

First, make sure you are running macOS Sierra or newer.

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
nightlight = "0.1.1"
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
- [ ] Consider command outputs: concise, human-readable, machine-parsable.
- [ ] Full lib documentation
- [ ] Test coverage of schedule/time parsing.
- [ ] Tests that use fake/stub ObjC library.
- [ ] Support for "Automatically adjust brightness" feature.
- [X] Other release mechanisms (like `brew`)
- [ ] Cross-platform support (e.g. Windows' "Night Light")

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
