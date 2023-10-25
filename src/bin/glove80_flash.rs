use std::{
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use anyhow::bail;
use itertools::Itertools;
use xshell::Shell;

/// Small struct to order paths by creation date.
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PathWithCreated {
    created: SystemTime,
    path: PathBuf,
}

/// Find the most recent firmware file.
///
/// For simplicity, we assume that all UF2 files are
/// for the Glove80.
fn most_recent_firmware(
    shell: &Shell,
    firmware_dir: &PathBuf,
) -> Result<Option<PathBuf>, anyhow::Error> {
    Ok(shell
        .read_dir(firmware_dir)?
        .into_iter()
        .filter(|path| matches!(path.extension(), Some(ext) if ext == OsStr::new("uf2")))
        .map(|path| {
            let created = path.metadata()?.created()?;
            Ok::<_, io::Error>(PathWithCreated { path, created })
        })
        .fold_ok(None, |prev, next| prev.max(Some(next)))?
        .map(|path_created| path_created.path))
}

/// Get system mounts path.
fn mounts_path() -> &'static Path {
    // TODO: add Linux mount paths.
    Path::new("/Volumes")
}

/// Wait until one of the given paths exist.
///
/// This function currently doesn't use any filesystem monitoring
/// API like inotify. This is a simple script replacement, so I
/// don't don't need it to be very robust. So this function simply
/// checks the paths once per second.
fn wait_until_exists<P: AsRef<Path>>(mount_path: &[P]) -> &Path {
    loop {
        for path in mount_path {
            let path = path.as_ref();
            if path.exists() {
                return path;
            }
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn main() -> anyhow::Result<()> {
    let flags = xflags::parse_or_exit!(
        /// Firmware download directory.
        required download_path: PathBuf
    );

    let shell = xshell::Shell::new()?;

    let firmware_path = match most_recent_firmware(&shell, &flags.download_path)? {
        Some(path) => path,
        None => bail!("No firmware found in `{}`", flags.download_path.display()),
    };
    eprintln!("Firmware: {}", firmware_path.display());

    let mounts_path = mounts_path();
    eprintln!(
        "Waiting for Glove volume to become available in `{}`...",
        mounts_path.display()
    );

    let glove80_paths = vec![
        mounts_path.join("GLV80LHBOOT"),
        mounts_path.join("GLV80RHBOOT"),
    ];

    let glove80_path = wait_until_exists(&glove80_paths);

    eprintln!(
        "Found `{}`, flashing...",
        glove80_path.display()
    );

    eprintln!("Flashing...");

    shell.copy_file(firmware_path, glove80_path)?;

    eprintln!("Done!");

    Ok(())
}
