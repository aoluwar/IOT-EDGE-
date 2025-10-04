use anyhow::Result;
use log::{info, warn, error};
use serde::Deserialize;
use std::time::Duration;
use tokio::sync::Notify;
use ring::signature;
use std::fs;

#[derive(Deserialize)]
struct UpdateManifest {
    url: String,
    signature_url: Option<String>,
    version: String,
    notes: Option<String>,
}

pub async fn poll_updates(server: &str, shutdown: std::sync::Arc<Notify>) {
    info!(target: "ota", "Starting OTA poller against {}", server);
    let mut ticker = tokio::time::interval(Duration::from_secs(60));
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                if let Err(e) = try_check_once(server).await {
                    warn!(target: "ota", "OTA check failed: {:?}", e);
                }
            }
            _ = shutdown.notified() => {
                info!(target: "ota", "OTA poller shutting down");
                return;
            }
        }
    }
}

async fn try_check_once(server: &str) -> Result<()> {
    let manifest_url = format!("{}/latest.json", server);
    let res = reqwest::get(&manifest_url).await?;
    if !res.status().is_success() {
        return Err(anyhow::anyhow!("bad response {}", res.status()));
    }
    let manifest: UpdateManifest = res.json().await?;
    info!(target: "ota", "Found manifest: version {}", manifest.version);

    let art = reqwest::get(&manifest.url).await?;
    let bytes = art.bytes().await?;

    // signature required for production
    let sig_bytes = if let Some(sig_url) = manifest.signature_url {
        let s = reqwest::get(&sig_url).await?;
        let sb = s.bytes().await?;
        sb.to_vec()
    } else {
        warn!(target: "ota", "No signature provided; refusing to stage unsigned artifact");
        return Err(anyhow::anyhow!("unsigned artifact"));
    };

    // Load pubkey from filesystem (allow runtime-configurable path)
    let pubkey_path = "/etc/iot-agent/pubkey.ed25519";
    let pubkey = fs::read(pubkey_path).map_err(|e| anyhow::anyhow!("read pubkey {}: {}", pubkey_path, e))?;
    // Expected 32-byte raw public key for ED25519
    if pubkey.len() != 32 {
        return Err(anyhow::anyhow!("pubkey must be 32 bytes (ed25519 raw), found {}", pubkey.len()));
    }

    // Verify signature using ring
    let pk = signature::UnparsedPublicKey::new(&signature::ED25519, &pubkey);
    pk.verify(&bytes, &sig_bytes).map_err(|_| anyhow::anyhow!("signature verification failed"))?;
    info!(target: "ota", "Signature verified");

    // Stage artifact
    let staging_dir = "/var/lib/iot-updates";
    tokio::fs::create_dir_all(staging_dir).await?;
    let artifact_path = format!("{}/staged-update.bin", staging_dir);
    tokio::fs::write(&artifact_path, &bytes).await?;
    let sig_path = format!("{}/staged-update.sig", staging_dir);
    tokio::fs::write(&sig_path, &sig_bytes).await?;
    tokio::fs::write(format!("{}/ready", staging_dir), manifest.version.as_bytes()).await?;
    info!(target: "ota", "Staged update {}, ready marker written", artifact_path);

    Ok(())
}
