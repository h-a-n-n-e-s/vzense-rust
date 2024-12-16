use std::path::Path;
use std::{io, fs};

fn main() {
    /*
    Shared libraries need to be within the target dir, see
    https://doc.rust-lang.org/cargo/reference/environment-variables.html#dynamic-library-paths

    Therefore, libraries are copied from vzense_lib/ to target/<buildType>/deps/. The following hack was adapted from
    https://gitlab.com/tangram-vision/oss/realsense-rust/-/blob/9d490f386e608f3dc24c89ea2cda8fe99dea1e87/realsense-sys/build.rs
    */

    // list of libraries
    let libs = [
        "libvzense_api.so",
        "libImgPreProcess.so",
        "libScepter_api.so",
        "libDSImgPreProcess.so",
    ];

    let lib_src = std::env::current_dir().unwrap().join("vzense-lib");
    let mut lib_src_list = vec![];
    for (i, l) in libs.iter().enumerate() {
        lib_src_list.push(lib_src.clone());
        lib_src_list[i].push(l);
    }

    let mut deps_path = std::env::current_exe().unwrap();
    for _ in 0..3 {
        // 3 x cd ..
        deps_path.pop();
    }
    deps_path.push("deps"); // cd deps

    let lib_path = deps_path.clone();
    let mut lib_dest_list = vec![];
    for (i, l) in libs.iter().enumerate() {
        lib_dest_list.push(deps_path.clone());
        lib_dest_list[i].push(l);
    }

    // for (src, dest) in std::iter::zip(lib_src_list.as_slice(), lib_dest_list) {
    //     match fs::copy(src, dest) {
    //         Ok(_) => println!("libvzense_api.so successfully copied to deps folder."),
    //         Err(e) => panic!("{}; attempting from source {:#?}", e, src),
    //     }
    // }

    copy_dir_all(lib_src, deps_path).expect("failed to copy libraries");

    // tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search={}", lib_path.to_str().unwrap());

    // necessary for runtime to find shared library
    println!(
        "cargo:rustc-link-arg=-Wl,-rpath,{}",
        lib_path.to_str().unwrap()
    );

    // tell rustc to link the shared library
    println!("cargo:rustc-link-lib=vzense_api");
    println!("cargo:rustc-link-lib=Scepter_api");
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}