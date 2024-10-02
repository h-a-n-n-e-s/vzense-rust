fn main() {
    /*
    Shared libraries need to be within the target dir, see
    https://doc.rust-lang.org/cargo/reference/environment-variables.html#dynamic-library-paths

    Therefore, libraries are copied from vzense_lib/ to target/<buildType>/deps/. The following hack was adapted from
    https://gitlab.com/tangram-vision/oss/realsense-rust/-/blob/9d490f386e608f3dc24c89ea2cda8fe99dea1e87/realsense-sys/build.rs
    */
    let lib_src = std::env::current_dir().unwrap().join("vzense-lib");
    let lib_src2 = lib_src.clone();
    let mut lib_src_list = vec![lib_src, lib_src2];
    lib_src_list[0].push("libvzense_api.so");
    lib_src_list[1].push("libImgPreProcess.so");

    let mut deps_path = std::env::current_exe().unwrap();
    for _ in 0..3 {
        // 3 x cd ..
        deps_path.pop();
    }
    deps_path.push("deps"); // cd deps

    let lib_path = deps_path.clone();
    let deps_path2 = deps_path.clone();
    let mut lib_dest_list = vec![deps_path, deps_path2];
    lib_dest_list[0].push("libvzense_api.so");
    lib_dest_list[1].push("libImgPreProcess.so");

    for (src, dest) in std::iter::zip(lib_src_list.as_slice(), lib_dest_list) {
        match std::fs::copy(src, dest) {
            Ok(_) => println!("libvzense_api.so successfully copied to deps folder."),
            Err(e) => panic!("{}; attempting from source {:#?}", e, src),
        }
    }

    // tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search={}", lib_path.to_str().unwrap());

    // necessary for runtime to find shared library
    println!(
        "cargo:rustc-link-arg=-Wl,-rpath,{}",
        lib_path.to_str().unwrap()
    );

    // tell rustc to link the shared library
    println!("cargo:rustc-link-lib=vzense_api");
}
