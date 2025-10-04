use anyhow::Result;
use log::{info, warn};
use std::time::Duration;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::Notify;

pub struct Watchdog {
    file: tokio::fs::File,
}

impl Watchdog {
    pub async fn open(path: &str) -> Result<Self> {
        let file = OpenOptions::new().write(true).open(path).await?;
        Ok(Watchdog { file })
    }

    pub async fn heartbeat_loop(&mut self, shutdown: std::sync::Arc<Notify>) {
        info!(target: "watchdog", "Starting watchdog loop");
        let mut ticker = tokio::time::interval(Duration::from_secs(5));
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    if let Err(e) = self.file.write_all(b"\n").await {
                        warn!(target: "watchdog", "Failed heartbeat: {:?}", e);
                    }
                    let _ = self.file.flush().await;
                }
                _ = shutdown.notified() => {
                    info!(target: "watchdog", "Shutdown notified — attempting disarm");
                    let _ = self.file.write_all(b"V").await;
                    let _ = self.file.flush().await;
                    return;
                }
            }
        }
    }
}
