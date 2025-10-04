use anyhow::Result;
use std::time::Duration;
use log::{info, warn, error};
use std::sync::Arc;
use tokio::sync::Notify;

mod watchdog;
mod ota_client;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    env_logger::init();
    info!(target: "agent", "Starting IoT Gateway Agent v0.2");

    let shutdown = Arc::new(Notify::new());

    // Start watchdog
    let wd = watchdog::Watchdog::open("/dev/watchdog").await;
    let wd_handle = {
        let shutdown = shutdown.clone();
        tokio::spawn(async move {
            match wd {
                Ok(mut w) => w.heartbeat_loop(shutdown).await,
                Err(e) => warn!(target: "watchdog", "Watchdog open failed: {:?}", e),
            }
        })
    };

    // Start OTA poller
    let ota_handle = {
        let shutdown = shutdown.clone();
        tokio::spawn(async move {
            ota_client::poll_updates("https://example.com/updates", shutdown).await;
        })
    };

    // Wait for ctrl-c
    tokio::signal::ctrl_c().await?;
    info!(target: "agent", "Shutdown requested");
    shutdown.notify_waiters();

    // Allow tasks to exit
    tokio::time::sleep(Duration::from_secs(1)).await;

    info!(target: "agent", "Exiting");
    Ok(())
}
