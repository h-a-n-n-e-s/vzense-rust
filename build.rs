use std::path::Path;
use std::{fs, io};

fn main() {
    /*
    Shared libraries need to be within the target dir, see
    https://doc.rust-lang.org/cargo/reference/environment-variables.html#dynamic-library-paths

    Therefore, libraries are symlinked from vzense_lib/ to target/<buildType>/deps/.
    */

    let lib_src = std::env::current_dir().unwrap().join("vzense-lib");

    let mut deps_path = std::env::current_exe().unwrap();
    for _ in 0..3 {
        deps_path.pop(); // 3 x cd ..
    }
    deps_path.push("deps"); // cd deps

    // create symlinks
    symlink_dir_all(lib_src, deps_path.clone()).expect("failed to create symlinks to libraries");

    // tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search={}", deps_path.to_str().unwrap());

    // necessary for runtime to find shared libraries
    println!(
        "cargo:rustc-link-arg=-Wl,-rpath,{}",
        deps_path.to_str().unwrap()
    );

    // tell rustc to link the shared libraries
    println!("cargo:rustc-link-lib=vzense_api");
    println!("cargo:rustc-link-lib=Scepter_api");
}

/// create symlinks recursively to whole directory
fn symlink_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file = dst.as_ref().join(entry.file_name());
        let ty = entry.file_type()?;
        if ty.is_dir() {
            symlink_dir_all(entry.path(), file)?;
        } else {
            // fs::copy(entry.path(), file)?;
            if !file.exists() {
                std::os::unix::fs::symlink(entry.path(), file)?;
            }
        }
    }
    Ok(())
}
