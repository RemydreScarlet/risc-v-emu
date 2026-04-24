# riscv-rust/wasm/web

[riscv-rust/wasm/web](https://github.com/takahirox/riscv-rust/tree/master/wasm/web) is a directory for the WebAssembly RISC-V emulator compiled from [riscv-rust](https://github.com/takahirox/riscv-rust) and its online demo. You can import the emulator into your web page.

## Online Demo

[index.html](https://takahirox.github.io/riscv-rust/wasm/web/index.html)

## How to Import in a Web Page

Download [riscv_emu_rust_wasm.js](https://github.com/takahirox/riscv-rust/blob/master/wasm/web/riscv_emu_rust_wasm.js) and [riscv_emu_rust_wasm_bg.wasm](https://github.com/takahirox/riscv-rust/blob/master/wasm/web/riscv_emu_rust_wasm_bg.wasm), and place them where your web page can access them.

Below is example code to import and use them:

```html
<script type="module">
  import init, { WasmRiscv } from "./riscv_emu_rust_wasm.js";
  init().then(async wasm => {
    const riscv = WasmRiscv.new();
    const programBuffer = await fetch(path_to_program).then(res => res.arrayBuffer());
    riscv.setup_program(new Uint8Array(programBuffer));

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
  });
</script>
```

## API

Refer to the comments in [`WasmRiscv`](https://github.com/takahirox/riscv-rust/blob/master/wasm/src/lib.rs)

## Debugger Commands

The web demo includes a built-in debugger. Press `Ctrl-A` in the terminal to enter debug mode.

| Command | Description |
|---------|-------------|
| `breakpoint` | Show set breakpoints |
| `breakpoint <address\|symbol>` | Set a breakpoint |
| `delete <address>` | Delete a breakpoint |
| `continue` | Continue execution |
| `step [n]` | Execute `n` steps (default: 1) |
| `pc` | Show program counter |
| `reg <n>` | Show register `n` (0-31) |
| `mem <address>` | Show 8-byte memory content |
| `help` | Show all commands |

## JIT Commands (Experimental)

The web demo supports experimental JIT compilation commands in debug mode:

| Command | Description |
|---------|-------------|
| `jit enable` | Enable JIT compilation |
| `jit disable` | Disable JIT compilation |
| `jit stats` | Show JIT statistics |
| `jit compile <start_addr> <end_addr>` | Manually compile a trace |

## How to Build WebAssembly RISC-V Emulator and Run Demo in Web Browser Locally

Prerequisites:
- Install [wasm-bindgen client](https://rustwasm.github.io/docs/wasm-bindgen/)

```sh
$ git clone https://github.com/takahirox/riscv-rust.git
$ cd riscv-rust/wasm
$ bash build.sh
# Boot a local server and access riscv-rust/wasm/web/index.html
```

