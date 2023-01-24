#!/bin/bash

#export RUST_BACKTRACE=full
rm -rf ../target

rustup component add rust-src --toolchain nightly-2022-04-29-aarch64-apple-darwin

# catalyst targets
#
cargo build --target aarch64-apple-ios-macabi --lib -Z avoid-dev-deps -Z unstable-options -Zbuild-std
cargo build --target x86_64-apple-ios-macabi --lib -Z avoid-dev-deps -Z unstable-options -Zbuild-std

# other libmobilecoin targets
#
cargo build --target aarch64-apple-ios --lib -Z avoid-dev-deps -Z unstable-options -Zbuild-std
cargo build --target aarch64-apple-ios-sim --lib -Z avoid-dev-deps -Z unstable-options -Zbuild-std
cargo build --target x86_64-apple-ios --lib -Z avoid-dev-deps -Z unstable-options -Zbuild-std
