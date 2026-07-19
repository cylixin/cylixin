# Compiler Architecture

This document walks through how `cylixin_compiler` turns Cylixin source text into a native executable, and where in the codebase each stage lives. It's aimed at contributors getting oriented in the code, not at Cylixin *users*. For the language itself, see the [Syntax Reference](syntax.md).

## Pipeline Overview

```
source text
    │
    ▼
┌─────────┐    Vec<Token>    ┌─────────┐    Program (AST)    ┌──────────┐    LLVM IR    ┌───────┐
│  Lexer  │ ───────────────▶ │  Parser │ ──────────────────▶ │  Codegen │ ────────────▶ │ clang │ ──▶ native binary
└─────────┘                  └─────────┘                     └──────────┘               └───────┘
src/lexer/                   src/parser/                     src/codegen/
```

`src/main.rs` implements a CLI that reads `.cyx` files, drives all four stages, prints progress, writes the result to a `.ll` file, and calls `clang` to link the final executable.

## Lexer (`src/lexer/`)

* `token.rs` defines `TokenKind` (every literal, keyword, operator, and delimiter Cylixin recognizes) and `Token`/`Span` for attaching line/column info to each token.
* `lexer.rs` implements `Lexer`, which walks the source character by character and produces a `Vec<Token>`, resolving identifiers against `TokenKind::from_keyword` to distinguish reserved words from user identifiers.

## AST (`src/ast/mod.rs`)

A single file defines the whole tree: `CyType` for type annotations, `Expr` for expressions (literals, binary/unary ops, calls, indexing, collection literals), and `Stmt` for statements (`VarDecl`, `Assign`, `If`, `ForRange`, `ForC`, `While`, `FunDecl`, `Return`, `Break`, `ExprStmt`). `EndWhen` captures the `when (condition): value` clause attached to `If`/`ForRange`/`ForC`/`While`. See the [Syntax Reference's Known Limitations](syntax.md#known-limitations) for how this is currently wired up in codegen versus how it's intended to work.

## Parser (`src/parser/parser.rs`)

A hand-written recursive-descent parser over the token stream, with a `ParseError` enum (`UnexpectedToken`, `UnexpectedEof`) carrying line/column info for diagnostics. Expression parsing uses precedence climbing (`parse_binary_from`), with `peek_binary_op` as the single source of truth for operator precedence. See [Syntax Reference](syntax.md#operators) for the resulting table. Statement parsing dispatches on the leading token in `parse_statement`.

## Codegen (`src/codegen/`)

* `compiler.rs` defines `Compiler<'ctx>`, wrapping an `inkwell::context::Context`, `Module`, and `Builder`. `compile()` walks the `Program`, first declaring all function signatures (so forward references between functions work), then emitting bodies. `compile_stmt` dispatches per `Stmt` variant; `compile_end_when` implements the current (function-level early-return) semantics of the `when` clause described in the syntax reference.
* `expressions.rs` handles `Expr` compilation: arithmetic and comparison ops on ints/longs/floats, string concatenation via calls into libc (`strlen`, `malloc`, `strcpy`/`strcat`, declared as external functions in the module), array/set/dict literals and indexing, and `compile_call`, which special-cases `write`/`writeln` (built directly on `printf`) and otherwise dispatches to user-defined functions registered during signature declaration.

Sets and dictionaries are backed by a small hash-table implementation in `runtime.c` (`cy_dict_*`/`cy_set_*`, open addressing with linear probing over `int64_t` keys/values), which is compiled and linked alongside the generated LLVM IR rather than reimplemented in IR directly.

## Where to Look for What

| If you want to... | Start in... |
|---|---|
| Add a new keyword or token | `src/lexer/token.rs`, `src/lexer/lexer.rs` |
| Add a new statement or expression form | `src/ast/mod.rs`, then `src/parser/parser.rs` |
| Change how something compiles to LLVM IR | `src/codegen/compiler.rs` or `src/codegen/expressions.rs` |
| Add a runtime-level data structure (beyond set/dict) | `runtime.c`, plus the corresponding `extern "C"` declarations in codegen |
| See the main compiler entry point and CLI commands | `src/main.rs` |

## Known Architectural Gaps
* There's no module or import system, so everything lives in one compilation unit today.
* There's no separate semantic-analysis / type-checking pass, so type errors currently surface as codegen errors (`CodegenError`) rather than being caught and reported earlier with good diagnostics.


If you're planning a larger change to any of this, please open an issue first per [CONTRIBUTING.md](../CONTRIBUTING.md).
