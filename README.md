1 - Component has been scaffold with:
`cargo component new rust-template --lib`

2 - sdk.wit has been added into sdk/wit

3 - `cargo.toml` has been updated with:

```toml
[package.metadata.component.target.dependencies]
"klave:sdk" = { path = "sdk/wit" }

[profile.release]
lto = true
# Tell `rustc` to optimize for small code size.
opt-level = "s"
strip = true
```

4 - Bindings have been generated with:
`cargo component build --target wasm32-unknown-unknown --release`
this also create a `target` folder with the built wasm files in  `target\wasm32-unknown-unknown\release\`

5 - When the bindings are already created, it is possible to build with:
`cargo build --target wasm32-unknown-unknown --release`
