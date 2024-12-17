use anyhow::{bail, Context, Result};
use log::{debug, error, info, LevelFilter};
use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify, InotifyEvent};
use signal_hook::{
    consts::{SIGINT, SIGTERM},
    iterator::Signals,
};
use std::{fs, path::Path, path::PathBuf, thread};
use systemd_journal_logger::JournalLog;
use walkdir::WalkDir;

use clap::Parser;

/// Manage the kernel-gpu-2404 contents
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// The sentinel path to monitor
    #[arg(env = "COMPONENT_SENTINEL_PATH")]
    sentinel_path: PathBuf,
    /// The target path to manage content in
    #[arg(env = "COMPONENT_TARGET")]
    target: PathBuf,
    /// Turn on debug logging
    #[arg(env, long, short)]
    debug: bool,
}

/// Remove the sentinel file from target, then clear the target directory
fn cleanup(target: &Path, sentinel_name: &str) -> Result<()> {
    info!(target: "files", "cleaning up {target:?}");
    let sentinel_path = target.join(sentinel_name);
    if sentinel_path.exists() {
        fs::remove_file(&sentinel_path)
            .with_context(|| format!("Failed to remove {:?}", &sentinel_path))?;
        debug!(target: "files", "removed sentinel ({:?})", &sentinel_path);
    }
    for entry in fs::read_dir(target).context("Failed to list target directory")? {
        let path = entry.context("Failed to resolve entry")?.path();
        if path.is_dir() {
            fs::remove_dir_all(&path).with_context(|| format!("Failed to remove {:?}", &path))?;
            debug!(target: "files", "removed {:?} recursively", &path);
        } else {
            fs::remove_file(&path).with_context(|| format!("Failed to remove {:?}", &path))?;
            debug!(target: "files", "removed {:?}", &path);
        }
    }

    Ok(())
}

/// Check if the sentinel file is current, else clean the target directory and populate again,
/// sentinel file being last
fn populate(source: &Path, sentinel_name: &str, target: &Path) -> Result<()> {
    info!(target: "files", "populating {target:?} from {source:?} with sentinel {sentinel_name:?}");
    let sentinel_src = source.join(sentinel_name);

    if !sentinel_src.exists() {
        info!(target: "files", "sentinel file ({:?}) not found, skipping", &sentinel_src);
        return Ok(());
    }

    let sentinel_data = match fs::read(&sentinel_src) {
        Ok(d) if d.trim_ascii() == b"" => {
            error!(target: "files", "found empty sentinel file ({:?})", &sentinel_src);
            return Ok(());
        }
        Ok(d) => d,
        Err(e) => {
            error!(target: "files", "failed to read sentinel file: {e}");
            return Ok(());
        }
    };

    let sentinel_tgt = target.join(sentinel_name);
    if fs::read(&sentinel_tgt).is_ok_and(|content| content == sentinel_data) {
        info!(target: "files", "found current sentinel, skipping");
        return Ok(());
    }

    cleanup(target, sentinel_name).context("Cleanup failed")?;
    info!(target: "files", "copying from {source:?} to {target:?}");
    for entry in WalkDir::new(source).min_depth(1) {
        let entry = entry.context("Failed to resolve entry")?;
        let path = entry.path();
        let relative_path = path
            .strip_prefix(source)
            .context("Failed to strip relative path from entry")?;
        if relative_path.as_os_str() == sentinel_name {
            continue;
        }

        let target_path = target.join(relative_path);
        let meta = fs::symlink_metadata(&path)
            .with_context(|| format!("Failed to read file metadata for {:?}", &path))?;

        if meta.is_symlink() {
            let link_target = std::fs::read_link(&path)
                .with_context(|| format!("Failed to read link at {:?}", &path))?;
            std::os::unix::fs::symlink(link_target, &target_path)
                .with_context(|| format!("Failed to create symlink at {:?}", target_path))?;
        } else if meta.is_dir() {
            fs::create_dir(&target_path)
                .with_context(|| format!("Failed to create {:?}", &target_path))?;
            debug!(target: "files", "created {:?}", &target_path)
        } else {
            fs::copy(path, &target_path)
                .with_context(|| format!("Failed to copy {:?} to {:?}", &path, &target_path))?;
            debug!(target: "files", "copied {:?} to {:?}", &path, &target_path);
        }
    }

    fs::copy(&sentinel_src, &sentinel_tgt).context("Failed to copy sentinel")?;
    debug!(target: "files", "copied sentinel ({:?} to {:?})", sentinel_src, sentinel_tgt);

    Ok(())
}

/// Handle inotify events:
/// - run cleanup on sentinel deletion
/// - populate on sentinel written
/// - panic on source remove
fn monitor(inotify: &Inotify, source: &Path, sentinel_name: &str, target: &Path) -> Result<()> {
    info!(target: "inotify", "starting event monitoring on {:?}", &source);
    loop {
        let events = inotify
            .read_events()
            .context("Error reading Inotify events")?;

        for InotifyEvent { name, mask, .. } in events {
            debug!(
                target: "inotify",
                "handling {:?} event for {:?}",
                mask,
                name
            );

            if name.is_some_and(|n| n == sentinel_name) {
                match mask {
                    AddWatchFlags::IN_DELETE => cleanup(&target, &sentinel_name)?,
                    AddWatchFlags::IN_CLOSE_WRITE => populate(&source, &sentinel_name, &target)?,
                    _ => bail!("Unexpected Inotify event: {:?} on the sentinel file", mask),
                }
            } else if mask.intersects(AddWatchFlags::IN_DELETE_SELF | AddWatchFlags::IN_MOVE_SELF) {
                bail!("Monitored folder disappeared");
            }
        }
    }
}

/// Set up source directory monitoring, populate the target directory and start monitoring
/// the source for sentinel changes
fn main() -> Result<()> {
    let args = Cli::parse();
    let mut signals = Signals::new([SIGINT, SIGTERM]).context("Failed to set up signals")?;
    let inotify = Inotify::init(InitFlags::IN_CLOEXEC).context("Failed to set up inotify")?;

    let source = match args.sentinel_path.parent() {
        Some(p) if p.is_dir() => p,
        _ => bail!(
            "Sentinel ({:?})'s parent is not a directory",
            args.sentinel_path
        ),
    };

    let sentinel_name = args
        .sentinel_path
        .file_name()
        .context("Sentinel name could not be determined")?
        .to_str()
        .context("Sentinel name could not be read as a string")?;

    if !args.target.is_dir() {
        bail!("Target ({:?}) is not a directory", args.target);
    }

    thread::spawn(move || {
        signals.forever().next().is_some_and(|s| {
            info!("{:?} received, shutting down", s);
            std::process::exit(0);
        })
    });

    inotify
        .add_watch(
            source,
            AddWatchFlags::IN_CLOSE_WRITE
                | AddWatchFlags::IN_DELETE
                | AddWatchFlags::IN_MOVE_SELF
                | AddWatchFlags::IN_DELETE_SELF,
        )
        .context("Failed to add inotify watcher")?;

    JournalLog::new()
        .context("Failed to create Journal Logger")?
        .install()
        .context("Failed to install Journal Logger")?;

    let log_level = if args.debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    log::set_max_level(log_level);

    populate(&source.to_path_buf(), &sentinel_name, &args.target)
        .context("Initial populating failed")?;
    monitor(
        &inotify,
        &source.to_path_buf(),
        &sentinel_name,
        &args.target,
    )
    .context("Sentinel monitoring failed")?;

    Ok(())
}
