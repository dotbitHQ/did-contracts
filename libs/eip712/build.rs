use std::path::{Path, PathBuf};
use std::{env, fs, io, process};

fn main() {
    // This is the project directory when using das-types as the workspace member.
    let current_dir = env::current_dir().expect("The current directory is not available.");
    // This is the compiling directory when using das-types as a dependency.
    let pwd = PathBuf::from(&env::var("PWD").expect("$PWD is required to properly compiling this library."));

    let dotenv_path = match find(current_dir.as_path(), Path::new(".env")) {
        Ok(dotenv_path) => dotenv_path,
        Err(_) => match find(pwd.as_path(), Path::new(".env")) {
            Ok(dotenv_path) => dotenv_path,
            Err(err) => {
                println!("cargo:warning=❌ Loading .env file failed: {:?}", err);
                process::exit(1);
            }
        },
    };

    println!("cargo:rerun-if-changed={}", dotenv_path.as_path().display());

    match dotenvy::from_path_iter(dotenv_path.as_path()) {
        Ok(dotenv_iter) => {
            println!("cargo:warning=✅ {} loaded", dotenv_path.as_path().display());

            for env_var in dotenv_iter {
                match env_var {
                    Ok((key, value)) => {
                        println!("cargo:rustc-env={key}={value}");
                        // println!("cargo:warning=Set env var {key} = {value}");
                    }
                    Err(err) => {
                        println!("cargo:warning={:?}", err);
                        continue;
                    }
                };
            }
        }
        Err(err) => {
            println!(
                "cargo:warning=❌ Loading {} file failed: {:?}",
                dotenv_path.as_path().display(),
                err
            );
            process::exit(1);
        }
    }
}

/// Searches for `filename` in `directory` and parent directories until found or root is reached.
/// Copy and slightly modified from the dotenvy crate.
pub fn find(directory: &Path, filename: &Path) -> Result<PathBuf, io::Error> {
    let candidate = directory.join(filename);

    match fs::metadata(&candidate) {
        Ok(metadata) => {
            if metadata.is_file() {
                return Ok(candidate);
            }
        }
        Err(error) => {
            if error.kind() != io::ErrorKind::NotFound {
                return Err(error);
            }
        }
    }

    if let Some(parent) = directory.parent() {
        find(parent, filename)
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound, "path not found"))
    }
}
