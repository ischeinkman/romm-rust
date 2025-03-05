use std::io;

use tokio::{fs, process::Command};

const SERVICE_INSTALL_PATH: &str = "/mnt/SDCARD/.tmp_update/startup/start-daemon.sh";
const SERVICE_PATH: &str = "./start-daemon.sh";

/// Installs the daemon into the Miyoo Mini, including:
///
/// * Moving all files to the correct locations.
/// * Telling the operating system to start the daemon on boot.
/// * Starting the daemon now.
pub async fn install_daemon() -> Result<(), anyhow::Error> {
    fs::copy(SERVICE_PATH, SERVICE_INSTALL_PATH).await?;
    start_daemon().await?;
    Ok(())
}

/// Uninstalls the daemon from the Miyoo Mini, including:
///
/// * Stopping the daemon.
/// * Telling the operating system to NOT start the daemon on boot.
/// * Removing any stray files in the OS.
pub async fn uninstall_daemon() -> Result<(), anyhow::Error> {
    stop_daemon().await?;
    match fs::remove_file(SERVICE_INSTALL_PATH).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub async fn restart_daemon() -> Result<(), anyhow::Error> {
    stop_daemon().await?;
    start_daemon().await?;
    Ok(())
}

pub async fn start_daemon() -> Result<(), anyhow::Error> {
    let res = Command::new(SERVICE_PATH).output().await?;
    if res.status.success() {
        return Ok(());
    }
    todo!()
}

pub async fn stop_daemon() -> Result<bool, anyhow::Error> {
    let res = Command::new("killall")
        .arg("syncer-daemon")
        .output()
        .await?;
    Ok(res.status.success())
}

pub async fn reinstall_daemon() -> Result<(), anyhow::Error> {
    install_daemon().await?;
    restart_daemon().await?;
    Ok(())
}

pub async fn daemon_is_installed() -> Result<bool, anyhow::Error> {
    match fs::try_exists(SERVICE_INSTALL_PATH).await {
        Ok(b) => Ok(b),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
}
