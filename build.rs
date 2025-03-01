use reqwest::blocking::get;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::{error, fs, io};

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

        let lib_url =
            format!("https://github.com/h-a-n-n-e-s/vzense-rust/blob/main/lib/{arch}.tar.xz");

        // download(lib_url, "bernd.tar.xz").expect("cannot download file");

        /*
        Shared libraries need to be within the target dir, see
        https://doc.rust-lang.org/cargo/reference/environment-variables.html#dynamic-library-paths

        Therefore, libraries are first extracted from `vzense-lib.tar.xz` to `targst/vzense_lib/` and then symlinked from there to target/<buildType>/deps/.
        */

        let root = std::env::current_dir().unwrap();
        let lib_path = root.join("target/vzense-lib").join(arch);

        // check if libraries have been extracted already
        if !existing(lib_path.clone()) {
            fs::create_dir_all(lib_path.clone()).unwrap();

            download(
                &lib_url,
                lib_path.join(format!("{}{}", arch, ".tar.xz"))
            )
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

        let deps_path = std::env::current_exe().unwrap().join("../../../deps");

        // create symlinks
        symlink_dir_all(lib_path, deps_path.clone())
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
            // fs::copy(entry.path(), file)?;
            if !file.exists() {
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

    let mut downloaded_file = std::fs::File::create(file_name)?;
    downloaded_file.write_all(&content)?;

    let duration = now.elapsed();
    println!("Downloaded file {url} in {duration:?}");
    Ok(())
}
