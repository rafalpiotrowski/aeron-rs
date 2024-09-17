#!/bin/sh -e

cargo +nightly fmt -- --emit files
cargo clippy --all-targets --all-features --workspace -- -D warnings #-W clippy::nursery
