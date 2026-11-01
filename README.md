# GAME TITLE

TODO: link

## Run locally

```bash
cargo run --release --features dev
```

## WASM

Install prerequisites:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130 --locked
cargo install basic-http-server
```

```bash
$ RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --profile wasm-release --target wasm32-unknown-unknown
$ wasm-bindgen --out-name jam8 \
  --out-dir wasm/target \
  --target web target/wasm32-unknown-unknown/wasm-release/bevy-jam-8.wasm

$ basic-http-server wasm
```

## Known issues

TODO

## Credits

TODO
