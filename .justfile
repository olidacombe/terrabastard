set dotenv-load := false

# print options
default:
    @just --list --unsorted

# install cargo tools
init:
    cargo upgrade --incompatible
    cargo update
    cargo install cargo-rdme

# generate README
readme:
    cargo rdme --force

# format code
fmt:
    cargo fmt
    prettier --write .
    just --fmt --unstable

# check code
check:
    cargo check
    cargo clippy --all-targets --all-features

# build project
build:
    cargo build --all-targets

# execute tests
test:
    cargo test
