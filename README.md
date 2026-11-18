⟹ New [Open Combat](https://github.com/buxx/OpenCombat) [bevy](https://bevy.org/) based engine. Will be merge into Open Combat repository when finished.

## Developer guide

See also [developer guide](doc/developer/developer.md)

## Setup

On your computer:

- Install Rust/Cargo ([rust-lang.org/tools/install/](rust-lang.org/tools/install/))
- Install Bevy dependencies ([https://bevy.org/learn/quick-start/getting-started/setup/](https://bevy.org/learn/quick-start/getting-started/setup/))
- Install "just" (https://just.systems/man/en/installation.html)[https://just.systems/man/en/installation.html] (optional)

## Run tests

To run all unit tests (warning: bins compilation is disk write/read intensive)

    cargo nextest run

To run all unit tests, package by package (less disk read/write intensive, but more longer)

    just test

To run end-to-end tests

    just test-e2e
