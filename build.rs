use std::env;

fn main() {
    let current_dir = env::current_dir().unwrap();

    // tell cargo to look for shared libraries in the specified directory
    println!(
        "cargo:rustc-link-search={}/vzense-sys/vzense/lib/",
        current_dir.to_str().unwrap()
    );

    // necessary for runtime to find shared library
    println!(
        "cargo:rustc-link-arg=-Wl,-rpath,{}/vzense-sys/vzense/lib/",
        current_dir.to_str().unwrap()
    );

    // tell rustc to link the shared library
    println!("cargo:rustc-link-lib=vzense_api");
  }