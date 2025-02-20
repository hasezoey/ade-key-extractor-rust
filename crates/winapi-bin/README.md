# ade-extract-winapi-bin

This Binary is made to use [`winapi`](https://crates.io/crates/winapi) to call [`CryptUnprotectData`](https://learn.microsoft.com/en-us/windows/win32/api/dpapi/nf-dpapi-cryptunprotectdata), because there didnt exist a cli tool to call this function previously, so it could not be called outside of a dedicated binary.

This Binary can be used stand-alone without any of the other tools in this repository.

## Usage

All input's & output's are encoded in hex.

Running the binary only needs 2 parameters, the hex encoded `entropy` and the hex encoded `data`:

```sh
ade-extract-winapi-bin.exe "hex_entropy" "hex_data"
```

will output to stdout:

```txt
decrypted "hex_outdata"
```

## Building

MSVC is required to be installed:

```sh
# This Script assumes you are at the root of the git repository

# Add windows MSVC as a target
rustup target add x86_64-pc-windows-msvc

# Install the toolchain's downloader
cargo install xwin

# Install the toolchain into this project (to not have it globally)
xwin --accept-license splat --output ./.xwin
```

This Binary needs to be manually compiled with a specific target:

```sh
cargo build --target=x86_64-pc-windows-msvc --bin ade-extract-winapi-bin
```
