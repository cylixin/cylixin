mod lexer;
mod token;
mod parser; // New: import parser
mod ast;    // New: import ast

use lexer::Lexer;
use parser::Parser; // New: use Parser

fn main() {
    println!("Starting Cylixin Parser...");
    println!("--------------------------");

    // Example source code to parse
    let source_code = r#"
let myVar = 123;
var globalCount = 3.14;
const PI = 3.14159;
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
let mySet = {1, 2, 3};
let myDic = {"key": "value", "another": 42};
let booleanVal = true;
let emptyCheck = !empty; // Test !empty
    "#;

    let lexer = Lexer::new(source_code);
    let mut parser = Parser::new(lexer); // Create parser instance

    // For now, let's try to parse just the first expression (123)
    // We'll expand this to parse the whole program later.
    println!("Attempting to parse a single expression (e.g., '123' from 'let myVar = 123;'):");
    if let Some(expr) = parser.parse_expression() {
        println!("Parsed Expression: {:?}", expr);
    } else {
        println!("Failed to parse expression.");
    }

    // Check for any parser errors
    if !parser.get_errors().is_empty() {
        println!("\nParser Errors:");
        for err in parser.get_errors() {
            println!("{}", err);
        }
    }

    println!("--------------------------");
    println!("Parsing setup complete.");
}