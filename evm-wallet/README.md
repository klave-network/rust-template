# EVM Wallet Template
Use this template written in Rust to help you scaffold a new EVM-Compatible Wallet application.
Note that no user management is available with this template, only a single wallet.

## Usage
This EVM Wallet application allows users to leverage confidential computing to create a wallet 
with a single non-extractable private key. It can be used to interact with any number of ethereum-based blockchains, all using that same private key/wallet.
On-chain transactions can be performed, however cross-chain transactions are not available here.
It also provides a possibility to deploy a solidity contract and call some limited routes like mint and burn for a contract inheriting the ERC20 contract specification.

## Prerequisites
To use and build this template the following tools must be installed:
- The [Rust Toolchain](https://www.rust-lang.org/tools/install) (incl. rust, rustup, cargo)
- Cargo component : `cargo install cargo-component`
- `wasm32-unknown-unknown` target : `rustup target add wasm32-unknown-unknown`

## Rust Libraries used
In this template, the following libraries are leveraged:
- alloy (https://crates.io/crates/alloy)
- http (https://crates.io/crates/http)

## Wasm component
Klave apps are `wasm component`.
In this template, three methods are implemented, registered and exposed: 
You can see these methods exposed in the `wit` [interface](https://github.com/klave-network/rust-template/blob/feat/main/evm-wallet/apps/%7B%7Bproject-name%7D%7D/wit/world.wit):
- `export register-routes: func();`
- `export export network-add: func(cmd: string);`
- `export network-set-chain-id: func(cmd: string);`
- `export network-set-gas-price: func(cmd: string);`
- `export networks-all: func(cmd: string);`
- `export wallet-add: func(cmd: string);`
- `export wallet-add-network: func(cmd: string);`
- `export wallet-address: func(cmd: string);`
- `export wallet-public-key: func(cmd: string);`
- `export wallet-balance: func(cmd: string);`
- `export wallet-networks: func(cmd: string);`
- `export wallet-transfer: func(cmd: string); `
- `export wallet-deploy-contract: func(cmd: string);`
- `export wallet-call-contract: func(cmd: string);`

## klave-networks folder
A local crate is available to handle all network features needed:
- multiple networks
- HTTP request formatting for 
  - JSON-RPC transactions/queries
  - authentication. 
Are currently supported public and private ethereum-based blockchains.

## solidity contracts
The CrossChainToken.sol contract is only available as an example of contract that is deployed and called as part of the evm-wallet.
In the background, Hardhat was used to test the contract and the bytecode was generated using the typescript solc.compile command.
In this particular example, only mint and burn methods can be called.

## Deploy Your App on Klave
[![Deploy on Klave](https://klave.com/images/deploy-on-klave.svg)](https://app.klave.com/login)

## You can also build locally
`cargo component build --target wasm32-unknown-unknown --release`
this also create a `target` folder with the built wasm files in  `target\wasm32-unknown-unknown\release\`

## Authors
This template is created by [Klave](https://klave.com) and [Secretarium](https://secretarium.com) team members, with contributions from:

- Jeremie Labbe ([@jlabbeklavo](https://github.com/jlabbeKlavo)) - [Klave](https://klave.com) | [Secretarium](https://secretarium.com)
