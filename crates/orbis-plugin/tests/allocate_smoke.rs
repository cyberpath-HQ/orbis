use std::fs;
use wasmtime::{Engine, Linker, Module, Store};

#[test]
fn smoke_allocate() {
    // Load plugin wasm
    let bytes = fs::read("../../plugins/my-first-plugin/plugin.wasm").expect("wasm not found");

    let engine = Engine::default();
    let module = Module::from_binary(&engine, &bytes).expect("failed to create module");

    let mut linker = Linker::new(&engine);

    // Provide stub host functions for imports
    linker.func_wrap("env", "state_get", |_: i32, _: i32| -> i32 { 0 }).unwrap();
    linker.func_wrap("env", "state_set", |_: i32, _: i32, _: i32, _: i32| -> i32 { 1 }).unwrap();
    linker.func_wrap("env", "state_remove", |_: i32, _: i32| -> i32 { 1 }).unwrap();
    linker.func_wrap("env", "log", |_: i32, _: i32, _: i32| {}).unwrap();
    linker.func_wrap("env", "db_query", |_: i32, _: i32, _: i32, _: i32| -> i32 { 0 }).unwrap();
    linker.func_wrap("env", "db_execute", |_: i32, _: i32, _: i32, _: i32| -> i32 { 1 }).unwrap();
    linker.func_wrap("env", "http_request", |_: i32, _: i32, _: i32, _: i32, _: i32, _: i32| -> i32 { 0 }).unwrap();
    linker.func_wrap("env", "emit_event", |_: i32, _: i32, _: i32, _: i32| -> i32 { 1 }).unwrap();
    linker.func_wrap("env", "get_config", |_: i32, _: i32| -> i32 { 0 }).unwrap();
    linker.func_wrap("env", "crypto_hash", |_: i32, _: i32, _: i32| -> i32 { 0 }).unwrap();
    linker.func_wrap("env", "crypto_random", |_: i32| -> i32 { 0 }).unwrap();

    let mut store = Store::new(&engine, ());

    let instance = linker.instantiate(&mut store, &module).expect("instantiate");

    // Get allocate export
    let alloc = instance.get_typed_func::<(i32,), i32>(&mut store, "allocate").expect("alloc func");

    // Small allocation should succeed and return a non-zero pointer
    let ptr = alloc.call(&mut store, (64,)).expect("alloc call failed");
    assert!(ptr != 0, "allocate returned 0 pointer");

    // Large allocation - try to allocate a huge amount to provoke failure
    let res = alloc.call(&mut store, (10_000_000,));
    match res {
        Ok(p) => println!("Large allocation returned ptr {} (ok)", p),
        Err(e) => println!("Large allocation trapped as expected: {}", e),
    }
}
