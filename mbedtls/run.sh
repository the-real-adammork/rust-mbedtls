#!/bin/bash

rustup component add rust-src --toolchain nightly-2022-04-29-aarch64-apple-darwin
cargo build --target aarch64-apple-ios-macabi --lib -Z avoid-dev-deps -Z unstable-options -Zbuild-std
