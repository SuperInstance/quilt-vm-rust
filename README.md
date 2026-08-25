# quilt-vm-rust

The 5-opcode Quilt VM in Rust.

> A runtime is a function from context to value with an inverse,
> advanced by a clock that processes async I/O while projecting
> a sync view.

## What this is

The Rust port of the foundation layer. The same 5 opcodes that
host 8 polyformalisms, written in safe Rust.

## The 5 opcodes

```rust
vm.bind("bathy:0", 4.2);          // BIND — make a thing
vm.link("a", "b", "type");        // LINK — connect things
vm.effect(target, fwd, inv);      // EFFECT — reversible change
vm.view("target", "viewer");      // VIEW — project for viewer
vm.tick(1.0);                     // TICK — advance time
```

## The 8 polyformalisms

1. Quilt cell
2. Cordis plugin
3. Spreadsheet
4. MUD
5. TTRPG (with perception check)
6. The bay dance
7. The cowboy
8. The bus

## Build

```bash
cargo build --release
cargo test
cargo run --release --bin quilt-gold
```

## Why Rust

Rust is the systems language. The cowboy's view of "what
doesn't crash." If the 5 opcodes work in Rust, they work
anywhere — Rust's borrow checker is the strictest test of
API design.

## Test count

7 tests, all passing.

## Related

- `quilt-foundation` (Python) — the original 5-opcode VM
- `quilt-vm-c` — C port
- `quilt-vm-typescript` — TypeScript port
- `quilt-vm-haskell` — Haskell port

## Version

0.1.0 — first public release.
