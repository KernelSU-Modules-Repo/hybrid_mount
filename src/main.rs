mod config;
mod magic_mount;
mod utils;

use std::io::Write;

use anyhow::{Context, Result};
use config::{CONFIG_FILE_DEFAULT, Config};
use env_logger::Builder;

use crate::magic_mount::UMOUNT;

fn load_config() -> Result<Config> {
    // 2. 尝试从默认位置加载
    if let Ok(config) = Config::load_default() {
        log::info!(
            "Loaded config from default location: {}",
            CONFIG_FILE_DEFAULT
        );
        return Ok(config);
    }

    // 3. 使用默认配置
    log::info!("Using default configuration (no config file found)");
    Ok(Config::default())
}

fn init_logger(verbose: bool) -> Result<()> {
    let level = if verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    let mut builder = Builder::new();

    builder.format(|buf, record| {
        writeln!(
            buf,
            "[{}] [{}] {}",
            record.level(),
            record.target(),
            record.args()
        )
    });
    builder.filter_level(level).init();

    log::info!("log level: {}", level.as_str());

    Ok(())
}

fn main() -> Result<()> {
    // 加载配置
    let config = load_config()?;

    // 初始化日志
    init_logger(config.verbose)?;

    log::info!("Magic Mount Starting");
    log::info!("module dir      : {}", config.moduledir.display());

    let tempdir = if let Some(temp) = config.tempdir {
        log::info!("temp dir (cfg)  : {}", temp.display());
        temp
    } else {
        let temp = utils::select_temp_dir().context("failed to select temp dir automatically")?;
        log::info!("temp dir (auto) : {}", temp.display());
        temp
    };

    log::info!("mount source    : {}", config.mountsource);
    log::info!("verbose mode    : {}", config.verbose);
    if !config.partitions.is_empty() {
        log::info!("extra partitions: {:?}", config.partitions);
    }

    utils::ensure_temp_dir(&tempdir)?;

    if config.umount {
        UMOUNT.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    let result = magic_mount::magic_mount(
        &tempdir,
        &config.moduledir,
        &config.mountsource,
        &config.partitions,
    );

    utils::cleanup_temp_dir(&tempdir);

    match result {
        Ok(_) => {
            log::info!("Magic Mount Completed Successfully");
            Ok(())
        }
        Err(e) => {
            log::error!("Magic Mount Failed");
            for cause in e.chain() {
                log::error!("{cause:#?}");
            }
            log::error!("{:#?}", e.backtrace());
            Err(e)
        }
    }
}
