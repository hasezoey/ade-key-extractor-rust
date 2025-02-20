#!/bin/sh

# this file is an shorthand for the command below
cargo clippy --all-features "$@" --
# the following options have been transferred to /Cargo.toml#workspace.lints.clippy
