use reqwest::blocking::get;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::{env, error, fs, io};

fn main() {
    // prevent running for docs.rs
    #[cfg(not(feature = "docsrs"))]
    {
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            panic!(
                "\x1b[31mError vzense-rust: Sorry, no libraries for this architecture. Libraries are only provided for x86_64 and aarch64.\x1b[0m"
            )
        }
        #[cfg(target_arch = "x86_64")]
        let arch = "x86_64";
        #[cfg(target_arch = "aarch64")]
        let arch = "aarch64";

        /*
        Shared libraries need to be within the target dir, see
        https://doc.rust-lang.org/cargo/reference/environment-variables.html#dynamic-library-paths

        Therefore, the library archive {arch}.tar.xz is first downloaded from github and then extracted in <vzense-rustPackageDir>/target/vzense_lib/. Finally, symlinks to the libraries are added to <whateverProjectDir>/target/<buildType>/deps/.
        */

        // <vzense-rustPackageDir>/target/vzense_lib/
        let lib_path = env::current_dir().unwrap().join("target/vzense-lib");

        // check if libraries have been extracted already
        if !existing(lib_path.join(arch)) {
            fs::create_dir_all(lib_path.clone()).unwrap();

            let lib_url = format!(
                "https://github.com/h-a-n-n-e-s/vzense-rust/raw/refs/heads/main/lib/{arch}.tar.xz"
            );

            download(&lib_url, lib_path.join(format!("{}{}", arch, ".tar.xz")))
                .expect("\x1b[31mError vzense-rust: Unable to download libraries.\x1b[0m");

            // decompress
            execute(
                "unxz",
                &[&format!("{arch}.tar.xz")],
                lib_path.clone(),
                "could not unxz vzense-lib",
            );

            // untar
            execute(
                "tar",
                &["-xf", &format!("{arch}.tar")],
                lib_path.clone(),
                "could not untar vzense-lib",
            );
        }

        // <whateverProjectDir>/target/<buildType>/deps/
        let deps_path = Path::new(&env::var("OUT_DIR").unwrap()).join("../../../deps");

        // create symlinks
        symlink_dir_all(lib_path.join(arch), deps_path.clone()).expect(&format!(
            "\x1b[31mfailed to create symlinks to libraries\x1b[0m"
        ));

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
}

fn existing(path: PathBuf) -> bool {
    path.try_exists()
        .expect(&format!("cannot check if {path:?} exists"))
}

fn execute(command: &str, args: &[&str], dir: PathBuf, err: &str) {
    std::process::Command::new(command)
        .current_dir(dir)
        .args(args)
        .output()
        .expect(err);
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
            if !existing(file.clone()) {
                std::os::unix::fs::symlink(entry.path(), file)?;
            }
        }
    }
    Ok(())
}

/// download from `url` and save as `file_name`
fn download(url: &str, file_name: PathBuf) -> Result<(), Box<dyn error::Error>> {
    let now = Instant::now();
    let response = get(url)?;
    let content = response.bytes()?;

    let mut downloaded_file = fs::File::create(file_name)?;
    downloaded_file.write_all(&content)?;

    let duration = now.elapsed();
    println!("Downloaded file {url} in {duration:?}");
    Ok(())
}
