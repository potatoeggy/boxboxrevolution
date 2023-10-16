# Box Box Revolution

A punching rhythm game

## Setup

Pin layout is described in `src/bin/main.rs`. STLink, Cargo, and Rustup are assumed to be available on the user's system.

```bash
# install deps
cargo install probe-rs --features cli

# compile to arm
rustup target add thumbv7em-none-eabi
```

Run:

```bash
cargo rb main
```