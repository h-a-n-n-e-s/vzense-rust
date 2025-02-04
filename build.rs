use std::path::Path;
use std::{fs, io};

fn main() {
    // prevent running for docs.rs
    #[cfg(not(feature = "docsrs"))]
    {
        /*
        Shared libraries need to be within the target dir, see
        https://doc.rust-lang.org/cargo/reference/environment-variables.html#dynamic-library-paths

        Therefore, libraries are first extracted from `vzense-lib.tar.xz` to `targst/vzense_lib/` and then symlinked from there to target/<buildType>/deps/.
        */

        let lib_src = std::env::current_dir().unwrap().join("target/vzense-lib");

        // check if libraries have been extracted already
        if !lib_src.exists() {
            std::fs::copy("vzense-lib.tar.xz", "target/vzense-lib.tar.xz")
                .expect(&format!("could not copy vzense-lib {:?}", std::env::current_dir().unwrap()));

            // decompress the vzense-lib directory
            std::process::Command::new("unxz")
                .current_dir("target/")
                .arg("vzense-lib.tar.xz")
                .output()
                .expect("could not unxz vzense-lib");

            // untar
            std::process::Command::new("tar")
                .current_dir("target/")
                .arg("-xf")
                .arg("vzense-lib.tar")
                .output()
                .expect("could not untar vzense-lib");

            // rm tar
            std::process::Command::new("rm")
                .current_dir("target/")
                .arg("vzense-lib.tar")
                .output()
                .unwrap();
        }

        let mut deps_path = std::env::current_exe().unwrap();
        for _ in 0..3 {
            deps_path.pop(); // 3 x cd ..
        }
        deps_path.push("deps"); // cd deps

        // create symlinks
        symlink_dir_all(lib_src, deps_path.clone())
            .expect("failed to create symlinks to libraries");

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
}
