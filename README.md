[![Build Status](https://github.com/RatScanner/RatUpdater/workflows/test/badge.svg)](https://github.com/RatScanner/RatUpdater/actions)
[![dependency status](https://deps.rs/repo/github/RatScanner/RatUpdater/status.svg)](https://deps.rs/repo/github/RatScanner/RatUpdater)
[![Lines Of Code](https://tokei.rs/b1/github/RatScanner/RatUpdater?category=code)](https://github.com/RatScanner/RatUpdater)

# Rat Updater

## Build

```
cargo build
# or
cargo build --release
```

## Test

```
cargo test
```

## Run

> ⚠️ The updater will clear the `root-path` (which defaults to `.`) before installing the new version.

```
cargo run -- --root-path ./tmp --update --start
```
