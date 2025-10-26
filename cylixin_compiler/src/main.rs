mod lexer;
mod token;
mod parser;
mod ast;

use lexer::Lexer;
use parser::Parser;

fn main() {
    println!("Starting Cylixin Parser...");
    println!("--------------------------");

//     let source_code = r#"
// let myVar = 123;
// var globalCount = 3.14;
// const PI = 3.14159;
// if myVar >= 10 and PI <= 4.0 then // This line won't parse yet, but it's in the source
//     write("Hello, Cylixin!");
//     write("Sum: " + (10 + 5));
// endif
// fun calculate_area(radius) then
//     let area = PI * radius ** 2;
//     return area;
// endfun
// let result = @calculate_area(5.0);
// write("Area:", result);
// let mySet = {1, 2, 3};
// let myDic = {"key": "value", "another": 42};
// let booleanVal = true;
// let emptyCheck = !empty; // Test !empty
//     "#;

    let source_code = r#"
let myVar = 123
var globalCount = 3.14
const PI = 3.14159
if myVar >= 10 and PI <= 4.0 then // This line won't parse yet, but it's in the source
    write("Hello, Cylixin!")
    write("Sum: " + (10 + 5))
endif
fun calculate_area(radius) then
    let area = PI * radius ** 2
    return area
endfun
let result = @calculate_area(5.0);
write("Area:", result)
let mySet = {1, 2, 3}
let myDic = {"key": "value", "another": 42}
let booleanVal = true
let emptyCheck = !empty // Test !empty
    "#;

    let lexer = Lexer::new(source_code);
    let mut parser = Parser::new(lexer);

    println!("Attempting to parse the entire program...");
    let program_statements = parser.parse_program(); // Call parse_program

    println!("\nParsed Program AST:");
    for stmt in program_statements {
        println!("{:?}", stmt); // Print each statement for inspection
    }

    if !parser.get_errors().is_empty() {
        println!("\nParser Errors:");
        for err in parser.get_errors() {
            println!("{}", err);
        }
    } else {
        println!("\nParsing completed with no errors.");
    }

    println!("--------------------------");
    println!("Parsing setup complete.");
}