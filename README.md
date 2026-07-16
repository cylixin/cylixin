<div align="center">

# Cylixin

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Status: Early Development](https://img.shields.io/badge/status-early%20development-orange.svg)](#project-status)

**A statically-typed, explicit-syntax programming language, compiled to native code via LLVM.**

[Syntax Reference](docs/syntax.md) · [Design Principles](docs/design_principles.md) · [Architecture](docs/architecture.md) · [Contributing](CONTRIBUTING.md)

</div>

---

## What is Cylixin?

Cylixin is a programming language with a deliberately explicit syntax: blocks are opened with `then` and closed with type-specific terminators (`endif`, `endfor`, `endwhile`, `endfun`), function calls are prefixed with `@`, and every variable can be given an explicit type annotation. The goal is code whose structure is unambiguous at a glance, without relying on indentation.

The `cylixin_compiler` in this repository takes Cylixin source, lexes and parses it into an AST, and generates LLVM IR, which is then compiled to a native executable via `clang`/`llc`. This is a from-scratch compiler front-end and backend written in Rust, using [`inkwell`](https://github.com/TheDan64/inkwell) as the LLVM binding.

For the philosophy behind these design choices, see [Design Principles](docs/design_principles.md).

## Project Status

Cylixin is **pre-alpha**. There is no package manager and no standard library beyond `write`/`writeln`, and there's no installable release yet: you build the compiler from source and invoke it directly. That said, the core pipeline is functional today:

| Stage | Status |
|---|---|
| Lexer | ✅ Working |
| Parser → AST | ✅ Working |
| LLVM IR codegen | ✅ Working (ints, longs, floats, bools, chars, strings, arrays, sets, dicts, functions) |
| Native compilation | ✅ Working, via `clang`/`llc` on the emitted `.ll` file |
| Standard library | 🚧 `write` / `writeln` only |
| Package manager | 📋 Planned, not started |

See [Architecture](docs/architecture.md) for how the pieces fit together, and [Syntax Reference](docs/syntax.md) for exactly what the language supports right now, including a few rough edges that are still being ironed out.

## A Taste of Cylixin

```cylixin
fun add(a: int, b: int): int then
    return a + b;
endfun

var result: int = @add(3, 7);
@writeln(result);

var total: int = 0;
for i from 0 to 5 then
    total += i;
endfor

if total > 5 then
    @writeln("Sum is big!");
elif total > 3 then
    @writeln("Sum is medium.");
else
    @writeln("Sum is small.");
endif

var nums: arr<int> = [10, 20, 30];
nums[1] = 99;
@writeln(nums[1]);
```

More constructs (labelled loops, the `endif when` / `endfor when` conditional-exit feature, sets, dictionaries, and more) are covered in the [Syntax Reference](docs/syntax.md).

## Getting Started

### Prerequisites

* [Rust](https://www.rust-lang.org/tools/install) (stable toolchain, 2021 edition)
* LLVM 15 development libraries (the `llvm15-0` feature of `inkwell` is pinned in `Cargo.toml`)
* `clang` (or `llc` plus a linker) to turn the emitted LLVM IR into a native binary

### Build the compiler

```bash
git clone https://github.com/cylixin/cylixin.git
cd cylixin/cylixin_compiler
cargo build --release
```

### Run it

The compiler currently takes its source from a hardcoded string in `src/main.rs` rather than a file argument. That's the first thing to change as the CLI matures; see [Contributing](CONTRIBUTING.md) if you'd like to help. To try it out:

```bash
cargo run
```

This lexes, parses, and compiles the sample program in `main.rs`, writing the result to `output.ll` in the working directory. Turn that into an executable with:

```bash
clang output.ll cylixin_compiler/runtime.c -o program -lm
./program
```

`runtime.c` provides the small C runtime backing sets and dictionaries (hash-table implementations for `cy_set`/`cy_dict`), and it's linked in alongside the generated IR.

To experiment with your own program, edit the `source` string in `src/main.rs` and re-run `cargo run`.

## Repository Layout

```
cylixin/
├── cylixin_compiler/       # The compiler (Rust)
│   ├── src/
│   │   ├── lexer/          # Source text to tokens
│   │   ├── ast/            # AST node definitions
│   │   ├── parser/         # Tokens to AST
│   │   ├── codegen/        # AST to LLVM IR (via inkwell)
│   │   └── main.rs         # Entry point / demo program
│   └── runtime.c           # C runtime for sets & dicts
├── docs/
│   ├── design_principles.md
│   ├── syntax.md
│   └── architecture.md
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
└── LICENSE
```

## Contributing

Contributions are very welcome, especially given how early-stage this project is: there's a lot of surface area to help with, from closing gaps in the type system to writing the first standard library modules. Please see [CONTRIBUTING.md](CONTRIBUTING.md) for how to get involved, and open an issue before starting on anything substantial so we can align on direction.

## Community

* [Instagram](https://www.instagram.com/cylixin)
* [GitHub](https://github.com/cylixin)

## License

Cylixin is licensed under the [MIT License](LICENSE).
