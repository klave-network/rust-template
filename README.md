# Klave Rust Templates Collection

A collection of Rust templates for building Klave applications using `cargo-generate`. Each template provides a specialized foundation for different use cases within Trusted Execution Environments (TEEs).

## 📖 About

Klave aims to make it easy to build and deploy WebAssembly applications within Trusted Execution Environments (TEEs) and leverage the latest developments in the [WebAssembly component model](https://github.com/WebAssembly/component-model) and [Wasmtime](https://wasmtime.dev/) runtime.

For detailed documentation, please read the [Klave docs](https://docs.klave.com/sdk/latest).

## 📦 Prerequisites

To use these templates, ensure the following tools are installed:
- The [Rust Toolchain](https://www.rust-lang.org/tools/install) (incl. rust, rustup, cargo)
- `cargo-generate`: `cargo install cargo-generate`
- `cargo-component`: `cargo install cargo-component`
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`

## ⛩️ Available Templates

### 📝 Attestation Template
TEE attestation and quote verification functionality.
```bash
cargo generate --git https://github.com/klave-network/rust-template attestation
```
**Features**: Quote generation, verification, and parsing for TEE security guarantees.

### 🔏 Ledger Template
Basic ledger operations for persistent data storage.
```bash
cargo generate --git https://github.com/klave-network/rust-template ledger
```
**Features**: Load and insert operations with Klave's secure ledger.

### 🗄️ PostgreSQL Template
Secure database connectivity with deterministic encryption.
```bash
cargo generate --git https://github.com/klave-network/rust-template postgre
```
**Features**: PII encryption, secure database operations, and SQL query execution.

### 🔗 EVM Light Client Template
Ethereum-compatible light client for blockchain interaction.
```bash
cargo generate --git https://github.com/klave-network/rust-template evm-light-client
```
**Features**: Header verification, block synchronization, and on-chain data validation.

### 💰 EVM Wallet Template
Secure wallet functionality for Ethereum-based networks.
```bash
cargo generate --git https://github.com/klave-network/rust-template evm-wallet
```
**Features**: Non-extractable private keys, multi-network support, and contract deployment.

### 🔒 EVM JSON-RPC Template
Ethereum JSON-RPC client for blockchain communication.
```bash
cargo generate --git https://github.com/klave-network/rust-template evm-json-rpc
```
**Features**: Standard Ethereum JSON-RPC methods and secure transaction handling.

### 🔐 MuSig2 Template
Multi-signature wallet implementation using MuSig2 protocol.
```bash
cargo generate --git https://github.com/klave-network/rust-template musig2
```
**Features**: Collaborative signing, key aggregation, and secure multi-party signatures.

## 🚴 Quick Start

1. **Choose a template** from the list above
2. **Generate your project** using the cargo generate command
3. **Navigate to your project** directory
4. **Build locally** (optional):
   ```bash
   cargo component build --target wasm32-unknown-unknown --release
   ```
5. **Deploy on Klave**: [Deploy on Klave](https://app.klave.com/login)

## 🧩 About Klave Applications

All Klave apps are WebAssembly components that:
- Run within Trusted Execution Environments (TEEs)
- Provide cryptographic guarantees of integrity and confidentiality
- Support both queries (read operations) and transactions (write operations)
- Use the WebAssembly Interface Types (WIT) for component interfaces

## 🧑‍🤝‍🧑 Authors

This template is created by [Klave](https://klave.com) and [Secretarium](https://secretarium.com) team members, with contributions from:

- Etienne Bosse ([@Gosu14](https://github.com/Gosu14)) - [Klave](https://klave.com) | [Secretarium](https://secretarium.com)
- Jeremie Labbe ([@jlabbeklavo](https://github.com/jlabbeKlavo)) - [Klave](https://klave.com) | [Secretarium](https://secretarium.com)
- Nicolas Marie ([@Akhilleus20](https://github.com/akhilleus20)) - [Klave](https://klave.com) | [Secretarium](https://secretarium.com)