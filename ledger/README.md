# Klave App Rust Template
Use this template to scaffold a new Klave Rust application.

## 📖 About
Klave aims to make it easy to build and deploy WebAssembly application within Trusted Execution Environments (TEEs) and leverage the latest
developments in the [WebAssembly component model](https://github.com/WebAssembly/component-model) and [Wasmtime](https://wasmtime.dev/) runtime.

This template focus on leveragin the Klave ledger via the Klave SDK.
For a more detailed documentation, please read the [Klave docs](https://docs.klave.com/sdk/latest).

## 📦 Prerequisites
To use and build this template the following tools must be installed:
- The [Rust Toolchain](https://www.rust-lang.org/tools/install) (incl. rust, rustup, cargo)
- cargo-generate : `cargo install cargo-generate`
- cargo-component : `cargo install cargo-component`
- `wasm32-unknown-unknown` target : `rustup target add wasm32-unknown-unknown`

## 🚴 Usage

### 🐑 Use `cargo generate` to Clone this Template

[Learn more about `cargo generate` here.](https://github.com/ashleygwilliams/cargo-generate)

```
cargo generate --git https://github.com/klave-network/rust-template ledger --name my-project
cd my-project
```

### 🪼 Deploy on Klave

[Deploy on Klave](https://app.klave.com/login)

### 🛠️ You can also build locally

[Learn more about `cargo component` here.](https://github.com/bytecodealliance/cargo-component)

```cargo component build --target wasm32-unknown-unknown --release```

this creates a `target` folder with the built wasm files in `target\wasm32-unknown-unknown\release\`

## 🧩 Wasm component

Klave apps are `wasm component`.
In this template, three methods are implemented, registered and exposed: 
You can see these methods exposed in the `wit` [interface](https://github.com/klave-network/rust-template/blob/main/apps/rust-template/wit/world.wit):
- `export register-routes: func();`
- `export load-from-ledger: func(cmd: string);`
- `export insert-in-ledger: func(cmd: string);`

1 - The point of entry of the App is the `lib.rs` file and must expose the guest `wasm component` implementation:

```Rust
#[allow(warnings)]
mod bindings;

use bindings::Guest;
use klave;
struct Component;

impl Guest for Component {

    fn register_routes(){
        // By convention it is better to register the route with their wit names.
        // It means replacing the `_` by `-`
        // It will allows to call your routes with either `_` by `-`
        klave::router::add_user_query("your-first-query");
        klave::router::add_user_transaction("your-first-transaction");
    }

    fn your_first_query(cmd: String){
        // implement your Query
    }

    fn your_first_transaction(cmd: String){
        // Implement your Transaction
    }
}

bindings::export!(Component with_types_in bindings);
```
Make sure to register each Query or Transaction you want to expose via the `register_routes` method.

2 - Expose your `wasm component` interface in the `wit` file.

```wit
package component:{{component_name}};

/// An example world for the component to target.
world {{component_name}} {
    export register-routes: func();
    export your-first-query: func(cmd: string);
    export your-first-transaction: func(cmd: string);
}
```

## 🧑‍🤝‍🧑 Authors

This template is created by [Klave](https://klave.com) and [Secretarium](https://secretarium.com) team members, with contributions from:

- Etienne Bosse ([@Gosu14](https://github.com/Gosu14)) - [Klave](https://klave.com) | [Secretarium](https://secretarium.com)
- Jeremie Labbe ([@jlabbeklavo](https://github.com/jlabbeKlavo)) - [Klave](https://klave.com) | [Secretarium](https://secretarium.com)
- Nicolas Marie ([@Akhilleus20](https://github.com/akhilleus20)) - [Klave](https://klave.com) | [Secretarium](https://secretarium.com)