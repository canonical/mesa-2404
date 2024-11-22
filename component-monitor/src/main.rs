use anyhow::{bail, Context, Result};
use log::{debug, error, info, warn, LevelFilter};
use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify, InotifyEvent};
use signal_hook::{
    consts::{SIGINT, SIGTERM},
    iterator::Signals,
};
use std::{fs, path::PathBuf, thread};
use systemd_journal_logger::JournalLog;
use walkdir::WalkDir;

use clap::Parser;

/// Manage the kernel-gpu-2404 contents
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// The source path to monitor
    #[arg(env = "COMPONENT_SOURCE")]
    source: PathBuf,
    /// The sentinel file name
    #[arg(env = "COMPONENT_SENTINEL")]
    sentinel: PathBuf,
    /// The target path to manage content in
    #[arg(env = "COMPONENT_TARGET")]
    target: PathBuf,
    /// Turn on debug logging
    #[arg(env, long, short)]
    debug: bool,
}

/// Remove the sentinel file from target, then clear the target directory
fn cleanup(target: &PathBuf, sentinel: &PathBuf) -> Result<()> {
    info!(target: "files", "cleaning up {target:?}");
    let sentinel_path = target.join(sentinel);
    if sentinel_path.exists() {
        if let Err(err) = fs::remove_file(&sentinel_path) {
            warn!(target: "files", "failed to remove {0:?}: {err}", &sentinel_path)
        } else {
            debug!(target: "files", "removed sentinel ({0:?})", &sentinel_path)
        }
    }
    for entry in fs::read_dir(target).context("failed to list target directory")? {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                error!(target: "files", "{e}");
                continue;
            }
        };

        let path = entry.path();
        if path.is_dir() {
            if let Err(err) = fs::remove_dir_all(&path) {
                error!(target: "files", "failed to remove {0:?}: {err}", &path)
            } else {
                debug!(target: "files", "removed {0:?} recursively", &path)
            }
        } else {
            if let Err(err) = fs::remove_file(&path) {
                error!(target: "files", "failed to remove {0:?}: {err}", &path)
            } else {
                debug!(target: "files", "removed {0:?}", &path)
            }
        }
    }

    Ok(())
}

/// Check if the sentinel file is current, else clean the target directory and populate again,
/// sentinel file being last
fn populate(source: &PathBuf, sentinel: &PathBuf, target: &PathBuf) -> Result<()> {
    info!(target: "files", "populating {target:?} from {source:?} with sentinel {sentinel:?}");
    let sentinel_src = source.join(sentinel);

    if !sentinel_src.exists() {
        info!(target: "files", "sentinel file ({0:?}) not found, skipping", &sentinel_src);
        return Ok(());
    }

    let sentinel_data = match fs::read(&sentinel_src) {
        Ok(d) if d.trim_ascii() == b"" => {
            error!(target: "files", "found empty sentinel file ({0:?})", &sentinel_src);
            return Ok(());
        }
        Ok(d) => d,
        Err(e) => {
            error!(target: "files", "failed to read sentinel file: {e}");
            return Ok(());
        }
    };

    let sentinel_tgt = target.join(sentinel);
    if fs::read(&sentinel_tgt).is_ok_and(|content| content == sentinel_data) {
        info!(target: "files", "found current sentinel, skipping");
        return Ok(());
    }

    cleanup(target, sentinel)?;
    info!(target: "files", "copying from {source:?} to {target:?}");
    for entry in WalkDir::new(source) {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                error!(target: "files", "{e}");
                continue;
            }
        };

        let path = entry.path();
        let relative_path = path.strip_prefix(source)?;
        if path == source {
            continue;
        }
        if relative_path == sentinel {
            continue;
        }

        let target_path = target.join(relative_path);
        if path.is_dir() {
            if let Err(err) = fs::create_dir(&target_path) {
                error!(target: "files", "failed to create {0:?}: {err}", &target_path);
            } else {
                debug!(target: "files", "created {0:?}", &target_path)
            }
        } else {
            if let Err(err) = fs::copy(path, &target_path) {
                error!(target: "files", "failed to copy {0:?} to {1:?}: {err}", &path, &target_path);
            } else {
                debug!(target: "files", "copied {0:?} to {1:?}", &path, &target_path);
            }
        }
    }

    if let Err(err) = fs::copy(&sentinel_src, &sentinel_tgt) {
        error!(target: "files", "failed to copy sentinel: {err}");
    } else {
        debug!(target: "files", "copied sentinel ({0:?} to {0:?})", sentinel_src);
    }

    Ok(())
}

/// Handle inotify events:
/// - run cleanup on sentinel deletion
/// - populate on sentinel written
/// - panic on source remove
fn monitor(inotify: &Inotify, args: &Cli) -> Result<()> {
    info!(target: "inotify", "starting event monitoring on {0:?}", &args.source);
    loop {
        let events = inotify
            .read_events()
            .context("Error reading Inotify events")?;

        for InotifyEvent { name, mask, .. } in events {
            debug!(
                target: "inotify",
                "handling {0:?} event for {1:?}",
                mask,
                name
            );

            if name.is_some_and(|n| n == args.sentinel) {
                match mask {
                    AddWatchFlags::IN_DELETE => cleanup(&args.target, &args.sentinel)?,
                    AddWatchFlags::IN_CLOSE_WRITE => {
                        populate(&args.source, &args.sentinel, &args.target)?
                    }
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

    if !args.source.is_dir() {
        bail!("source ({0:?}) is not a directory", args.source);
    }
    if !args.target.is_dir() {
        bail!("target ({0:?}) is not a directory", args.target);
    }

    thread::spawn(move || {
        signals.forever().next();
        std::process::exit(0);
    });

    inotify
        .add_watch(
            &args.source,
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

    populate(&args.source, &args.sentinel, &args.target)?;
    monitor(&inotify, &args).context("Sentinel monitoring failed")?;

    Ok(())
}
