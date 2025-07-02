# Cylixin Syntax

## Introduction

The syntax of Cylixin aims for clarity and a balance between expressiveness and ease of learning, drawing inspiration from existing languages while introducing unique elements like the `@` for function calls and explicit `then`/`end[element]` block structures. It also incorporates a unique "conditional block termination" feature for flexible control flow.

## Lexical Structure

### Identifiers

Identifiers are names used for variables, functions, etc.

`myVariable`
`userCount`
`calculate_area`

* They can contain letters (a-z, A-Z), digits (0-9), and underscores (`_`).
* They cannot start with a digit (e.g., `123variable` is invalid).
* Cylixin is **case-sensitive**: `myVariable` and `MyVariable` are treated as different identifiers.

### Keywords

Keywords are reserved words with special meanings in Cylixin.

```cylixin
let x = true; // 'let' is a keyword for constant declaration
if x then // 'if' and 'then' are keywords for conditional statements
    write("x is true");
endif // 'endif' is a keyword delineating the block
```

The current set includes: `let`, `var`, `const`, `if`, `else`, `elif`, `for`, `while`, `fun`, `return`, `then`, `when`, `true`, `false`, `null`, `int`, `long`, `float`, `char`, `strg`, `bool`, `set`, `dic`, `arr`, `empty`, `!empty`, `break`, `write`, `writeln`, `endif`, `endfor`, `endwhile`, `endfun`.

### Operators
Operators perform actions on values (operands).

```cylixin
let sum = 5 + 3; // '+' is the addition operator
if age > 18 then // '>' is the greater than comparison operator
    write("Adult");
endif
```

* *Arithmetic:* `+` (addition), `-` (subtraction), `*` (multiplication), `/` (division), `%` (modulo), `**` (exponentiation)

* *Comparison:* `==` (equal to), `===` (strict equal to - value and type), `!=` (not equal to), `>` (greater than), `<` (less than), `>=` (greater than or equal to), `<=` (less than or equal to)

* *Logical:* `&&` (logical AND), `||` (logical OR), `!` (logical NOT)

* *Assignment:* `=` (simple assignment), `+=`, `-=`, `*=`, `/=`, `%=`, `**=` (compound assignment)

* *Bitwise:* `&` (bitwise AND), `|` (bitwise OR), `>>` (right shift), `<<` (left shift)

Operator precedence will follow standard mathematical conventions (e.g., exponentiation has higher precedence than multiplication and division).

#### Delimiters and Special Symbols
Delimiters are characters that structure the code, and some special symbols have unique meanings.

* `(` `)`: Parentheses for grouping expressions and function calls.

* `{` `}`: Braces for defining blocks, sets, and dictionaries.

* `[` `]`: Square brackets for array indexing or array literals.

* `,`: Comma for separating elements in lists, arguments, etc.

* `;`: Semicolon for terminating statements.

* `:`: Colon for type annotations or dictionary key-value separation.

* `@`: Used to prefix function calls.

### Literals
Literals are direct representations of values in the code.

```cylixin
let integerValue = 10; // Integer literal
let longValue = 1234567890L; // Long integer literal (suffix L or l)
let piValue = 3.14; // Floating-point literal
let truth = true; // Boolean literal
let initial = 'A'; // Character literal
let message = "Hello"; // String literal
let nothing = null; // Null literal (keyword)
let mySet = {1, 2, 3}; // Set literal
let config = {"key": "value"}; // Dictionary literal
let myNumbers = [1, 2, 3]; // Array literal
```

* *Integers:* Whole numbers (e.g., `10`, `123`).

* *Long Integers:* Whole numbers with L or l suffix (e.g., `123L`).

* *Floating-Point Numbers:* Numbers with a decimal point (e.g., `3.14`).

* *Booleans:* `true` or `false`.

* *Characters:* Single character enclosed in single quotes (e.g., `'a'`, `'B'`, `'\n'`). Supports escape sequences.

* *Strings:* Text enclosed in double quotes (e.g., `"Hello, World!"`). Supports escape sequences.

* *Null:* Represents the absence of a value (is a keyword).

* *Sets:* Collections of unique items enclosed in `{}`.

* *Dictionaries:* Collections of key-value pairs enclosed in `{}`, with keys and values separated by `:`.

* *Arrays:* Ordered collections of items enclosed in `[]`.

### Comments
Comments are used to add explanations to the code and are ignored by the compiler/interpreter.

```cylixin
// This is a single-line comment.

/*
This is a
multi-line comment.
*/
```

* Single-line comments start with `//`.

* Multi-line comments are enclosed between `/*` and `*/`.

### Whitespace
Whitespace characters (spaces, tabs, newlines) are generally used to improve readability and separate tokens.

```cylixin
let x = 5; // Spaces around '=' and ';'
if x > 0 then
    write("Positive");
endif
```

* Whitespace is mostly for readability and separating elements.

* Indentation is not significant for defining code blocks; `then` and `end[element]` are used instead.

