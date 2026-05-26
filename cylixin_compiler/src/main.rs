mod lexer;
mod ast;
mod parser;
mod codegen;

use lexer::Lexer;
use parser::Parser;
use codegen::Compiler;
use inkwell::context::Context;

fn main() {
    let source = r#"
fun add(a: int, b: int): int then
    return a + b;
endfun

var result: int = @add(3, 7);
@writeln(result);

var count: int = 0;

for i from 0 to 5 then
    count += i;
endfor

@writeln(count);

if count > 5 then
    @writeln("Sum is big!");
elif count > 3 then
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
    println!("✓ Parsed {} top-level statements", ast.body.len());

    // Step 3: Compile to LLVM IR
    let context = Context::create();
    let mut compiler = Compiler::new(&context);
    let ir = match compiler.compile(&ast) {
        Ok(ir) => ir,
        Err(e) => { eprintln!("Codegen error: {}", e); std::process::exit(1); }
    };
    println!("✓ Generated LLVM IR\n");

    // Write IR to file
    std::fs::write("output.ll", &ir).expect("failed to write output.ll");
    println!("Written to output.ll");
    println!("Run: clang output.ll -o program && ./program\n");
    println!("--- LLVM IR ---\n{}", ir);
}
