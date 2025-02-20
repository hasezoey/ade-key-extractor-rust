# ade-extract-key

This binary is the main entry-point of this repository & project, for full "quick" usage see the [root README](../../README.md).

## Usage

This binary aims to run everything automatically, but if the winapi stage fails, it can be executed manually and then resume with the output from the winapi stage:

```sh
# Execute until it fails or completely finishes
./ade-extract-key
```

If the `winapi` stage fails, it might look like this:

```sh
# Execute the program
$ ./ade-extract-key
Entropy (hex): "some_entropy_hex"
Device-Key (hex): "some_device_hex"
Adept-Key (base64): "some_adept_base64"

Some Error

# Manually execute the winapi stage
# The parameters to this stage are provided by the above output
$ wine ./ade-extract-winapi-bin.exe "some_entropy_hex" "some_device_hex"
decrypted "some_decrypted_key"

# Finally, the main binary can be resumed with the decrypted key
# By using the subcommand "aes" all the previous steps are skipped
$ ./ade-extract-key aes "some_decrypted_key" "some_adept_base64"
Wrote key to ./ade_key.der
```

Note that alternatively, you can also build this binary for the windows target directly and execute everything in wine (also should work on windows directly).
NOTE: the full windows version of `ade-extract-key` [requires wine 9 or later](https://github.com/rust-lang/rust/issues/128066).

If you dont use the default wineprefix, then `WINEPREFIX` needs to be set to the correct prefix for all the commands shown above.

`--help` Output:

```txt
ADE key extractor for DeDRM

Usage: ade-extract-key [OPTIONS] [OUTPUT_FILE_NAME]
       ade-extract-key [OPTIONS] [OUTPUT_FILE_NAME] <COMMAND>

Commands:
  aes  Resume at the AES decryption stage with the winapi decrypted key

Arguments:
  [OUTPUT_FILE_NAME]  Change output file name / directory

Options:
  -v, --verbosity...  Set Loggin verbosity (0 - Default - WARN, 1 - INFO, 2 - DEBUG, 3 - TRACE)
  -h, --help          Print help
  -V, --version       Print version
```
