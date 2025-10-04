use anyhow::Result;
use log::{info, warn, error};
use std::path::Path;
use std::fs;

fn main() -> Result<()> {
    env_logger::init();
    info!("OTA applier starting");

    let staging = Path::new("/var/lib/iot-updates");
    let ready = staging.join("ready");
    if !ready.exists() {
        info!("No ready update found, exiting");
        return Ok(());
    }
    let version = fs::read_to_string(&ready)?;
    info!("Found staged update version: {}", version.trim());

    let staged = staging.join("staged-update.bin");
    if !staged.exists() {
        error!("Staged artifact missing: {:?}", staged);
        return Err(anyhow::anyhow!("staged artifact missing"));
    }

    let target_dir = Path::new("/opt/iot-app/current");
    let backup_dir = Path::new("/opt/iot-app/backup");

    fs::create_dir_all(target_dir)?;
    fs::create_dir_all(backup_dir)?;

    let current_bin = target_dir.join("app.bin");
    if current_bin.exists() {
        let backup_path = backup_dir.join("app.bin.bak");
        fs::rename(&current_bin, &backup_path)?;
        info!("Backed up current app to {:?}", backup_path);
    }

    let target_bin = target_dir.join("app.bin");
    fs::rename(&staged, &target_bin)?;
    info!("Moved staged artifact into place: {:?}", target_bin);

    let applied = staging.join("applied");
    fs::write(&applied, version.as_bytes())?;
    info!("Wrote applied marker: {:?}", applied);

    info!("Apply complete. Administrator should verify and reboot if necessary.");
    Ok(())
}
