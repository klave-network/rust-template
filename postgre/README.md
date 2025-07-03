# Klave App Rust Template
Use this template to scaffold a new Klave Rust application and target our PostGreSQL connection service running in SGX.

## 📖 About
Klave aims to make it easy to build and deploy WebAssembly application within Trusted Execution Environments (TEEs) and leverage the latest
developments in the [WebAssembly component model](https://github.com/WebAssembly/component-model) and [Wasmtime](https://wasmtime.dev/) runtime.
For a more detailed documentation, please read the [Klave docs](https://docs.klave.com/sdk/latest).
This template demonstrates how to secure your PII data through deterministic encryption. Please note that the method used for this deterministic encryption has not been audited.
It serves as an example of how to leverage the Klave SDK to create deterministic encryption, but it offers no assurance regarding its cryptographic security or suitability for production environments.


## 📦 Prerequisites
To use and build this template the following tools must be installed:
- The [Rust Toolchain](https://www.rust-lang.org/tools/install) (incl. rust, rustup, cargo)
- cargo-generate : `cargo install cargo-generate`
- cargo-component : `cargo install cargo-component`
- `wasm32-unknown-unknown` target : `rustup target add wasm32-unknown-unknown`

## PostGreSQL 🗄️ database
To deploy this application on Klave, a primary prerequisite is an access to an external PostgreSQL database. This database must contain three specific tables: `users`, `products` and `purchases`.

We have provided the necessary resources to set up these tables:
* Table Schemas: You will find the SQL schemas for all three tables in `sql-example/schemas` directory
* Initial Data Feed: A sample data set for these tables is available in the  `sql-example/feed_db.sql` file. This file contains INSERT statements to populate the tables with initial values.

This example demonstrates a straightforward e-commerce model:

The `users` table holds user data, including various Personally Identifiable Information (PII) columns that are designed for deterministic encryption within this template.

The `products` table lists all available items that users can purchase.

The `purchases` table records all transactions, linking users to the products they have bought.

Before deploying the application, ensure your external PostgreSQL database is configured with these three tables and populated with the provided sample data from `feed_db.sql`

## 🚴 Usage

### 🪼 Deploy on Klave

[Deploy on Klave](https://app.klave.com/login)

### 🛠️ You can build locally before deploying on Klave

[Learn more about `cargo component` here.](https://github.com/bytecodealliance/cargo-component)

```cargo component build --target wasm32-unknown-unknown --release```

this creates a `target` folder with the built wasm files in `target\wasm32-unknown-unknown\release\`

## 🧩 Wasm component

Klave apps are `wasm component`.
In this template, five methods are implemented, registered and exposed:
You can see these methods exposed in the `wit` [interface](https://github.com/klave-network/klave-rust-postgre-template/blob/main/apps/klave-rust-postgre-template/wit/world.wit):
- `export register-routes: func();`
- `export db-setup: func(cmd: string);`
- `export execute-table-encryption: func(cmd: string);`
- `export read-encrypted-data-per-user: func(cmd: string);`
- `export avg-age-for-male: func(cmd: string);`
- `export avg-age-for-female: func(cmd: string);`

1 - The point of entry of the App is the `lib.rs` file and exposes the mandatory guest `wasm component` implementation. First api `db_setup` allows to record in the ledger the database connection settings, second api `execute_table_encryption` allows to encrypt deterministically your PIIs. Third, fourth and fifth apis are encrypted queries.

```Rust
#[allow(warnings)]
mod bindings;

use bindings::Guest;
use klave;
struct Component;

impl Guest for Component {

    fn register_routes(){
        klave::router::add_user_transaction(&String::from("db_setup"));
        klave::router::add_user_query(&String::from("execute_table_encryption"));

        //routes defined in business part
        klave::router::add_user_query(&String::from("read_encrypted_data_per_user"));
        klave::router::add_user_query(&String::from("avg_age_for_male"));
        klave::router::add_user_query(&String::from("avg_age_for_female"));
    }

    //endpoints to test Postgres client management
    fn db_setup(cmd: String) {
        let input: database::DBInputDetails = match serde_json::from_str(&cmd) {
            Ok(input) => input,
            Err(err) => {
                klave::notifier::send_string(&format!("Invalid input: {}", err));
                return;
            }
        };

        let mut clients = match database::Clients::load() {
            Ok(c) => c,
            Err(err) => {
                klave::notifier::send_string(&format!("Failed to load clients: {}", err));
                return;
            }
        };

        match clients.add(
            input.clone(),
        ) {
            Ok(database_id) => {
                klave::notifier::send_string(&database_id);
            },
            Err(err) => {
                klave::notifier::send_string(&format!("Failed to add database client: {}", err));
                return;
            }
        };
    }

    fn execute_table_encryption(cmd: String) {
        let db_table: database::DBTable = match serde_json::from_str(&cmd) {
            Ok(input) => input,
            Err(err) => {
                klave::notifier::send_string(&format!("Invalid input: {}", err));
                return;
            }
        };

        let mut client: database::Client = match database::Client::load(db_table.database_id.clone()) {
            Ok(c) => c,
            Err(err) => {
                klave::notifier::send_string(&format!("Failed to load client: {}", err));
                return;
            }
        };
        let _ = match client.connect() {
            Ok(_) => (),
            Err(err) => {
                klave::notifier::send_string(&format!("Failed to connect to client: {}", err));
                return;
            }
        };
        let _ = match client.encrypt_columns(db_table) {
            Ok(_) => (),
            Err(err) => {
                klave::notifier::send_string(&format!("Failed to encrypt columns: {}", err));
                return;
            }
        };
    }

    fn read_encrypted_data_per_user(cmd: String) {
        business::read_encrypted_data_per_user(cmd);
    }

    fn avg_age_for_male(cmd: String) {
        business::avg_age_for_male(cmd);
    }

    fn avg_age_for_female(cmd: String) {
        business::avg_age_for_female(cmd);
    }
}

bindings::export!(Component with_types_in bindings);
```
Make sure to register each additional Query or Transaction you want to expose via the `register_routes` method. Please note any call to `klave::sql::query`, `klave::sql::execute` and `klave::sql::connectionOpen` have to be done through a Query as the result is not deterministic.


## 🧑‍🤝‍🧑 Authors

This template is created by [Klave](https://klave.com) and [Secretarium](https://secretarium.com) team members, with contributions from:

- Nicolas Marie ([@Akhilleus20](https://github.com/akhilleus20)) - [Klave](https://klave.com) | [Secretarium](https://secretarium.com)
- Jeremie Labbe ([@jlabbeklavo](https://github.com/jlabbeKlavo)) - [Klave](https://klave.com) | [Secretarium](https://secretarium.com)

For more information and support, refer to the [Klave documentation](https://docs.klave.com) or contact the authors.