cargo build --target wasm32-unknown-unknown --release -p my-first-plugin
# cargo build --target wasm32-unknown-unknown -p my-first-plugin
cp ../../target/wasm32-unknown-unknown/release/my_first_plugin.wasm ./plugin.wasm
# cp ../../target/wasm32-unknown-unknown/debug/my_first_plugin.wasm ./plugin.wasm