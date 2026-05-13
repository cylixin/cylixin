mod lexer;
mod ast;
mod parser;

use lexer::Lexer;
use parser::Parser;

fn main() {
    let source = r#"
fun add(a: int, b: int): int then
    return a + b;
endfun

var count: int = 0;

for i from 0 to 5 then
    count += i;
endfor

if count > 10 then
    @writeln("Sum is big!");
elif count > 5 then
    @writeln("Sum is medium.");
else
    @writeln("Sum is small.");
endif
    "#;

    println!("=== Cylixin Compiler ===\n");

    // Step 1: Lex
    let tokens = match Lexer::new(source).tokenize() {
        Ok(t)  => t,
        Err(e) => { eprintln!("Lexer error: {}", e); std::process::exit(1); }
    };
    println!("✓ Lexed {} tokens", tokens.len());

    // Step 2: Parse
    let ast = match Parser::new(tokens).parse() {
        Ok(a)  => a,
        Err(e) => { eprintln!("Parser error: {}", e); std::process::exit(1); }
    };
    println!("✓ Parsed {} top-level statements\n", ast.body.len());
    println!("AST:\n{:#?}", ast);
}
