# Cylixin Syntax

## Introduction

The syntax of Cylixin aims for clarity and a balance between expressiveness and ease of learning, drawing inspiration from existing languages while introducing unique elements like the `@` for function calls and the explicit `then`/`end[element]` block structure.

## Lexical Structure

### Identifiers

Identifiers are names used for variables, functions, etc.

`myVariable`
`userCount`
`calculate_area`

* They can contain letters (a-z, A-Z), digits (0-9), and underscores (_).
* They cannot start with a digit (e.g., `123variable` is invalid).
* Cylixin is **case-sensitive**: `myVariable` and `MyVariable` are treated as different identifiers.

### Keywords

Keywords are reserved words with special meanings in Cylixin.

```cylixin
let x = true; // 'let' is a keyword for constant declaration
if x then // 'if' and 'then' are keywords for conditional statements
    write("x is true");
endif
```
The current set includes: `let`, `var`, `if`, `else`, `elif`, `for`, `while`, `fun`, `return`, `then`, `endfun`, `endif`, `endfor`, `endwhile`, `true`, `false`, `null`, `int`, `float`, `strg`, `bool`, `set`, `dic`, `empty`, `!empty`, `break`.

### Operators

Operators perform actions on values (operands).

```cylixin
let sum = 5 + 3; // '+' is the addition operator
if age > 18 then // '>' is the greater than comparison operator
    write("Adult");
endif
```

- **Arithmetic:** `+`(addition), `-`(subtraction), `*`(multiplication), `/`(division), `%`(modulo), `**`(exponentiation), `//`(integer division)
- **Comparison:** `==`(equal to), `===`(strict equal to - value and type), `!=`(not equal to), `>`(greater than), `<`(less than), `>=`(greater than or equal to), `<=`(less than or equal to)
- **Logical:** `and`, `or`, `not`
- **Assignment:** `=` (simple assignment), `+=`, `-=`, `*=`, `/=`, `%=` (compound assignment)
- **Bitwise:** `>>`(right shift), `<<`(left shift)

Operator precedence will follow standard mathematical conventions (e.g., exponentiation has higher precedence than multiplication and division).

### Literals

Literals are direct representations of values in the code.

```cylixin
let integerValue = 10; // Integer literal
let piValue = 3.14; // Floating-point literal
let truth = true; // Boolean literal
let message = "Hello"; // String literal
let nothing = null; // Null literal
let notANumber = NaN; // NaN literal
let mySet = {1, 2, 3}; // Set literal
let config = {"key": "value"}; // Dictionary literal
```

- **Integers:** Whole numbers.
- **Floating-Point Numbers:** Numbers with a decimal point.
- **Booleans:** `true` or `false`.
- **Strings:** Text enclosed in double quotes.
- **Null:** Represents the absence of a value.
- **NaN:** "Not a Number," typically the result of invalid numerical operations.
- **Sets:** Collections of unique items enclosed in `{}`.
- **Dictionaries:** Collections of key-value pairs enclosed in `{}`, with keys and values separated by `:`.

### Comments

Comments are used to add explanations to the code and are ignored by the compiler/interpreter.

```cylixin
// This is a single-line comment.

/*
This is a
multi-line comment.
*/
```

- Single-line comments start with `//`.
- Multi-line comments are enclosed between `/*` and `*/`.

### Whitespace

Whitespace characters (spaces, tabs, newlines) are generally used to improve readability and separate tokens.

```cylixin
let x = 5; // Spaces around '=' and ';'
if x > 0 then
    write("Positive"); // Indentation for readability (not syntactically required)
endif
```

- Whitespace is mostly for readability and separating elements.
- Indentation is not significant for defining code blocks; `then` and `end[element]` are used instead.

### Statements

Statements are instructions that the Cylixin interpreter/compiler executes.

#### Variable Declaration
Variables are declared to store data.
```cylixin
var name = "Cylixin"; // Declares a mutable variable 'name'
let constantValue = 3.14; // Declares an immutable constant 'constantValue'
int age = 30; // Declares a variable 'age' specifically for integers
```