### Statements
Statements are instructions that the Cylixin interpreter/compiler executes.

#### Variable Declaration
Variables are declared to store data. Cylixin supports three keywords for variable declaration: `var` for mutable, function/global-scoped variables; `let` for mutable, block-scoped variables; and `const` for immutable, compile-time constants. For improved code clarity and faster processing, variables can (and often should) be explicitly type-annotated using a colon (`:`) followed by the type.

```cylixin
var globalName: strg = "Cylixin"; // Declares a mutable, globally/function-scoped string variable
let localCount: int = 0; // Declares a mutable, block-scoped integer variable
const PI_VALUE: float = 3.14159; // Declares an immutable, compile-time float constant

if true then
    let x: int = 10; // 'x' is block-scoped to this 'if' block
    write("Inside block, x:", x);
endif
// 'x' is not accessible here
var age: int = 30; // Declares a mutable integer variable
let bigNumber: long = 9876543210L; // Declares a mutable long integer
var initial: char = 'J'; // Declares a mutable character variable
let isValid: bool = true; // Declares a mutable boolean
var mySet: set = {1, 2}; // Declares a mutable set variable
let myDict: dic = {"key": "value"}; // Declares a mutable dictionary
var myScores: arr = [90, 85, 92]; // Declares a mutable array of integers

var score = 0; // Type inference can be used if no explicit type is provided
let greeting = "Hello"; // Type inference will determine the type here too
```

* Use `var` for variables that are mutable and have function or global scope.

* Use `let` for variables that are mutable but are scoped to the block in which they are declared (e.g., inside an `if` block, a `for` loop, or a _function_). They cease to exist once execution leaves that block.

* Use `const` for values that are immutable (cannot be reassigned) and are known at compile-time. They are typically block-scoped.

* Type-specific keywords (`int`, `long`, `float`, `char`, `strg`, `bool`, `set`, `dic`, `arr`) are used after a colon (:) to explicitly declare the data type. While type annotation is optional (Cylixin supports type inference), explicitly declaring types is highly recommended for clarity and can aid in optimization.

#### Assignment
Assignment statements are used to give a value to a variable.

```cylixin
count = 0; // Assigns the value 0 to the variable 'count'
message = "Hello"; // Assigns the string "Hello" to 'message'
```

Compound assignment operators provide a shorthand for common operations:

```cylixin
counter += 1; // Equivalent to 'counter = counter + 1'
```

#### Control Flow
Control flow statements determine the order in which code is executed. Cylixin supports optional labels for `for` and `while` loops to be used with the `break` statement, and a unique conditional block termination feature.

##### Labels:
A label is an identifier followed by a colon (:) placed before a loop.

```cylixin
outer_loop: for i = 0; i < 3; i += 1 then
    inner_loop: for j = 0; j < 3; j += 1 then
        write("i:", i, "j:", j);
        if i == 1 && j == 1 then
            break outer_loop; // Breaks out of the outer loop
        endif
    endfor
endfor
write("Outer loop finished.");
```

##### If/Else/Elif:
```cylixin
let temperature = 20;
if temperature > 25 then
    write("It's hot!");
elif temperature > 15 then
    write("It's warm.");
else then
    write("It's cool.");
endif
```
The `if` statement executes a block of code if a condition is true. `elif` (else if) checks another condition if the previous `if` or `elif` was false. `else` executes if none of the preceding conditions were true. The blocks are delimited by `then` and `endif`.

##### For Loop:

```cylixin
my_counter_loop: for i = 0; i < 5; i += 1 then
    write("Count:", i);
    if i == 3 then
        break my_counter_loop;
    endif
endfor
```

For loops define an initialization, a condition, and an increment expression. They can optionally be prefixed with a label followed by a colon. The block is delimited by `then` and `endfor`.

##### While Loop:

```cylixin
processing: while !finished then
    // ...
    if error_occurred then
        break processing;
    endif
endwhile
```
While loops execute as long as a condition is true. They can also be labeled. The block is delimited by `then` and `endwhile`.

##### Break Statement
The `break` statement is used to exit a loop prematurely. It can optionally take a label as an argument to break out of a specific labeled loop.

```cylixin
for i = 0; i < 5 then
    if i == 3 then
        break; // Breaks out of the current (innermost) loop
    endif
    write("i:", i);
endfor

outer: for a = 0; a < 2 then
    for b = 0; b < 2 then
        write("a:", a, "b:", b);
        if a == 1 && b == 0 then
            break outer; // Breaks out of the loop labeled 'outer'
        endif
    endfor
endfor
```
If `break` is used without a label, it exits the innermost loop containing the `break` statement. If a label is provided, it attempts to exit the loop with that label.

##### Conditional Block Termination
Cylixin provides a unique way to conditionally exit any block (identified by `if`, `for`, `while`, `fun`). When the compiler reaches this statement, the provided condition is evaluated. If the condition evaluates to `true`, the current block is immediately terminated, and execution continues after the block's natural end. If the condition is `false`, the statement is ignored, and execution continues normally within the block.

