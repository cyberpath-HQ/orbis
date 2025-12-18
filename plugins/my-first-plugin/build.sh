cargo build --target wasm32-unknown-unknown --release -p my-first-plugin
cp ../../target/wasm32-unknown-unknown/release/my_first_plugin.wasm ./plugin.wasm