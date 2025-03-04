/// Installs the daemon into the Miyoo Mini, including:
///
/// * Moving all files to the correct locations.
/// * Telling the operating system to start the daemon on boot.
/// * Starting the daemon now.
pub async fn install_daemon() -> Result<(), anyhow::Error> {
    Err(anyhow::anyhow!("TODO"))
}

/// Uninstalls the daemon from the Miyoo Mini, including:
///
/// * Stopping the daemon.
/// * Telling the operating system to NOT start the daemon on boot.
/// * Removing any stray files in the OS.
pub async fn uninstall_daemon() -> Result<(), anyhow::Error> {
    Err(anyhow::anyhow!("TODO"))
}
