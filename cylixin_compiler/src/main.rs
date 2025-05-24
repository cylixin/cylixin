// src/main.rs

// Declare our modules. This tells Rust that `token.rs` and `lexer.rs` exist
// and contain public items that we want to use here.
mod token;
mod lexer;

fn main() {
    // This is a sample Cylixin code string to test our lexer.
    // The `r#""#` syntax creates a raw string literal, which is useful for
    // multi-line strings and avoids needing to escape backslashes (like in `\n`).
    let source_code = r#"
// This is a single-line comment.
/* This is a
multi-line comment
that spans multiple lines. */
let myVar = 123;
var globalCount = 3.14;
const PI = 3.14159; // Using our new 'const' keyword!
if myVar >= 10 and PI <= 4.0 then
    write("Hello, Cylixin!");
    write("Sum: " + (10 + 5));
endif

fun calculate_area(radius) then
    let area = PI * radius ** 2;
    return area;
endfun

let result = @calculate_area(5.0);
write("Area:", result);

let mySet = {1, 2, 3}; // Placeholder for set literal - lexer will recognize { } , and numbers
let myDic = {"key": "value", "another": 42}; // Placeholder for dict literal
let booleanVal = true;
let emptyCheck = !empty; // Testing '!empty' keyword

for i = 0; i < 5; i += 1 then
    if i == 3 then
        break; // Basic break
    endif
endfor

outer_loop: for j = 0; j < 2; j += 1 then
    inner_loop: while true then
        write("Inner loop j:", j);
        if j == 1 then
            break outer_loop; // Breaking out of a labeled loop!
        endif
        break; // Break inner_loop
    endwhile
endfor

let nanValue = NaN; // Testing NaN keyword
    "#;

    // Create a new instance of our Lexer.
    // We pass a reference (`&source_code`) because the Lexer now takes a string slice.
    let mut lexer = lexer::Lexer::new(&source_code);

    println!("Starting Cylixin Lexer...");
    println!("--------------------------");

    // Loop indefinitely, calling `next_token` to get tokens
    // until we reach the End Of File (EOF) token or an Error token.
    loop {
        let token = lexer.next_token();
        // Print the token using its Debug implementation (`{:?}`).
        println!("{:?}", token);

        // Check if we've reached the end of the file or encountered an error.
        if token.kind == token::TokenKind::EOF || matches!(token.kind, token::TokenKind::Error(_)) {
            break; // Exit the loop.
        }
    }

    println!("--------------------------");
    println!("Lexing complete.");
}