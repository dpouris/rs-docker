use anyhow::{anyhow, Context, Result};
use std::{
    env, fs,
    os::unix::fs::chroot,
    path::{Path, PathBuf},
    process,
};
mod registry;
use registry::*;

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    assert!(args.len() >= 5);
    let image = &args[2];
    let command = &args[3];
    let command_args = &args[4..];
    let command_path = PathBuf::from(command);

    let repo = Repository::new(image);
    let registry = Registry::authenticate_repo(repo).await?;
    registry.pull().await?;

    setup_root(&command_path)?;

    let output = process::Command::new(&command_path)
        .args(command_args)
        .output()
        .context(format!(
            "Tried to spawn new child process of command '{}' with arguments {:?}",
            command_path.display(),
            command_args
        ))?;

    match output.status.code() {
        Some(code) => {
            let std_out = std::str::from_utf8(&output.stdout)?;
            print!("{}", std_out);
            process::exit(code)
        }
        None => {
            let std_err = std::str::from_utf8(&output.stderr)?;
            eprint!("{}", std_err);
            process::exit(1)
        }
    }
}

fn create_dir(path: &PathBuf) -> Result<()> {
    if path.is_dir() {
        return Ok(());
    }
    fs::DirBuilder::new()
        .recursive(true)
        .create(path)
        .context(format!("Attempted to create directory {}", path.display()))?;
    Ok(())
}

fn create_file(path: &PathBuf) -> Result<()> {
    if path.is_file() {
        return Ok(());
    }
    if path.is_dir() {
        return Err(anyhow!("Provided path {} is a directory", path.display()));
    }
    create_dir(
        &path
            .parent()
            .expect("Path always has parent dir")
            .to_path_buf(),
    )?;

    fs::File::create(path)?;

    Ok(())
}

fn cp_from_path(src: &PathBuf, dst: &Path) -> Result<()> {
    let dst_bin = if src.is_absolute() {
        dst.join(src.strip_prefix("/").expect("destination is absolute path"))
    } else {
        dst.join(src)
    };

    if !dst_bin.is_file() {
        create_file(&dst_bin)?;
    }
    fs::copy(src, &dst_bin).context(format!(
        "Attempted to copy {} to {}",
        src.display(),
        dst_bin.display()
    ))?;

    Ok(())
}

fn setup_root(bin_src: &PathBuf) -> Result<()> {
    let root_dir = PathBuf::from("/tmp/root");
    let root_pathname = root_dir.display();

    // create home dir inside root
    create_dir(&root_dir.join("home/tmp"))?;
    // create dev/null inside root
    create_file(&root_dir.join("dev/null"))?;
    // // copy the executable inside root
    // cp_from_path(bin_src, &root_dir)?;

    chroot(&root_dir).context(format!(
        "Attempted to chroot into directory {root_pathname}"
    ))?;

    // Unshare pid namespace to isolate processes inside chroot
    unshare_pid()?;

    env::set_current_dir("/home/tmp")
        .context("Attempted to change cwd inside chroot to `/home/tmp`")?;
    Ok(())
}

fn unshare_pid() -> Result<()> {
    #[cfg(target_os = "linux")]
    unsafe {
        if libc::unshare(libc::CLONE_NEWPID) == 0 {
            return Ok(());
        }
        Err(anyhow!("Could not unshare pid"))
    }

    #[cfg(not(target_os = "linux"))]
    Err(anyhow!(
        "Cannot use unshare. OS not linux and doesn't support namespaces"
    ))
}