This feature adds flexibility, allowing developers to define an exit condition for a block at its syntactic end point, providing an alternative to `break` or `return` for specific scenarios.

###### Syntax:
`end[element_name] when (condition): [return_value_or_label (optional)];`

* `end[element_name]`: The keyword identifying the type of block being terminated (e.g., `endif`, `endfor`, `endwhile`, `endfun`).

* `when`: A new keyword indicating a conditional termination.

* `(condition)`: A boolean expression that, if `true`, triggers the exit.

* `[return_value_or_label (optional)]`:

  * For `endfun when`: An optional value to return from the function. If omitted, `null` is returned.

  * For `endfor when / endwhile when`: Behaves like a `break`. An optional label can be provided to exit a specific labeled loop (similar to the `break` statement).

  * For `endif when`: Terminates the entire `if`/`elif`/`else` chain, and execution continues after the `endif` statement.

###### Examples:

```cylixin
// Example 1: Conditional exit from an if block
let x = 10;
if x > 5 then
    write("x is greater than 5.");
    // ... more code ...
endif when (x == 10); // If x is 10 at this point, exit the 'if' block.
// If x was 10, this line would be reached immediately after the 'write' above.
// If x was, say, 7, this 'endif when' would be ignored, and code would continue
// within the 'if' block, then naturally exit the 'if'.
write("Execution continued after the if block.");


// Example 2: Conditional exit from a for loop
for i = 0; i < 10 then
    write("Loop iteration:", i);
endfor when (i == 5); // Exit the loop when 'i' reaches 5
// This will print 0, 1, 2, 3, 4, 5 then exit the loop.
write("Loop finished early.");


// Example 3: Conditional return from a function
fun calculate_sum(a: int, b: int): int then // Parameters and return type explicitly typed
    let total = a + b;
endfun when (total < 0): 0; // If total is negative, return 0
    return total; // This line is only reached if total is NOT negative
endfun // Regular function end if the above condition was false

let res1 = @calculate_sum(5, 3); // res1 will be 8
let res2 = @calculate_sum(5, -10); // res2 will be 0 due to conditional return
write("Result 1:", res1);
write("Result 2:", res2);


// Example 4: Conditional exit from a nested loop with a label
outer_loop: for x = 0; x < 3 then
    inner_loop: for y = 0; y < 3 then
        write("x:", x, " y:", y);
    endfor when (y == 1): outer_loop; // Exit 'outer_loop' if 'y' is 1
    endfor // Regular end of inner loop
endfor // Regular end of outer loop
write("After nested loops.");
```

##### Function Definition
Functions are reusable blocks of code.

```cylixin
fun add(a: int, b: int): int then // Function with typed parameters and explicit return type
    let sum = a + b;
    return sum;
endfun

fun greet(name: strg): strg then // Function returning a string
    return "Hello, " + name;
endfun

fun noReturn(val: int) then // Function with no explicit return type (implicitly returns null)
    write("Value received: ", val);
endfun

let result = @add(10, 5); // Calling the 'add' function
write("The sum is:", result);
let greeting_msg = @greet("World");
write(greeting_msg);
@noReturn(100);
```

Functions are defined using the `fun` keyword, followed by the function name, parameters in parentheses, and an optional return type annotation using a colon. Parameters can (and should) also be type-annotated using a colon. The function body starts after `then` and ends with `endfun`. The `return` statement specifies the value returned by the function. Functions are called using the `@` symbol before the function name.

###### Return Statement
The `return` statement exits a function and can optionally return a value.

```cylixin
fun isEven(number: int): bool then
    if number % 2 == 0 then
        return true;
    else then
        return false;
    endif
endfun

let num = 7;
if @isEven(num) then
    write(num, " is even.");
else then
    write(num, " is odd.");
endif
```
If a `return` statement is used without a value, the function implicitly returns `null`.

##### Printing
The write() and writeln() functions are used to output values to the console or another output stream.

```cylixin
write("Welcome to Cylixin!");
let name = "Developer";
writeln("Hello, ", name); // writeln adds a newline after output
let error = 404;
write("Error statement: " + error); // String concatenation using '+'
let value = 10;
writeln("The value is: " + value + ". Isn't it?"); // Multiple concatenations with newline
let pi = 3.14159;
write("The value of pi is: " + pi);
```
Both `write()` and `writeln()` are designed to output strings. When you provide non-string arguments, these functions will automatically convert them to their string form. This is particularly useful when using the `+` operator for string concatenation within the call to create formatted output. `writeln()` is similar to `write()` but appends a newline character to the output.

### Expressions
Expressions are parts of code that evaluate to a value.

```cylixin
let calculation = (5 * 2) + 1; // Arithmetic expression
let comparison = age >= 18; // Comparison expression
let logicalResult = true && false; // Logical expression
let greeting = @greet("World"); // Function call expression
```
Expressions are used in various contexts, such as assignments, conditions in control flow statements, and arguments to function calls.
