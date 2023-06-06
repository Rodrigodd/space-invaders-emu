# Space Invaders Emulator

This is a emulator of the classic arcade game, Space Invaders (1978), write in Rust.

## Test it: https://rodrigodd.github.io/space-invaders-emu/

# Controls

'Z' is Shoot, 'Left' and 'Right' arrows to move, 'Return' for 1 Player Start button, and 'Backspace' for the 2 Player Button.

And for binary builds with the `debug` feature enable, press Escape to enter the debug mode. 

## debugger

When in debug mode, you can enter commands in the terminal: 
- `run` to exit the debug mode;
-  `bp <HEX ADRESS>` to place a breakpoint at an address;
-  `runto <HEX ADRESS>` to run until the given address.

# Compile

To compile and run to Windows, Linux or macOS, run the command `cargo run --release`. 

Or `cargo run --release --features=debug` to enable the intel 8080 debugger.

## WebAssembly

To compile to WebAssembly, you can use `wasm-pack` with the command:

```
wasm-pack build --target web --out-dir ../pkg  ./space-invaders-wasm
```

You can run it in a webpage by starting a http server in the root directory.
You can use python for that, for example:

```
py -m http.server
```
