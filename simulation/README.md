# Iron Doctrine Simulation

This directory contains the authoritative Rust simulation workspace.

## Docker Environment

The Rust toolchain is pinned to `1.97.1` in `rust-toolchain.toml`.

Verified commands:

Run these commands from this `simulation/` directory.

```powershell
docker compose -f docker-compose.yml build rust
docker compose -f docker-compose.yml run --rm rust rustc --version
docker compose -f docker-compose.yml run --rm rust cargo --version
docker compose -f docker-compose.yml run --rm rust cargo metadata --no-deps --format-version 1
docker compose -f docker-compose.yml run --rm rust cargo check --workspace --all-targets
```
