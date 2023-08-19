# print options
default:
    @just --list --unsorted

# install cargo tools
init:
    cargo upgrade --incompatible
    cargo update
    cargo install cargo-readme

# generate README
readme:
    cargo readme > README.md

# format code
fmt:
    cargo fmt --all -- --check
    prettier --write .
    just --fmt --unstable

# check code
check:
    cargo check
    cargo clippy --features "default" -- -D warnings

# build project
build:
    cargo build --all-targets

# execute tests
test:
    cargo test run --all-targets

# execute benchmarks
bench:
    cargo bench

