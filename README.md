# ADE key extractor rust

DeDRM's Adobe (2.5) key extractor re-written in rust, with the goal being to not require anything else installed to extract the key.

## Motivation

This Project was created as a test to see if i could convert the [Python Script](https://github.com/noDRM/DeDRM_tools/blob/master/DeDRM_plugin/adobekey.py) to a rust project, because wine somehow didnt allow me to install python 3 and in python 2 wouldnt let me install pycrypto.

## Requirements

Currently package `cpuid` is required.

## Usage

To use this project 2 things are required:

- The Linux binary (`ade-extract-key`)
- The Windows binary (`ade-extract-winapi-bin.exe`)

The following expects Adobe Digital Editions (2.5) to already be set-up and authorized:

```sh
# Execute the program
./ade-extract-key

# If successful, the key should be in
cat ./ade_key.der
```

## Building

Building this project requires both the linux target and a windows target:

```sh
# Add linux target
rustup target add x86_64-unknown-linux-gnu
# Add windows target
rustup target add x86_64-pc-windows-msvc

# Install xwin, to manage msvc install
cargo install xwin

# Run xwin to install msvc
xwin --accept-license splat --output ./.xwin

# Compile the project for linux
cargo build
# Compile the windows binary
cargo build --target=x86_64-pc-windows-msvc --bin ade-extract-winapi-bin

# Copy output files into a out directory
mkdir ./final-bin
cp ./target/debug/ade-extract-key ./final-bin/
cp ./target/x86_64-pc-windows-msvc/debug/ade-extract-winapi-bin.exe ./final-bin
```

For usage of the final binaries (in `./final-bin`), see [Usage](#usage).

## Known Issues

### Wine does not let the windows binary run

Currently it is known that wine out of some reason does not wanna run the windows binary, there may be a package missing, but i have not been able to figure out what is missing.

Example Errors include:

```txt
wine: Unhandled page fault ...
```

## Working on this Project

This project requires:
- NodeJS with `yarn` installed (when working on an main branch)
- Rust install with `rustfmt` & `clippy` (nightly version of mentioned components), see [`fmt.sh`](./fmt.sh) and [`clippy.sh`](./clippy.sh)
