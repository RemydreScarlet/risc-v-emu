# riscv_emu_rust_wasm

[![npm version](https://badge.fury.io/js/riscv_emu_rust_wasm.svg)](https://badge.fury.io/js/riscv_emu_rust_wasm)

riscv_emu_rust_wasm is a WebAssembly [RISC-V](https://riscv.org/) processor and peripheral devices emulator based on [riscv-rust](https://github.com/takahirox/riscv-rust).

## How to Install

```
$ npm install riscv_emu_rust_wasm
```

## How to Use

```javascript
const riscv = require('riscv_emu_rust_wasm').WasmRiscv.new();

// Setup program content binary (ELF file)
riscv.setup_program(new Uint8Array(elfBuffer));

// Setup filesystem content binary (optional)
riscv.setup_filesystem(new Uint8Array(fsBuffer));

// Optionally enable experimental JIT compilation
riscv.enable_jit(true);

// Emulator needs to break program regularly to handle input/output
// because the emulator is currently designed to run in a single thread.
const runCycles = () => {
  // Run 0x100000 (or certain) cycles, handle input/output,
  // and fire next cycles.
  // Note: Every instruction is completed in a cycle.
  setTimeout(runCycles, 0);
  riscv.run_cycles(0x100000);

  // Output handling
  while (true) {
    const data = riscv.get_output();
    if (data !== 0) {
      // print data
    } else {
      break;
    }
  }

  // Input handling. Assuming inputs holds input ascii data.
  while (inputs.length > 0) {
    riscv.put_input(inputs.shift());
  }
};
runCycles();
```

## API

### Core Methods

- `WasmRiscv.new()` — Creates a new emulator instance
- `setup_program(content: Uint8Array)` — Loads an ELF program
- `setup_filesystem(content: Uint8Array)` — Loads a filesystem image
- `setup_dtb(content: Uint8Array)` — Loads a device tree blob
- `run()` — Runs forever (or until test completion)
- `run_cycles(cycles: number)` — Runs for a fixed number of cycles
- `run_until_breakpoints(breakpoints: BigUint64Array, max_cycles: number): boolean` — Runs until a breakpoint is hit

### I/O Methods

- `get_output(): number` — Gets one output byte (0 if buffer empty)
- `put_input(data: number)` — Sends one input byte to the emulator

### Inspection Methods

- `read_register(reg: number): BigInt` — Reads a register (0-31)
- `read_pc(): BigInt` — Reads the program counter
- `load_doubleword(address: BigInt, error: Uint8Array): BigInt` — Reads memory
- `disassemble_next_instruction()` — Disassembles the next instruction (output via get_output)
- `get_address_of_symbol(symbol: string, error: Uint8Array): BigInt` — Looks up a symbol address

### JIT Methods (Experimental)

- `enable_jit(enabled: boolean)` — Enables or disables JIT compilation
- `compile_trace(start_addr: BigInt, end_addr: BigInt): boolean` — Manually compiles a trace
- `get_jit_stats(): string` — Returns JIT statistics as a JSON string

### Configuration Methods

- `enable_page_cache(enabled: boolean)` — Enables experimental page cache optimization
- `load_program_for_symbols(content: Uint8Array)` — Loads symbols from an ELF file

Refer to [the comments in WasmRiscv](../../wasm/src/lib.rs) for more detail.

## How to Build WebAssembly RISC-V Emulator Locally

Prerequisites:
- Install [wasm-bindgen client](https://rustwasm.github.io/docs/wasm-bindgen/)

```sh
$ git clone https://github.com/takahirox/riscv-rust.git
$ cd riscv-rust/wasm
$ bash build.sh
```

