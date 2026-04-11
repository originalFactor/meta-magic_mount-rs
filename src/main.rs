// Copyright 2025 Magic Mount-rs Authors
// SPDX-License-Identifier: GPL-3.0-or-later
#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]

mod config;
mod defs;
mod magic_mount;
mod misc;
mod scanner;
mod utils;

use std::path::Path;

use anyhow::Result;
use mimalloc::MiMalloc;
use rustix::mount::{MountFlags, mount};

use crate::{
    config::{Config, handle_gen_config, handle_save_config, handle_show_config},
    defs::MODULE_PATH,
};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() -> Result<()> {
    misc::pre_init();

    let args: Vec<_> = std::env::args().collect();

    if let Some(arg) = args.get(1) {
        match arg.as_str() {
            "show-config" => {
                handle_show_config()?;
            }
            "save-config" => {
                handle_save_config(&args[2..])?;
            }
            "gen-config" => {
                handle_gen_config()?;
            }
            "modules" => {
                let config = Config::load_or_default();
                let modules = scanner::list_modules(MODULE_PATH, &config.partitions);
                println!("{}", serde_json::to_string(&modules)?);
            }
            "version" => {
                println!("{{ \"version\": \"{}\" }}", env!("CARGO_PKG_VERSION"));
            }
            _ => {}
        }

        return Ok(());
    }

    let config = Config::load()?;

    utils::ksucalls::check_ksu();

    log::info!("Magic Mount Starting");
    log::info!("config info:\n{config}");

    log::debug!(
        "current selinux: {}",
        std::fs::read_to_string("/proc/self/attr/current")?
    );

    let tempdir = utils::generate_tmp();

    utils::ensure_dir_exists(&tempdir)?;

    if let Err(e) = mount(
        &config.mountsource,
        &tempdir,
        "tmpfs",
        MountFlags::empty(),
        None,
    ) {
        panic!("mount tmpfs failed: {e}");
    }

    let result = magic_mount::magic_mount(
        &tempdir,
        Path::new(MODULE_PATH),
        &config.mountsource,
        &config.partitions,
        #[cfg(any(target_os = "linux", target_os = "android"))]
        config.umount,
    );

    let cleanup = || {
        use rustix::mount::{UnmountFlags, unmount};
        if let Err(e) = unmount(&tempdir, UnmountFlags::DETACH) {
            log::warn!("failed to unmount tempdir: {e}");
        }
        if let Err(e) = std::fs::remove_dir(&tempdir) {
            log::warn!("failed to remove tempdir: {e}");
        }
    };

    match result {
        Ok(()) => {
            log::info!("Magic Mount Completed Successfully");
            cleanup();
            Ok(())
        }
        Err(e) => {
            log::error!("Magic Mount Failed");
            for cause in e.chain() {
                log::error!("{cause:#?}");
            }
            log::error!("{:#?}", e.backtrace());
            cleanup();
            Err(e)
        }
    }
}
