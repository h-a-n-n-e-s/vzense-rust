# Vzense ToF Camera Bindings for Rust

This library provides high-level bindings to connect to and read data from [Vzense](https://www.vzense.com) Time of Flight (ToF) Cameras. The underlying low-level FFI crate is `vzense-sys`.

### Architecture support

Support for `x86_64` and `aarch64` is provided.

### OS support

Currently only Linux is supported (for MacOS, Vzense unfortunately does not provide drivers). The library is based on the [Scepter SDK](https://github.com/ScepterSW/ScepterSDK) and [Vzense SDK](https://github.com/Vzense/Vzense_SDK_Linux) repositories. Tested for Debian 12.8 on x86-64 and [Asahi Linux](https://asahilinux.org/) on aarch64 (Apple M2).

### Camera support

The `scepter` module supports [NYX650/660](https://industry.goermicro.com/product/nyx-series), DS86/87, DS77C, and DS77 cameras. (Only NYX650 has been tested so far.)

The `dcam560` module is specifically for the Vzense DCAM560 camera but other models like DCAM305, DCAM550, and DCAM710 can be added upon request. (Only DCAM560 has been tested so far.)

## Example and Usage

The `scepter` module is used by default. To use the `dcam560` module, set `default = ["dcam560"]` under `[features]` in Cargo.toml.

The [basic](examples/basic.rs) example covers all the functionality provided by the library and can be run with `cargo run --example basic`. To stream with maximum frame rate add `--release`. For the example, the [`show-image`](https://docs.rs/show-image/latest/show_image) crate is used as a dev-dependency to display data.

For a standalone binary to find links (stored in `<projectDir>/target/<buildType>/deps/`) to the shared libraries, one can add that path to `LD_LIBRARY_PATH`. Or use [chrpath](https://linux.die.net/man/1/chrpath) but make sure that `rpath = true` is set under `[profile.<buildType>]` in Cargo.toml.

### Issues

There is an issue that data for the "color mapped to depth frame" is not available for the NYX650 camera if running with `--release`. Please see [here](https://users.rust-lang.org/t/raw-pointer-contains-no-data-when-running-in-release/122814/16) for details.

## License

This project is licensed under the terms of the [MIT and BSD-3-Clause License](LICENSE.txt).
