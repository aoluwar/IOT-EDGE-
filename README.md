# IoT Edge Gateways That Don’t Die — Starter Project (Updated)

**Target**: Linux-based ARM gateways (example: Raspberry Pi Compute Module 4).  
**Goal**: resilient, updatable gateway agent in Rust using Tokio for async, with OTA and ring-verified signatures, plus a safe restart/watchdog pattern.

## What’s included (updated)
- `agent/` — Rust Tokio-based user-space agent (starter code) with ED25519 signature verification for OTA.
- `ota/` — updater applier (`applier/`) which applies staged updates atomically (skeleton). 
- `systemd/` — example systemd unit files for agent and applier.
- `scripts/` — cross-compile and build helper scripts.
- `README.md` — this file
- `LICENSE` — MIT

## New: Signature verification (ED25519)
The agent now performs ED25519 verification using `ring::signature::UnparsedPublicKey`.
Place `pubkey.ed25519` (raw 32-byte public key) in `agent/keys/` before building or embed it into the binary.

Example verification flow (implemented):
1. Download artifact and its `.sig` file (raw 64-byte ed25519 signature).
2. Verify with the provided public key.
3. If valid, stage artifact and write a `ready` marker.
4. The `applier` binary picks up staged artifact and performs an *out-of-process* apply (here: simulated copy to /opt/iot-app/current, with backup).

## How to build (example)
- Install Rust and add target `aarch64-unknown-linux-gnu` if building for aarch64:
  ```bash
  rustup target add aarch64-unknown-linux-gnu
  ```
- Build agent:
  ```bash
  cd agent
  cargo build --release --target aarch64-unknown-linux-gnu
  ```
- Copy `agent/target/aarch64-unknown-linux-gnu/release/iot-gateway-agent` to your device.
- Place your ED25519 public key bytes in `agent/keys/pubkey.ed25519` on builder before embedding or supply it at runtime under `/etc/iot-agent/pubkey.ed25519`.

## Notes & cautions
- This is a safe skeleton for teaching & testing. For production, integrate with a tested A/B updater (RAUC, swupdate) and secure key storage (TPM or secure element).
- Testing: generate ED25519 keypair, sign an artifact with `ed25519` tools (e.g., `openssl` evp or `libsodium`), and serve via simple HTTP for the agent to fetch.

## Next steps you can ask me to do
- Add RAUC-style A/B apply integration.
- Add TPM-backed key verification or use Linux keyring.
- Produce a bare-metal/Tock `no_std` firmware skeleton.
