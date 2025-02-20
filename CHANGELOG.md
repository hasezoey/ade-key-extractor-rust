# Changelog

## 0.3.0

- feat: allow building on / for windows only. Also provide windows-only binaries
- feat: update some dependencies
- feat: dont rely on linux's `lscpu` for vendor-id anymore
- fix: add some more specific errors
- fix: change some boiler-plate documentation that was still accidentally included
- style: set (& build) MSRV to 1.75
- chore: build both linux and windows targets in the CI

## 0.2.0

- feat: use "__cpuid" and not require "cpuid" package anymore (though use as fallback)
- fix: trim return value when "echo" is used for user-name
- fix: actually use the correct regex for winapi-bin captures
- style: fix clippy

## 0.1.0

Initial release
