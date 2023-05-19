<h1 align="center">
  
  # ⌨️ Type Defender ⌨️
  [![CI][ci0]][ci1] [![DL][dl0]][dl1] [![crates][cr0]][cr1] ![MIT][li0] ![RV][rv0] 
 
</h1>

[ci0]: https://img.shields.io/github/actions/workflow/status/stephanmalan/type_defender/release.yml
[ci1]: https://github.com/StephanMalan/type_defender/actions
[dl0]: https://img.shields.io/endpoint?url=https://gist.githubusercontent.com/stephanmalan/415afdd86874b8620caa8e841546f4b3/raw/version.json
[dl1]: https://github.com/StephanMalan/type_defender/releases/latest
[cr0]: https://img.shields.io/badge/dynamic/json?color=success&label=crates.io&prefix=v&query=versions%5B0%5D.num&url=https%3A%2F%2Fcrates.io%2Fapi%2Fv1%2Fcrates%type_defender%2Fversions
[cr1]: https://crates.io/crates/type_defender
[li0]: https://img.shields.io/badge/license-MIT-blue.svg
[rv0]: https://img.shields.io/badge/rustc-1.71%2B-lightgrey.svg

![type_defender](https://user-images.githubusercontent.com/12143963/228680082-cd2495b5-306b-467e-8ad4-20d77af12fac.gif)

Type Defender is a Rust based, terminal game where the player needs to quickly type words as the appear on the screen.
The game was created to help improve your typing while having fun.

## Features

- Available for Afrikaans, English, and 한국어.
- Adaptive speed based on your typing.
- Works with Mac OS, Linux, and Windows.

## Technology

Type Defender is written in Rust using the [TUI](https://github.com/fdehau/tui-rs)/[Crossterm](https://github.com/crossterm-rs/crossterm) libraries.
