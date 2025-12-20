use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result, bail};
use crate::conf::config::Config;
use crate::defs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Silo {
    pub id: String,
    pub timestamp: u64,
    pub label: String,
    pub reason: String,
    pub config_snapshot: Config,
}

const RATOON_COUNTER_FILE: &str = "/data/adb/meta-hybrid/ratoon_counter";
const GRANARY_DIR: &str = "/data/adb/meta-hybrid/granary";
const MAX_AUTO_SILOS: usize = 5;

pub fn engage_ratoon_protocol() -> Result<()> {
    let path = Path::new(RATOON_COUNTER_FILE);
    let mut count = 0;

    if path.exists() {
        let content = fs::read_to_string(path).unwrap_or_default();
        count = content.trim().parse::<u8>().unwrap_or(0);
    }

    count += 1;
    fs::write(path, count.to_string())?;
    log::info!(">> Ratoon Protocol: Boot counter at {}", count);

    if count >= 3 {
        log::error!(">> RATOON TRIGGERED: Detected potential bootloop (3 failed boots).");
        log::warn!(">> Executing emergency rollback from Granary...");
        
        match restore_latest_silo() {
            Ok(_) => {
                log::info!(">> Rollback successful. Resetting counter.");
                let _ = fs::remove_file(path); 
            },
            Err(e) => {
                log::error!(">> Rollback failed: {}. Disabling all modules as last resort.", e);
                disable_all_modules()?;
            }
        }
    }

    Ok(())
}

pub fn disengage_ratoon_protocol() {
    let path = Path::new(RATOON_COUNTER_FILE);
    if path.exists() {
        if let Err(e) = fs::remove_file(path) {
            log::warn!("Failed to reset Ratoon counter: {}", e);
        } else {
            log::debug!("Ratoon Protocol: Counter reset. Boot successful.");
        }
    }
}

pub fn create_silo(config: &Config, label: &str, reason: &str) -> Result<String> {
    fs::create_dir_all(GRANARY_DIR)?;
    
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let id = format!("silo_{}", now);
    
    let silo = Silo {
        id: id.clone(),
        timestamp: now,
        label: label.to_string(),
        reason: reason.to_string(),
        config_snapshot: config.clone(),
    };

    let file_path = Path::new(GRANARY_DIR).join(format!("{}.json", id));
    let json = serde_json::to_string_pretty(&silo)?;
    fs::write(&file_path, json)?;

    prune_old_silos()?;

    Ok(id)
}

pub fn list_silos() -> Result<Vec<Silo>> {
    let mut silos = Vec::new();
    if !Path::new(GRANARY_DIR).exists() {
        return Ok(silos);
    }

    for entry in fs::read_dir(GRANARY_DIR)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = fs::read_to_string(&path)?;
            if let Ok(silo) = serde_json::from_str::<Silo>(&content) {
                silos.push(silo);
            }
        }
    }
    silos.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(silos)
}

pub fn restore_silo(id: &str) -> Result<()> {
    let file_path = Path::new(GRANARY_DIR).join(format!("{}.json", id));
    if !file_path.exists() {
        bail!("Silo {} not found", id);
    }

    let content = fs::read_to_string(&file_path)?;
    let silo: Silo = serde_json::from_str(&content)?;

    log::info!(">> Restoring Silo: {} ({})", silo.id, silo.label);
    silo.config_snapshot.save_to_file(crate::conf::config::CONFIG_FILE_DEFAULT)?;

    Ok(())
}

fn restore_latest_silo() -> Result<()> {
    let silos = list_silos()?;
    if let Some(latest) = silos.first() {
        restore_silo(&latest.id)
    } else {
        bail!("No silos found in Granary");
    }
}

fn prune_old_silos() -> Result<()> {
    let silos = list_silos()?;
    if silos.len() > MAX_AUTO_SILOS {
        for silo in &silos[MAX_AUTO_SILOS..] {
            let path = Path::new(GRANARY_DIR).join(format!("{}.json", silo.id));
            fs::remove_file(path).ok();
        }
    }
    Ok(())
}

fn disable_all_modules() -> Result<()> {
    let modules_dir = Path::new(defs::MODULES_DIR);
    if modules_dir.exists() {
        for entry in fs::read_dir(modules_dir)? {
            let entry = entry?;
            let disable_path = entry.path().join("disable");
            if !disable_path.exists() {
                fs::File::create(disable_path)?;
            }
        }
    }
    Ok(())
}