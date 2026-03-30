# Reticulum-rs

A work-in-progress Rust implementation of the [Reticulum Network Stack](https://reticulum.network/).

## About

Reticulum is a cryptography-based networking stack for building resilient mesh networks over any medium. This is an experimental Rust port focused on performance and embedded systems support, For the time being it has only been successfully tested with an esp32 wroom module.

## Status

Early development. Core components implemented:
- ✅ Cryptographic primitives (X25519, Ed25519, AES-CBC, HKDF)
- ✅ Identity and packet structures
- ✅ Basic transport layer with routing
- 🚧 Interface implementations (in progress)
- ❌ Full protocol compatibility (not yet achieved)

## Building

```bash
cargo build
cargo run --example minimal
```

## Embedded Consumption (ESP32)

This crate now supports an embedded-oriented build surface by disabling default features:

```bash
cargo check --no-default-features
```

For ESP32 firmware projects (for example, a separate `wifi-test` app), depend on this crate with default features disabled:

```toml
[dependencies]
reticulum-rs = { path = "../reticulum-rs", default-features = false }
```

Notes:
- `transport` and `reticulum` modules are `std`-gated.
- `interfaces::wrapper` and host WiFi driver are `std`/`tokio`-gated.
- Embedded consumers should use interface-level APIs (for example WiFi traits/drivers) until full transport parity is implemented for no_std targets.

## License

MIT

