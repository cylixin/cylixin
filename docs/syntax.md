# Cylixin Syntax Reference

## Introduction

This document describes the syntax Cylixin's lexer, parser, and code generator actually support today, based on the current state of `cylixin_compiler`. Cylixin is still pre-alpha, so a few constructs are parsed but not yet fully wired up to codegen. Those are called out explicitly under [Known Limitations](#known-limitations) rather than presented as if they worked end to end.

Cylixin uses `then` to open blocks and explicit, block-specific keywords (`endif`, `endfor`, `endwhile`, `endfun`) to close them, so structure stays clear without relying on indentation. Function calls are prefixed with `@`. A conditional early-exit feature (`when`) is attached to these terminators for more flexible control flow; see [Conditional Block Termination](#conditional-block-termination).

## Lexical Structure

### Identifiers

```cylixin
myVariable
userCount
calculate_area
```

* Identifiers contain letters (`a` to `z`, `A` to `Z`), digits (`0` to `9`), and underscores (`_`).
* They cannot start with a digit.
* Cylixin is **case-sensitive**: `myVariable` and `MyVariable` are different identifiers.

### Keywords

The current lexer reserves the following words:

`let`, `var`, `const`, `int`, `long`, `float`, `string`, `char`, `bool`, `set`, `dic`, `arr`, `null`, `if`, `else`, `elif`, `then`, `when`, `for`, `from`, `to`, `while`, `break`, `fun`, `return`, `endif`, `endfor`, `endwhile`, `endfun`, `true`, `false`, `empty`, `read`, `write`, `writeln`.

Note the string type keyword is `string`, not `strg`. There is no `continue` keyword; see [Known Limitations](#known-limitations) for how early loop exits work today.

### Operators

```cylixin
let sum = 5 + 3;       // '+' addition
if age > 18 then       // '>' greater-than comparison
    write("Adult");
endif
```

* **Arithmetic:** `+`, `-`, `*`, `/`, `%`, `**` (exponentiation, right-associative)
* **Comparison:** `==`, `===` (strict, meaning both value and type must match), `!=`, `>`, `<`, `>=`, `<=`
* **Logical:** `&&`, `||`, `!`
* **Assignment:** `=`, `+=`, `-=`, `*=`, `/=`, `%=`, `**=`
* **Bitwise:** `&`, `|`, `<<`, `>>`

Precedence, from lowest to highest: `||`, then `&&`, then `|`, then `&`, then equality (`==`, `===`, `!=`), then comparison (`<`, `>`, `<=`, `>=`), then bitwise shifts (`<<`, `>>`), then additive (`+`, `-`), then multiplicative (`*`, `/`, `%`), then `**`, then unary (`!`, `-`), then calls/indexing, then primary expressions.

#### Delimiters and Special Symbols

* `(` `)`: grouping and function-call argument lists
* `{` `}`: set and dictionary literals
* `[` `]`: array literals and indexing
* `,`: separating list/argument elements
* `;`: terminating statements
* `:`: type annotations, dictionary key/value separation, and labels
* `@`: prefixes every function call, including built-ins like `write`/`writeln`

### Literals

```cylixin
let integerValue = 10;          // Integer
let longValue = 1234567890L;    // Long (L or l suffix)
let piValue = 3.14;             // Float
let truth = true;               // Bool
let initial = 'A';              // Char
let message = "Hello";          // String
let nothing = null;             // Null
let mySet: set<int> = {1, 2, 3};
let config: dic<int> = {1: "value"};
let myNumbers: arr<int> = [1, 2, 3];
```

* **Integers / Longs:** whole numbers; a trailing `L`/`l` marks a long literal.
* **Floats:** numbers with a decimal point.
* **Booleans:** `true` or `false`.
* **Characters:** single characters in single quotes, with escape-sequence support.
* **Strings:** double-quoted text, with escape-sequence support.
* **Null:** the `null` keyword.
* **Sets:** `{ ... }` with comma-separated values and no `:`, giving a set of unique items.
* **Dictionaries:** `{ ... }` with `key: value` pairs.
* **Arrays:** `[ ... ]`, an ordered, comma-separated collection.

An empty `{}` is parsed as an empty set literal.

### Comments

```cylixin
// Single-line comment

/*
Multi-line
comment.
*/
```

### Whitespace

Whitespace is for readability only. Indentation carries no syntactic meaning; `then` and the `end[element]` keywords define block boundaries instead.

## Statements

### Variable Declaration

```cylixin
var globalName: string = "Cylixin"; // mutable, function/global-scoped
let localCount: int = 0;            // mutable, block-scoped
const PI_VALUE: float = 3.14159;    // immutable, block-scoped

var score = 0;       // type inference, no annotation needed
let greeting = "Hi"; // type inference works here too
```

* `var`: mutable, function/global-scoped.
* `let`: mutable, scoped to the enclosing block.
* `const`: immutable, scoped to the enclosing block.
* A colon followed by a type name (`int`, `long`, `float`, `char`, `string`, `bool`, `set`, `dic`, `arr`) explicitly annotates the type. Annotations are optional, since the compiler falls back to inference when one is omitted, but they're recommended for clarity.
* Container types take a generic argument in `< >`: `arr<int>`, `set<int>`. `dic` currently accepts one generic argument (the key type) in the type annotation grammar; see [Known Limitations](#known-limitations) for the current gap around declaring the value type this way.

### Assignment

```cylixin
count = 0;
message = "Hello";
counter += 1; // shorthand for counter = counter + 1
```

Array elements can be assigned through indexing:

```cylixin
nums[2] = 99;
nums[0] += 5;
```

### Control Flow

#### If / Elif / Else

```cylixin
let temperature = 20;
if temperature > 25 then
    write("It's hot!");
elif temperature > 15 then
    write("It's warm.");
else
    write("It's cool.");
endif
```

`if` and each `elif` take a condition followed by `then`. `else` does **not** take its own `then`; it goes straight into its block. The whole chain closes with a single `endif`.

#### For Loops

Cylixin currently supports two `for` forms.

**Range form**, written `for <var> from <start> to <end> then ... endfor`:

```cylixin
for i from 0 to 5 then
    write("Count:", i);
endfor
```

**C-style form**, written `for <var> = <init>; <condition>; <var> <op> <update> then ... endfor`:

```cylixin
for i = 0; i < 5; i += 1 then
    write("Count:", i);
endfor
```

Both forms accept an optional label (see below) and an optional `endfor when` clause.

#### While Loops

```cylixin
processing: while !finished then
    // ...
endwhile
```

#### Labels and Break

A label is an identifier followed by `:` placed immediately before a `for` or `while` loop. `break` exits the innermost loop, or a specific labelled loop if given a label:

```cylixin
outer: for i from 0 to 3 then
    inner: for j from 0 to 3 then
        if i == 1 && j == 1 then
            break outer; // exits the outer loop
        endif
    endfor
endfor
```

`break` (with or without a label) always requires a terminating `;`.

#### Conditional Block Termination

`if`, `for`, and `while` blocks can be followed by a `when` clause after their terminator, giving an early-exit path attached to the syntactic end of the block:

```
end[element] when (condition): value;
```

* `end[element]` is `endif`, `endfor`, or `endwhile`.
* `(condition)` is a boolean expression.
* `value` is a required expression (there is no bare form without a value).

**Current behavior:** if `condition` evaluates to `true`, execution **returns `value` from the enclosing function immediately**, regardless of whether the clause is attached to `endif`, `endfor`, or `endwhile`. If `condition` is `false`, execution simply falls through to the statement after the block, as normal.

```cylixin
fun scanForNegative(a: int, b: int, c: int): int then
    if a < 0 then
        // ...
    endif when (a < 0): -1; // early-returns -1 from scanForNegative if a is negative

    if b < 0 then
        // ...
    endif when (b < 0): -1;

    return 0;
endfun
```

This differs from the original design intent, where an `endfor when`/`endwhile when` was meant to behave like a labelled `break`, and only `endfun when` was meant to return from the function. See [Known Limitations](#known-limitations) for where the two currently diverge, and use this feature with that in mind until the semantics are unified.

### Function Definition

```cylixin
fun add(a: int, b: int): int then
    let sum = a + b;
    return sum;
endfun

fun greet(name: string): string then
    return "Hello, " + name;
endfun

fun noReturn(val: int) then // no return type annotation, so it returns null
    write("Value received: ", val);
endfun

let result = @add(10, 5);
```

Functions are declared with `fun`, a name, a parenthesized, comma-separated parameter list (each parameter requires a `: type` annotation), and an optional `: returnType` after the closing parenthesis. The body runs from `then` to `endfun`. Calls are always `@`-prefixed.

An `endfun when (...): value;` clause is accepted by the parser but does not currently affect the generated function; see [Known Limitations](#known-limitations).

#### Return

```cylixin
fun isEven(number: int): bool then
    if number % 2 == 0 then
        return true;
    else
        return false;
    endif
endfun
```

`return;` with no expression implicitly returns `null`.

### I/O (Printing and Input)

```cylixin
// Printing
write("Welcome to Cylixin!");
writeln("Hello, ", name); // writeln appends a newline
write("Error code: " + error); // string concatenation with '+'

// Input (type-directed)
let age: int     = @read("Enter age: ");
let name: string = @read("Name: ");
let gpa: float   = @read("GPA: ");
let ok: bool     = @read("Continue? ");
let ch: char     = @read("Press key: ");
```

`write`/`writeln` and `read` are built-in functions, called like any other with `@`. 

**Printing:** `write`/`writeln` currently render `int`, `long`, `float`, `bool`, `char`, and `string` values. Arrays, sets, and dictionaries can be passed but currently print as a raw pointer value rather than their contents.

**Input:** `@read("prompt")` displays a prompt and waits for user input from standard in. The function it calls internally is determined by the **expected type** (the variable's type annotation). If the user enters invalid input for the type (e.g. typing "hello" for an `int`), the runtime prints an error and re-prompts automatically without crashing. The prompt string argument is required.

## Expressions

```cylixin
let calculation = (5 * 2) + 1;
let comparison = age >= 18;
let logicalResult = true && false;
let greeting = @greet("World");
let first = nums[0];
```

Expressions appear in assignments, conditions, function arguments, and indexing.

## Known Limitations

These are gaps between the language as designed and the language as currently implemented in `cylixin_compiler`. They're listed here so contributors and early adopters aren't surprised by them, and they're good first/early places to contribute:

* **`when` clause semantics are not yet unified across block types.** Today, `endif when`, `endfor when`, and `endwhile when` all compile to the same thing: an early `return value` from the enclosing function. The originally intended behavior, where `endfor`/`endwhile when` acted like a conditional labelled `break` and only `endfun when` returned from the function, has not been implemented yet.
* **`endfun when (...): value;` is currently a no-op.** The parser accepts the clause on a function body, but the value and condition are discarded during code generation; a function ending this way behaves as if the clause weren't written at all.
* **`dic<K, V>` two-parameter generics aren't parsed from a type annotation.** The type-annotation grammar for `dic` currently accepts a single generic argument; a comma-separated key/value pair (e.g. `dic<int, int>`) in an annotation is not yet supported. Dictionary literals themselves (`{1: 10, 2: 20}`) work regardless of how the variable is annotated.
* **The `empty` keyword is reserved but not yet usable.** It's recognized by the lexer but has no meaning in the parser or codegen yet (there's no `!empty` either).
* **No `continue` statement.** Only `break` (with the labelled early-exit `when` caveat above) is available for altering loop flow from inside the loop body.
* **Containers print as pointers.** `write`/`writeln` on an `arr`, `set`, or `dic` value prints its underlying pointer rather than a human-readable rendering of its contents.

If you're picking up any of these, please open an issue first per [CONTRIBUTING.md](../CONTRIBUTING.md) so effort doesn't get duplicated.
