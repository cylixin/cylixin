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

var power: int = 2 ** 10;
@writeln(power);

var base: int = 3;
base **= 4;
@writeln(base);

fun double_positive(x: int): int then
    var doubled: int = x + x;
    if doubled < 0 then
    endif when (x < 0): -1;
    return doubled;
endfun

var r1: int = @double_positive(5);
@writeln(r1);

var r2: int = @double_positive(-3);
@writeln(r2);

var total: int = 0;
for outer: i from 0 to 5 then
    for j from 0 to 5 then
        if i == 2 then
            if j == 1 then
                break outer;
            endif
        endif
        total += 1;
    endfor
endfor
@writeln(total);

var st1: bool = 5 === 5;
@writeln(st1);

var st2: bool = 5 === 5.0;
@writeln(st2);
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
