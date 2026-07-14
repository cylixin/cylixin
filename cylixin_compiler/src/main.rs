mod lexer;
mod ast;
mod parser;
mod codegen;

use lexer::Lexer;
use parser::Parser;
use codegen::Compiler;
use inkwell::context::Context;
use std::path::{Path, PathBuf};
use std::process::Command;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    match args[1].as_str() {
        "build" => cmd_build(&args[2..]),
        "emit"  => cmd_emit(&args[2..]),
        "version" | "--version" | "-v" => {
            println!("cylixin {}", VERSION);
        }
        "help" | "--help" | "-h" => print_usage(),
        other => {
            // If someone just runs `cylixin file.cyx`, treat it as `build`
            if other.ends_with(".cyx") {
                cmd_build(&args[1..]);
            } else {
                eprintln!("✗ Unknown command: {}", other);
                print_usage();
                std::process::exit(1);
            }
        }
    }
}

fn print_usage() {
    eprintln!("Cylixin Compiler v{}\n", VERSION);
    eprintln!("Usage:");
    eprintln!("  cylixin build <file.cyx> [-o <output>]   Compile and link an executable");
    eprintln!("  cylixin emit  <file.cyx> [-o <output>]   Emit LLVM IR only (no linking)");
    eprintln!("  cylixin <file.cyx>                       Shorthand for `build`");
    eprintln!("  cylixin version                          Show version");
    eprintln!("  cylixin help                             Show this message");
}

/// Parse `-o <name>` from the tail of an argument slice.
/// Returns (source_file, output_name_override).
fn parse_file_args(args: &[String]) -> (String, Option<String>) {
    if args.is_empty() {
        eprintln!("✗ No source file specified.");
        print_usage();
        std::process::exit(1);
    }

    let source_file = args[0].clone();
    let mut output = None;

    let mut i = 1;
    while i < args.len() {
        if args[i] == "-o" {
            if i + 1 >= args.len() {
                eprintln!("✗ -o requires an output name.");
                std::process::exit(1);
            }
            output = Some(args[i + 1].clone());
            i += 2;
        } else {
            eprintln!("✗ Unknown option: {}", args[i]);
            std::process::exit(1);
        }
    }

    (source_file, output)
}

/// Read source, lex, parse, codegen → returns the LLVM IR string.
fn compile_source(path: &str) -> String {
    // Read source file
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("✗ Could not read '{}': {}", path, e);
            std::process::exit(1);
        }
    };

    let filename = Path::new(path).file_name().unwrap_or_default().to_string_lossy();
    println!("=== Cylixin Compiler ===");
    println!("  Source: {}\n", filename);

    // Step 1: Lex
    let tokens = match Lexer::new(&source).tokenize() {
        Ok(t)  => t,
        Err(e) => { eprintln!("✗ Lexer error in {}: {}", filename, e); std::process::exit(1); }
    };
    println!("  ✓ Lexed {} tokens", tokens.len());

    // Step 2: Parse
    let ast = match Parser::new(tokens).parse() {
        Ok(a)  => a,
        Err(e) => { eprintln!("✗ Parser error in {}: {}", filename, e); std::process::exit(1); }
    };
    println!("  ✓ Parsed {} top-level statements", ast.body.len());

    // Step 3: Compile to LLVM IR
    let context = Context::create();
    let mut compiler = Compiler::new(&context);
    let ir = match compiler.compile(&ast) {
        Ok(ir) => ir,
        Err(e) => { eprintln!("✗ Codegen error in {}: {}", filename, e); std::process::exit(1); }
    };
    println!("  ✓ Generated LLVM IR\n");

    ir
}

/// Locate the runtime.c file.  Search order:
///   1. Next to the source file
///   2. Next to the compiler executable
///   3. In the project root (one level up from the executable in a cargo layout)
fn find_runtime(source_path: &str) -> PathBuf {
    // 1. Same directory as the source file
    let source_dir = Path::new(source_path).parent().unwrap_or(Path::new("."));
    let candidate = source_dir.join("runtime.c");
    if candidate.exists() { return candidate; }

    // 2. Next to the compiler binary
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let candidate = exe_dir.join("runtime.c");
            if candidate.exists() { return candidate; }
            // 3. Cargo layout: target/debug/../.. = project root
            let candidate = exe_dir.join("../../runtime.c");
            if candidate.exists() { return candidate; }
        }
    }

    // 4. CWD
    let candidate = PathBuf::from("runtime.c");
    if candidate.exists() { return candidate; }

    eprintln!("✗ Could not find runtime.c — looked next to source, executable, and CWD.");
    eprintln!("  Place runtime.c in the same directory as your .cyx file, or in the compiler directory.");
    std::process::exit(1);
}

fn cmd_emit(args: &[String]) {
    let (source_file, output_override) = parse_file_args(args);
    let ir = compile_source(&source_file);

    let stem = Path::new(&source_file).file_stem().unwrap_or_default().to_string_lossy();
    let ir_path = output_override.unwrap_or_else(|| format!("{}.ll", stem));

    std::fs::write(&ir_path, &ir).unwrap_or_else(|e| {
        eprintln!("✗ Failed to write '{}': {}", ir_path, e);
        std::process::exit(1);
    });
    println!("  ✓ Written to {}", ir_path);
}

fn cmd_build(args: &[String]) {
    let (source_file, output_override) = parse_file_args(args);
    let ir = compile_source(&source_file);

    let stem = Path::new(&source_file).file_stem().unwrap_or_default().to_string_lossy();
    let ir_path = format!("{}.ll", stem);
    let exe_path = output_override.unwrap_or_else(|| stem.to_string());

    // Write IR
    std::fs::write(&ir_path, &ir).unwrap_or_else(|e| {
        eprintln!("✗ Failed to write '{}': {}", ir_path, e);
        std::process::exit(1);
    });
    println!("  ✓ Written to {}", ir_path);

    // Find runtime.c
    let runtime_path = find_runtime(&source_file);
    println!("  ✓ Using runtime: {}", runtime_path.display());

    // Invoke clang
    print!("  → Linking with clang...");
    let status = Command::new("clang")
        .args([
            "-Wno-override-module",
            "-mllvm", "-opaque-pointers",
            &ir_path,
            &runtime_path.to_string_lossy().to_string(),
            "-o",
            &exe_path,
            "-lm",
        ])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!(" done");
            println!("\n  ✓ Built: ./{}\n", exe_path);
            // Clean up the .ll file after successful linking
            let _ = std::fs::remove_file(&ir_path);
        }
        Ok(s) => {
            println!(" failed");
            eprintln!("\n✗ clang exited with status {}", s);
            eprintln!("  The LLVM IR has been kept at '{}' for inspection.", ir_path);
            std::process::exit(1);
        }
        Err(e) => {
            println!(" failed");
            eprintln!("\n✗ Could not run clang: {}", e);
            eprintln!("  Make sure clang is installed and in your PATH.");
            eprintln!("  You can link manually: clang {} {} -o {} -lm",
                ir_path, runtime_path.display(), exe_path);
            std::process::exit(1);
        }
    }
}
