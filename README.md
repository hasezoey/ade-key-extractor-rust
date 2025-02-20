# ADE key extractor rust

DeDRM's Adobe (2.5) key extractor re-written in rust, with the goal being to not require anything else installed to extract the key.

## Motivation

This Project was created as a test to see if i could convert the [Python Script](https://github.com/noDRM/DeDRM_tools/blob/master/DeDRM_plugin/adobekey.py) to a rust project, because i couldnt install to install python 3 in wine ([which since i have found the reason why](https://bugs.winehq.org/show_bug.cgi?id=54592)) and in python 2 wouldnt let me install pycrypto.

## Requirements

Nothing extra than a base linux system (at least when running, not building)

## Usage

To use this project 2 things are required:

- The Linux binary (`ade-extract-key`)
- The Windows binary (`ade-extract-winapi-bin.exe`)

Those binaries are available pre-compiled in the [Github Releases Page](https://github.com/hasezoey/ade-key-extractor-rust/releases) as `binaries.tar.gz`.

The following expects Adobe Digital Editions (2.5) to already be set-up and authorized:

```sh
# Execute the program
$ ./ade-extract-key
Entropy (hex): "some_entropy_hex"
Device-Key (hex): "some_device_hex"
Adept-Key (base64): "some_adept_base64"

Wrote key to ./ade_key.der

# If successful, the key should be in the file mentioned above
cat ./ade_key.der
```

Or if `ade-extract-key` is not successfull but still printed 3 values, then `ade-extract-winapi-bin.exe` can be run separately to transform the key, and then requires another run of `ade-extract-key` to get the final key.

```sh
# Execute the program
# May fail, the printed information can be used
$ ./ade-extract-key
Entropy (hex): "some_entropy_hex"
Device-Key (hex): "some_device_hex"
Adept-Key (base64): "some_adept_base64"

# Manually call the winapi-bin with the information
# this program only prints the final key, not saved to disk
$ wine ./ade-extract-winapi-bin.exe "some_entropy_hex" "some_device_hex"
decrypted "some_decrypted_key"

# Run the binary again, but only the last step to get the actual key
$ ./ade-extract-key aes "some_decrypted_key" "some_adept_base64"
Wrote key to ./ade_key.der

# If successful, the key should be in the file mentioned above
cat ./ade_key.der
```

Note that alternatively, you can also build this binary for the windows target directly and execute everything in wine (also should work on windows directly).
Those windows-only binaries are available as `win-binaries.tar.gz` in [Github Releases Page](https://github.com/hasezoey/ade-key-extractor-rust/releases).
NOTE: the full windows version of `ade-extract-key` [requires wine 9 or later](https://github.com/rust-lang/rust/issues/128066).

If you dont use the default wineprefix, then `WINEPREFIX` needs to be set to the correct prefix for all the commands shown above.

## Building for Linux & wine

Building this way requires both the linux target and a windows target:

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
cargo build --release
# Compile the windows binary
cargo build --release --target=x86_64-pc-windows-msvc --bin ade-extract-winapi-bin

# Copy output files into a out directory
mkdir ./final-bin
cp ./target/release/ade-extract-key ./final-bin/
cp ./target/x86_64-pc-windows-msvc/release/ade-extract-winapi-bin.exe ./final-bin/
```

For usage of the final binaries (in `./final-bin`), see [Usage](#usage).

## Building for wine

This way, only one target is build and everything can be directly executed within wine / windows, without requiring some parts in linux and some in wine:

```sh
# Add windows target
rustup target add x86_64-pc-windows-msvc
# Add the linux target too if you are using wine / building on linux
rustup target add x86_64-unknown-linux-gnu

# Install xwin, to manage msvc install
cargo install xwin

# Run xwin to install msvc
xwin --accept-license splat --output ./.xwin

# Compile the whole project as windows binaries
cargo build --release --target=x86_64-pc-windows-msvc

# Copy output files into a out directory
mkdir ./final-bin
cp ./target/x86_64-pc-windows-msvc/release/ade-extract-key.exe ./final-bin/
cp ./target/x86_64-pc-windows-msvc/release/ade-extract-winapi-bin.exe ./final-bin/
```

## Known Issues

### Wine does not let the windows binary run

If wine crashed with the following error, and the script / binary is located on a `ihc` filesystem, try again on a different filesystem, see [this wine bug](https://bugs.winehq.org/show_bug.cgi?id=54592)

```txt
wine: Unhandled page fault ...
```

## Working on this Project

This project requires:
- NodeJS with `yarn` installed (when working on an main branch)
- Rust install with `rustfmt`(nightly) & `clippy`, see [`fmt.sh`](./fmt.sh) & [`rustfmt.toml`](./rustfmt.toml)
