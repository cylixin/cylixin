# Cylixin Design Principles

## Core Goals and Philosophy

Cylixin is envisioned as a programming language primarily aimed at **game development** and applications requiring strong **mathematical capabilities**. Our key priorities are **performance**, **being lightweight**, **clarity of syntax**, and **ease of use** for developers in these domains.

We believe that a language for game development and mathematical applications should strive for efficient execution and minimal overhead. Cylixin will be designed to be **lightweight**, meaning it should have a small runtime footprint and aim for efficient resource utilization. This will be achieved through a focused standard library and careful consideration of memory management.

**Performance** is a central tenet of Cylixin's design. We aim for fast execution speeds, enabling smooth and responsive applications, especially in demanding scenarios like game loops and complex simulations. This will influence decisions across the language, from syntax design to the underlying implementation.

## Syntax Design Principles

The syntax of Cylixin is designed with **readability** as a paramount concern. The use of `then` to begin code blocks and explicit `end[element]` keywords aims to make the structure of the code very clear and unambiguous, reducing potential errors related to indentation.
You can find a detailed description of the syntax in the [Cylixin Syntax document](syntax.md).

The `@` symbol for function calls is a deliberate choice to provide **visual distinction** and create a mental model of "invoking" or "calling upon" a specific piece of functionality.

We strive for a syntax that feels **logical and consistent**, even if some elements are novel. The learning curve should be manageable for developers familiar with other imperative languages.

## Performance and Efficiency

We are exploring a **hybrid Ahead-of-Time (AOT) and Just-in-Time (JIT) compilation strategy**. Core, essential parts of the Cylixin runtime and standard library may be compiled AOT to ensure fast startup and predictable baseline performance. Less critical or more dynamic parts of the code could be candidates for JIT compilation, provided the JIT compiler is highly efficient and introduces minimal runtime overhead. This hybrid approach aims to balance the benefits of both compilation techniques for optimal speed, performance, and resource utilization.

We may also explore options for **low-level access** in the future, potentially through specific standard library modules, to allow experienced developers to fine-tune performance for critical sections of code by interacting more directly with memory and hardware features when necessary.

## Low-Level Interaction via Libraries

To provide low-level capabilities without complicating the core language, we plan to offer a standard library module (e.g., `lowlevel`). This module would contain functions and types for operations such as raw memory access, bit manipulation, and Foreign Function Interface (FFI) to interact with code written in other languages like C. This approach keeps the core Cylixin language clean and safe for general use while providing powerful tools for developers who require more control.

## Future Development: Package Management

To foster a vibrant ecosystem and facilitate code sharing and reuse within the Cylixin community, we envision the development of a dedicated package manager. This tool would allow developers to easily discover, install, and manage external libraries and packages, extending the functionality of the core language and standard library.

This package manager would interact with a central repository of publicly available packages. It would handle dependency resolution, version management, and the installation process, making it straightforward for developers to incorporate community-contributed code into their Cylixin projects. This will be crucial for the long-term growth and adoption of Cylixin, enabling the creation of a rich and diverse set of libraries for various domains, including game development, mathematics, and beyond.
