use models::Dev;
use serde_json::Value;
use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store
};
use wasmtime_wasi::preview2::{
    pipe::MemoryOutputPipe, Table, WasiCtx, WasiCtxBuilder, WasiView
};

wasmtime::component::bindgen!(
    {
        path: "../wits/sensor.wit",
    }
);

/**
cargo build --target wasm32-wasi --release --package sensor-modbus-01
wasm-tools component new .\target\wasm32-wasi\release\sensor_modbus_01.wasm -o sensor/sensor-modbus-01.wasm --adapt ./wasi_snapshot_preview1.reactor.15.0.0.wasm

**/
fn main() -> wasmtime::Result<()> {
    // Configure an `Engine` and compile the `Component` that is being
    // run for the application.
    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;
    let component = Component::from_file(
        &engine,
        "sensor/sensor-modbus-01.wasm"
    )?;

    // Instantiation of bindings always happens through a `Linker`.
    // Configuration of the linker is done through a generated
    // `add_to_linker` method on the bindings structure.
    //
    // Note that the closure provided here is a projection from `T` in
    // `Store<T>` to `&mut U` where `U` implements the
    // `HelloWorldImports` trait. In this case the `T`, `MyState`,
    // is stored directly in the structure so no projection is
    // necessary here.
    let mut linker = Linker::<MyState>::new(&engine);
    // Sensor::add_to_linker(&mut linker, |state: &mut MyState|
    // state)?;

    // As with the core wasm API of Wasmtime instantiation occurs
    // within a `Store`. The bindings structure contains an
    // `instantiate` method which takes the store, component, and
    // linker. This returns the `bindings` structure which is an
    // instance of `HelloWorld` and supports typed access
    // to the exports of the component.
    let stdout = MemoryOutputPipe::new(4096);
    let stderr = MemoryOutputPipe::new(4096);
    let mut builder = WasiCtxBuilder::new();
    builder.stdout(stdout.clone()).stderr(stderr.clone());

    let mut store = Store::new(
        &engine,
        MyState {
            table: Table::new(),
            wasi:  builder.build()
        }
    );
    wasmtime_wasi::preview2::command::sync::add_to_linker(
        &mut linker
    )?;
    let (bindings, _) =
        Sensor::instantiate(&mut store, &component, &linker)?;

    // Here our `greet` function doesn't take any parameters for the
    // component, but in the Wasmtime embedding API the first
    // argument is always a `Store`.
    // bindings.call_greet(&mut store)?;
    let rs = bindings.call_get_info(&mut store)?;
    let dev: Dev = serde_json::from_slice(rs.as_slice()).unwrap();
    println!("{:?}", dev);
    let Ok(req_bytes) =
        bindings.call_get_request_bytes(&mut store, 1, 1)?
    else {
        println!(
            "error: {}",
            String::from_utf8_lossy(stdout.contents().as_ref())
        );
        return Ok(());
    };
    println!("{:02x?}", req_bytes);
    // 01 03 04 03 10 00 D7 BB EC
    let Some(rs) = bindings
        .call_get_rs(
            &mut store,
            1,
            1,
            &[0x01, 0x03, 0x04, 0x03, 0x10, 0x00, 0xd7, 0xbb, 0xec]
        )?
        .unwrap()
    else {
        println!(
            "error: {}",
            String::from_utf8_lossy(stdout.contents().as_ref())
        );
        return Ok(());
    };
    let val: Value = serde_json::from_slice(rs.as_slice()).unwrap();
    println!("{:02x?}", val);
    Ok(())
}

struct MyState {
    table: Table,
    wasi:  WasiCtx
}

impl WasiView for MyState {
    fn table(&self) -> &Table {
        &self.table
    }

    fn table_mut(&mut self) -> &mut Table {
        &mut self.table
    }

    fn ctx(&self) -> &WasiCtx {
        &self.wasi
    }

    fn ctx_mut(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}