You can use `var` for variables whose value might change, and `let` for constants whose value should not be reassigned within their scope. Type-specific keywords (`int`, `float`, `strg`, `bool`, `set`, `dic`) can be used to explicitly declare the data type of a variable.

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
Control flow statements determine the order in which code is executed. Cylixin supports optional labels for `for` and `while` loops to be used with the `break` statement.

**Labels:** <br>
A label is an identifier followed by a colon (`:`) placed before a loop.

```cylixin
outer_loop: for i = 0; i < 3; i += 1 then
    inner_loop: for j = 0; j < 3; j += 1 then
        write("i:", i, "j:", j);
        if i == 1 and j == 1 then
            break outer_loop; // Breaks out of the outer loop
        endif
    endfor
endfor
write("Outer loop finished.");
```

**If/Else/Elif:**

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

The `if` statement executes a block of code if a condition is true. `elif` (else if) checks another condition if the previous if or elif was false. `else` executes if none of the preceding conditions were true. The blocks are delimited by `then` and `endif`.

**For Loop:**

```cylixin
my_loop: for item in collection then // Example of a potential range-based loop with a label
    write("Item:", item);
    if item == target then
        break my_loop;
    endif
endfor
```
Loops can optionally be prefixed with a label followed by a colon.

**While Loop:**

```cylixin
processing: while not finished then
    // ...
    if error_occurred then
        break processing;
    endif
endwhile
```
While loops can also be labeled.

### Break Statement
The `break` statement is used to exit a loop prematurely. It can optionally take a label as an argument to break out of a specific labeled loop.
```cylixin
for i = 0; i < 5; i += 1 then
    if i == 3 then
        break; // Breaks out of the current (innermost) loop
    endif
    write("i:", i);
endfor

outer: for a = 0; a < 2; a += 1 then
    for b = 0; b < 2; b += 1 then
        write("a:", a, "b:", b);
        if a == 1 and b == 0 then
            break outer; // Breaks out of the loop labeled 'outer'
        endif
    endfor
endfor
```
If `break` is used without a label, it exits the innermost loop containing the `break` statement. If a label is provided, it attempts to exit the loop with that label.

### Function Definition

Functions are reusable blocks of code.

```cylixin
fun add(a, b) then
    let sum = a + b;
    return sum;
endfun

let result = @add(10, 5); // Calling the 'add' function
write("The sum is:", result);
```
Functions are defined using the `fun` keyword, followed by the function name and parameters in parentheses. The function body starts after `then` and ends with `endfun`. The `return` statement specifies the value returned by the function. Functions are called using the `@` symbol before the function name.

**Return Statement** <br>
The `return` statement exits a function and can optionally return a value.
```cylixin
fun isEven(number) then
    if number % 2 == 0 then
        return true;
    else then
        return false;
    endif
endfun

let num = 7;
if @isEven(num) then
    write(num, "is even.");
else then
    write(num, "is odd.");
endif
```
If a `return` statement is used without a value, the function implicitly returns `null`.

### Printing

The `write()` function is used to output values to the console or another output stream.

```cylixin
write("Welcome to Cylixin!");
let name = "Developer";
write("Hello,", name); // Multiple arguments will be printed, likely with a space
let error = 404;
write("Error statement: " + error); // String concatenation using '+'
let value = 10;
write("The value is: " + value + ". Isn't it?"); // Multiple concatenations
let pi = 3.14159;
write("The value of pi is: " + pi);
```
The `write()` function is designed to output strings. When you provide non-string arguments, the `write()` function will automatically convert them to their string form. This is particularly useful when using the `+` operator for string concatenation within the `write()` call to create formatted output.

## Expressions

Expressions are parts of code that evaluate to a value.

```cylixin
let calculation = (5 * 2) + 1; // Arithmetic expression
let comparison = age >= 18; // Comparison expression
let logicalResult = true and false; // Logical expression
let greeting = @greet("World"); // Function call expression
```
Expressions are used in various contexts, such as assignments, conditions in control flow statements, and arguments to function calls.















