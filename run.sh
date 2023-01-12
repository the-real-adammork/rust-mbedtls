#!/bin/bash

cargo build --package libmobilecoin --target aarch64-apple-ios-macabi --lib -Z avoid-dev-deps -Z unstable-options -Zbuild-std
