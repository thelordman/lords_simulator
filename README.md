# Lord's Simulator

[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

GUI-based life simulator built with Rust, focusing on realism and limitless possibilities.

Runs on Linux, Windows, and macOS.

## Download

Packages for all three systems are attached to each
[release](https://github.com/thelordman/lords_simulator/releases).

- **Windows:** unzip and run `lords_simulator.exe`.
- **Linux:** unpack the `tar.gz` and run `lords_simulator`.
- **macOS:** unzip `Lords-Simulator-…-macos-universal.zip` and open the app. The `tar.gz` holds the same program.
  The app is not notarized, so macOS refuses to open it the first time; allow it under
  System Settings → Privacy & Security.

## Building

With a stable [Rust toolchain](https://rustup.rs) installed (1.95 or newer), on any of the three
systems:

```sh
cargo run --release
```

Linux needs a C linker.

The workspace is four crates: `lords_sim` is the simulation itself, `lords_runtime` runs it in
real time on its own thread, `lords_save` reads and writes saved games, and `lords_gui` is the
interface. `cargo test --workspace` runs the tests of all four.
