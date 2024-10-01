# Vzense ToF Camera Bindings for Rust

This library provides high-level bindings to connect to and read data from [Vzense](https://www.vzense.com) Time of Flight (ToF) Cameras. The underlying low-level FFI crate `vzense-sys` was created with [bindgen](https://docs.rs/bindgen/latest/bindgen).

## OS support

Currently only Linux is supported (got no Windows machine to test; for MacOS, Vzense unfortunately does not provide drivers). The library is based on the [Vzense SDK Linux](https://github.com/Vzense/Vzense_SDK_Linux) repository and was tested on Debian 12.7, but should also work on other distros based on it, like Ubuntu. Support for other distros (e.g., Arch) is possible using the respective headers and libraries from the repository and rebuilding the bindings (support can be added upon request).

## Camera support

Currently only the Vzense DCAM560 is supported (the only one that could be tested), but other models like DCAM305, DCAM550, and DCAM710 can be added upon request.

## Example and Usage

The [basic](examples/basic.rs) example covers all the functionality provided by the library and can be run with `cargo run --example basic`. To stream with maximum frame rate add `--release`. The [`show-image`](https://docs.rs/show-image/latest/show_image) crate is necessary to display the data.  

If an executable is used on a machine it was not built on, make sure it can find the shared libraries stored in `./vzense-lib/`.

## Contributing

Any suggestions to improve this library are welcome. There are certainly many features that should be added if time permits.

## License

This project is licensed under the terms of the [MIT license](LICENSE.txt).
