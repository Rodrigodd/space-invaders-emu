# Space Invaders Emulator

This is a emulator of the classic arcade game, Space Invaders (1978), write in Rust.

## Test it: https://rodrigodd.github.io/space-invaders-emu/

# Controls

- **C**: Insert Coin
- **Z**: Shoot
- **Left Arrow**: Move Left
- **Right Arrow**: Move Right
- **Return**: 1 Player Start
- **Backspace**: 2 Player Start

And for binary builds with the `debug` feature enable:
- **Esc**: Enter debugger.

## Debugger

When in debug mode, you can enter commands in the terminal: 
- `run` to exit the debug mode;
- `bp <HEX ADRESS>` to place a breakpoint at an address;
- `runto <HEX ADRESS>` to run until the given address.
- A empty line to execute one instruction.

# Compile And Run

To compile and run run the command `cargo run --release`. 

Or `cargo run --release --features=debug` to enable the intel 8080 debugger.

There are also some arguments you can pass:
- `-debug`: Start in debug mode.
- `test`: Run the test rom.
- `-d`: Dump ROM disassembly to stdout and exit.

## WebAssembly

To compile to WebAssembly, you can use `wasm-pack` with the command:

```
wasm-pack build --target web --out-dir ../pkg  ./space-invaders-wasm
```

You can run it in a webpage by starting a http server in the root directory.
You can use python for that, for example:

```
python -m http.server
```
