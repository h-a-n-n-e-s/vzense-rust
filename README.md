# Vzense ToF Camera Bindings for Rust

This library provides high-level bindings to connect to and read data from [Vzense](https://www.vzense.com) Time of Flight (ToF) Cameras. The underlying low-level FFI crate `vzense-sys` was created with [bindgen](https://docs.rs/bindgen/latest/bindgen).

## OS support

Currently only Linux is supported (for MacOS, Vzense unfortunately does not provide drivers). The library is based on the [Scepter SDK](https://github.com/ScepterSW/ScepterSDK) and [Vzense SDK](https://github.com/Vzense/Vzense_SDK_Linux) repositories and was tested on Debian 12.8, but should also work on other distros based on it, like Ubuntu. Support for other platforms (e.g., AArch64) is possible using the respective headers and libraries from the repositories and rebuilding the bindings (support can be added upon request).

## Camera support

The `scepter` module supports [NYX650/660](https://industry.goermicro.com/product/nyx-series), DS86/87, DS77C, and DS77 cameras. (Only NYX650 has been tested so far.)

The `dcam560` module is specifically for the Vzense DCAM560 camera but other models like DCAM305, DCAM550, and DCAM710 can be added upon request. (Only DCAM560 has been tested so far.)

## Example and Usage

The `scepter` module is used by default. To use the `dcam560` module, set `default = ["dcam560"]` under `[features]` in Cargo.toml.

The [basic](examples/basic.rs) example covers all the functionality provided by the library and can be run with `cargo run --example basic`. To stream with maximum frame rate add `--release`.

The [`show-image`](https://docs.rs/show-image/latest/show_image) crate is useds to display the data.  

If an executable is used on a machine it was not built on, make sure it can find the shared libraries in `vzense-lib/`.

**Note**: There is an issue that data for the "color mapped to depth frame" is not available for the NYX650 camera if running with `--release`. Please see [here](https://users.rust-lang.org/t/raw-pointer-contains-no-data-when-running-in-release/122814/16) for details.

## License

This project is licensed under the terms of the [MIT and BSD-3-Clause License](LICENSE.txt).
