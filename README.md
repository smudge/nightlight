# nightshift

> A CLI for configuring "Night Shift" on macOS 🌕🌖🌗🌘🌑

This crate also doubles as a Rust library. 🦀

### Why?

The "Night Shift" feature on macOS is a convenient, first party alternative
to the more feature-rich [f.lux®](https://justgetflux.com/). However, as
of now, there is no way to programmatically configure Night Shift (without
entering the system preferences GUI).

This `nightshift` CLI aims to enable such access via a few simple commands.
(Or, alternatively, via library access for other Rust tools.)

## Installing

[Set up Rust/Cargo](https://doc.rust-lang.org/book/ch01-01-installation.html)
and install from crates.io by running:

```
cargo install nightshift
```

Or clone this repo and install from the local path:

```
cargo install --path .
```

## Usage

First, make sure you are running macOS Sierra or newer.

### Command-Line Interface

Turn Night Shift on (until tomorrow/sunrise):

```
nightshift on
```

Turn Night Shift off:

```
nightshift off
```

Set color temperature (a number from 0 to 100):

```
nightshift temp 70
```

### Rust API

In addition to a CLI, `nightshift` can be pulled-in as a dependency for other Rust crates:

```
nightshift = "0.0.2"
```

Here's an example `fn` that toggles Night Shift off,
changes the color temperature preference, and then
toggles the feature back on:

```rust
extern crate nightshift;

use nightshift::NightShift;

fn main() {
    let night_shift = NightShift::new();
    night_shift.off().unwrap();
    night_shift.set_temp(70).unwrap();
    night_shift.on().unwrap();
}
```

## Todo:

- [ ] Ability to see current status of Night Shift
- [ ] Ability to enable/disable sunrise/sundown schedule
- [ ] Ability to enable/disable custom schedules
- [ ] API improvements and full documentation
- [ ] Tests that use fake/stub ObjC library.
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

## License

`nightshift` is released under the [MIT License](LICENSE).
