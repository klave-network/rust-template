# Klave Attestation App Template

Use this template to scaffold a new Klave Rust application focused on TEE attestation and quote verification.

## 📖 About

This template provides a ready-to-use implementation for working with Trusted Execution Environment (TEE) attestation quotes. It demonstrates how to generate, verify, and parse attestation quotes within the Klave platform, leveraging the security guarantees of TEEs to provide cryptographic proof of code integrity and execution environment.

The template includes three main functionalities:
- **Quote Generation**: Generate binary attestation quotes with random challenges
- **Quote Verification**: Verify the authenticity and validity of attestation quotes
- **Quote Parsing**: Parse and extract structured data from attestation quotes

For a more detailed documentation about Klave, please read the [Klave docs](https://docs.klave.com/sdk/latest).

## 📦 Prerequisites

To use and build this template the following tools must be installed:
- The [Rust Toolchain](https://www.rust-lang.org/tools/install) (incl. rust, rustup, cargo)
- cargo-generate : `cargo install cargo-generate`
- cargo-component : `cargo install cargo-component`
- `wasm32-unknown-unknown` target : `rustup target add wasm32-unknown-unknown`

## 🚴 Usage

### 🐑 Use `cargo generate` to Scaffold this Template

[Learn more about `cargo generate` here.](https://github.com/ashleygwilliams/cargo-generate)

```bash
cargo generate --git https://github.com/klave-network/rust-template attestation --name my-attestation-app
cd my-attestation-app
```

### 🪼 Deploy on Klave

[Deploy on Klave](https://app.klave.com/login)

### 🛠️ You can also build locally

[Learn more about `cargo component` here.](https://github.com/bytecodealliance/cargo-component)

```bash
cargo component build --target wasm32-unknown-unknown --release
```

This creates a `target` folder with the built wasm files in `target/wasm32-unknown-unknown/release/`

## 🧩 Wasm Component

Klave apps are `wasm component`. In this attestation template, three methods are implemented, registered and exposed:

You can see these methods exposed in the `wit` [interface](https://github.com/klave-network/rust-template/blob/main/attestation/apps/{{project-name}}/wit/world.wit):
- `export get-quote-binary: func(cmd: string);`
- `export verify-quote: func(cmd: string);`
- `export parse-quote: func(cmd: string);`

### 🔐 Attestation Methods

#### 1. **get-quote-binary**
Generates a new attestation quote with a random 64-byte challenge:
```rust
fn get_quote_binary(cmd: String) {
    // Generates random challenge and creates attestation quote
    // Returns: JSON with binary quote data
}
```

#### 2. **verify-quote**
Verifies the authenticity and validity of an attestation quote:
```rust
fn verify_quote(cmd: String) {
    // Input: JSON with "quote" field containing byte array
    // Verifies quote against current trusted time
    // Returns: JSON with verification results
}
```

#### 3. **parse-quote**
Parses and extracts structured information from an attestation quote:
```rust
fn parse_quote(cmd: String) {
    // Input: JSON with "quote" field containing byte array
    // Returns: JSON with parsed quote structure and metadata
}
```

### 📋 Implementation Structure

1 - The point of entry of the App is the `lib.rs` file and must expose the guest `wasm component` implementation:

```rust
#[allow(warnings)]
mod bindings;

use bindings::Guest;
use serde_json::json;
use serde_json::Value;

struct Component;

impl Guest for Component {
    fn register_routes() {
        // Register all attestation-related routes
        klave::router::add_user_query("get-quote-binary");
        klave::router::add_user_query("verify-quote");
        klave::router::add_user_query("parse-quote");
    }

    fn get_quote_binary(cmd: String) {
        // Generate attestation quote implementation
    }

    fn verify_quote(cmd: String) {
        // Verify attestation quote implementation
    }

    fn parse_quote(cmd: String) {
        // Parse attestation quote implementation
    }
}

bindings::export!(Component with_types_in bindings);
```

2 - Expose your `wasm component` interface in the `wit` file:

```wit
package component:{{component_name}};

/// An attestation world for TEE quote generation and verification.
world {{component_name}} {
    export register-routes: func();
    export get-quote-binary: func(cmd: string);
    export verify-quote: func(cmd: string);
    export parse-quote: func(cmd: string);
}
```

## 🔒 Security Features

This template leverages Klave's attestation capabilities to provide:

- **Remote Attestation**: Generate cryptographic proofs of code integrity
- **Quote Verification**: Validate attestation quotes against trusted time
- **Secure Random Generation**: Use TEE-provided random number generation
- **Trusted Time**: Access to secure, tamper-proof timestamps

## 📚 Usage Examples

### Generate an Attestation Quote
```json
// Call get-quote-binary with any string parameter
// Returns:
{
  "quote": [/* binary quote data as byte array */]
}
```

### Verify a Quote
```json
// Input to verify-quote:
{
  "quote": [/* binary quote data as byte array */]
}
// Returns verification results with validity status
```

### Parse a Quote
```json
// Input to parse-quote:
{
  "quote": [/* binary quote data as byte array */]
}
// Returns structured quote information
```

## 🧑‍🤝‍🧑 Authors

This template is created by [Klave](https://klave.com) and [Secretarium](https://secretarium.com) team members, with contributions from:

- Etienne Bosse ([@Gosu14](https://github.com/Gosu14)) - [Klave](https://klave.com) | [Secretarium](https://secretarium.com)

For more information and support, refer to the [Klave documentation](https://docs.klave.com) or contact the authors.