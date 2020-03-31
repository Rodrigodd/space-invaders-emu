# Space Invaders Emulator

This is a emulator of the classic arcade game, Space Invaders (1978), write in Rust.

## Test it: https://rodrigodd.github.io/space-invaders-emu/

# Controls

'Z' is Shoot, 'Left' and 'Right' arrows to move, 'Return' for 1 Player Start button, and 'Backspace' for the 2 Player Button.

And for binary builds with the ```debug``` feature enable, press Escape to enter the debug mode. 

## debugger

There you can use the command ```run``` to exit the debug mode, use ```bp <HEX ADRESS>``` to place a breakpoint in same opcode,  and ```runto <HEX ADRESS>``` to run until reach same opcode.


# Compile

To compile to windows, linux or macOS (I only tested for windows), run the command ```cargo run```. 

Or ```cargo run --features=debug``` to enable the intel 8080 debugger.

(And add ```--release``` after the commands, because the rust debug build may not be fast enough)

## WebAssembly

To compile to WebAssembly, you can use ```wasm-pack``` with the command:

```
wasm-pack build --target web --out-dir ../pkg  ./space-invaders-wasm
```