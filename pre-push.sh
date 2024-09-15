#!/bin/sh -e

cd aeron-archiver-codecs-build
sbe-cli schema generate --file schema/messages.xml --language rust --output-dir ..
cd ..
cargo +nightly fmt -- --emit files
cargo clippy --all-targets --all-features --workspace -- -D warnings #-W clippy::nursery
# TODO: change --lib to --all-targets for the integration tests to run
cargo test --all-features --lib -- --test-threads=1
