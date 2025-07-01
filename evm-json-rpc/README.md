# EVM JSON-RPC Client

This Rust-based template provides a foundation for interacting with Ethereum-based blockchains using JSON-RPC methods. It supports querying blockchain data, estimating gas, fetching transaction details, and other key Ethereum functionalities.

## Usage

The EVM JSON-RPC Client enables developers to interact with Ethereum-based blockchains using a set of predefined RPC methods. It does not include wallet-specific routes but focuses on retrieving blockchain information and executing calls to smart contracts.

## Prerequisites

To build and use this template, you need the following tools installed:

- **Rust Toolchain**: Includes `rust`, `rustup`, and `cargo`. [Install Rust](https://www.rust-lang.org/tools/install)
- **Cargo Component**: Install via `cargo install cargo-component`
- **WASM Target**: Add the target with `rustup target add wasm32-unknown-unknown`

## Rust libraries used

In this template, the following libraries are leveraged:
- alloy (https://crates.io/crates/alloy)
- http

## Wasm Component

Klave applications are built as WebAssembly (WASM) components. The following methods are implemented and exposed in the `evm-json-rpc` component:

```rust
- register-routes: Registers the available routes.
- network-add: Adds a new network.
- network-set-chain-id: Sets the chain ID for a network.
- network-set-gas-price: Sets the gas price for a network.
- networks-all: Lists all configured networks.
- eth-block-number: Retrieves the latest block number.
- eth-get-block-by-number: Fetches block details by block number.
- eth-gas-price: Retrieves the current gas price.
- eth-estimate-gas: Estimates the gas required for a transaction.
- eth-call-contract: Executes a call to a smart contract.
- eth-protocol-version: Retrieves the Ethereum protocol version.
- eth-chain-id: Fetches the chain ID of the connected network.
- eth-get-transaction-by-hash: Retrieves transaction details using its hash.
- eth-get-transaction-receipt: Fetches the receipt for a transaction.
- eth-get-transaction-count: Retrieves the number of transactions sent from an address.
- web-client-version: Returns the current version of the client.
- web-sha3: Computes the Keccak-256 hash of the input.
- net-version: Retrieves the network version.
- get-sender: Returns the sender address.
- get-trusted-time: Provides a trusted timestamp.
```

## Deploying Your App on Klave

To deploy your application on Klave:

1. **Build the Application**:
   ```sh
   cargo component build --target wasm32-unknown-unknown --release
   ```
   This command generates the WASM files in the `target/wasm32-unknown-unknown/release/` directory.

2. **Deploy to Klave**: Follow the deployment instructions provided in the [Klave documentation](https://docs.klave.com/deployment).

## Authors

This template is created by Klave and Secretarium team members, with contributions from:

- Jeremie Labbe ([@jlabbeklavo](https://github.com/jlabbeklavo)) - Klave | Secretarium

For more information and support, refer to the [Klave documentation](https://docs.klave.com) or contact the authors.

