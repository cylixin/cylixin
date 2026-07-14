# Cylixin Design Principles

> This document lays out the vision and long-term direction for Cylixin. Not everything described here is built yet; where that's the case, it's marked explicitly. For what's actually implemented today, see the [Syntax Reference](syntax.md) and [Architecture](architecture.md).

## Core Goals and Philosophy

Cylixin is envisioned as a programming language primarily aimed at **game development** and applications requiring strong **mathematical capabilities**. Our key priorities are **performance**, **being lightweight**, **clarity of syntax**, and **ease of use** for developers in these domains.

We believe that a language for game development and mathematical applications should strive for efficient execution and minimal overhead. Cylixin is designed to be **lightweight**: a small runtime footprint and efficient resource utilization, achieved through a focused standard library and careful memory management as those pieces get built out.

**Performance** is a central tenet of Cylixin's design, which is why the compiler generates LLVM IR and compiles to native code rather than interpreting. See [Architecture](architecture.md) for how that pipeline currently works.

## Syntax Design Principles

The syntax of Cylixin is designed with **readability** as a paramount concern. The use of `then` to begin code blocks and explicit `end[element]` keywords aims to make the structure of the code very clear and unambiguous, reducing potential errors related to indentation. You can find a detailed description of the current syntax in the [Syntax Reference](syntax.md).

The `@` symbol for function calls is a deliberate choice to provide **visual distinction** and create a mental model of "invoking" or "calling upon" a specific piece of functionality.

We strive for a syntax that feels **logical and consistent**, even if some elements are novel. The learning curve should be manageable for developers familiar with other imperative languages.

**Status:** the lexer, parser, and codegen implement the core of this syntax today, including variable declarations, typed functions, `if`/`elif`/`else`, both `for` forms, `while`, labelled `break`, and the `when` conditional-exit feature. The `when` feature's exact semantics across block types are still being finalized; see the Known Limitations section of the [Syntax Reference](syntax.md) for the current gap between intent and implementation.

## Performance and Efficiency

We are exploring a **hybrid Ahead-of-Time (AOT) and Just-in-Time (JIT) compilation strategy** as a long-term goal. Core, essential parts of the Cylixin runtime and standard library may be compiled AOT to ensure fast startup and predictable baseline performance. Less critical or more dynamic parts of the code could be candidates for JIT compilation, provided the JIT compiler is highly efficient and introduces minimal runtime overhead.

**Status today:** the compiler is AOT-only. Source is lexed, parsed, and lowered directly to LLVM IR, which is then compiled to a native binary via `clang`/`llc`. There is no JIT path yet; that remains a future direction, not a current feature.

We may also explore options for **low-level access** in the future, potentially through specific standard library modules, to allow experienced developers to fine-tune performance for critical sections of code by interacting more directly with memory and hardware features when necessary.

## Low-Level Interaction via Libraries

To provide low-level capabilities without complicating the core language, we plan to offer a standard library module (e.g., `lowlevel`). This module would contain functions and types for operations such as raw memory access, bit manipulation, and Foreign Function Interface (FFI) to interact with code written in other languages like C. This approach keeps the core Cylixin language clean and safe for general use while providing powerful tools for developers who require more control.

**Status:** not started. Today the only "standard library" surface is `write`/`writeln`, plus a small C runtime (`runtime.c`) backing sets and dictionaries that the compiler links against directly. There is no user-facing FFI or `lowlevel` module yet.

## Future Development: Package Management

To foster a vibrant ecosystem and facilitate code sharing and reuse within the Cylixin community, we envision the development of a dedicated package manager. This tool would allow developers to easily discover, install, and manage external libraries and packages, extending the functionality of the core language and standard library.

This package manager would interact with a central repository of publicly available packages, handling dependency resolution, version management, and installation. This will be crucial for the long-term growth and adoption of Cylixin, enabling a rich and diverse set of libraries for game development, mathematics, and beyond.

**Status:** planned, not started. There is currently no module system, import syntax, or package format defined.
