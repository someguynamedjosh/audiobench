extern crate nodespeak;
extern crate text_io;

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: nodespeak [compile|interpret|[phase]] [path to file]");
        eprintln!("compile: compiles the specified file and outputs the result.");
        eprintln!("interpret: interprets the specified file using the built-in resolver.");
        eprintln!("[phase]: runs compilation of the file up until [phase] of compilation.");
        eprintln!("    phases: parse, structure, resolve, trivialize, specialize");
        process::exit(64);
    }

    let mut compiler = nodespeak::Compiler::new();
    if let Some((width, _)) = terminal_size::terminal_size() {
        compiler.set_error_width(width.0 as usize - 1);
    }
    let main_source_name = &args[2];
    if let Err(err) = compiler.add_source_from_file(main_source_name.to_owned()) {
        eprintln!("Could not read from {}:", main_source_name);
        eprintln!("{:?}", err);
        process::exit(74);
    }

    for arg_index in 3..args.len() {
        let source_name = &args[arg_index];
        if let Err(err) = compiler.add_source_from_file(source_name.to_owned()) {
            eprintln!("Could not read from {}:", source_name);
            eprintln!("{:?}", err);
            process::exit(74);
        }
    }

    println!("\nStarting...");
    match args[1].as_ref() {
        "ast" => match compiler.compile_to_ast(main_source_name) {
            Result::Ok(program) => println!("{:#?}", program),
            Result::Err(err) => {
                eprintln!("{}", err);
                process::exit(101);
            }
        },
        #[cfg(not(feature = "no-vague"))]
        "vague" => match compiler.compile_to_vague(main_source_name) {
            Result::Ok(program) => println!("{:?}", program),
            Result::Err(err) => {
                eprintln!("{}", err);
                process::exit(101);
            }
        },
        #[cfg(not(feature = "no-resolved"))]
        "resolved" => match compiler.compile_to_resolved(main_source_name) {
            Result::Ok(program) => println!("{:?}", program),
            Result::Err(err) => {
                eprintln!("{}", err);
                process::exit(101);
            }
        },
        #[cfg(not(feature = "no-trivial"))]
        "trivial" => match compiler.compile_to_trivial(main_source_name) {
            Result::Ok(program) => println!("{:?}", program),
            Result::Err(err) => {
                eprintln!("{}", err);
                process::exit(101);
            }
        },
        #[cfg(not(feature = "no-llvmir"))]
        "llvmir" => match compiler.compile_to_llvmir(main_source_name) {
            Result::Ok(program) => println!("{:?}", program),
            Result::Err(err) => {
                eprintln!("{}", err);
                process::exit(101);
            }
        },
        _ => {
            eprintln!("Invalid mode '{}', expected compile or a phase.", args[1]);
            eprintln!("compile: compiles the specified file and outputs the result.");
            eprintln!("[phase]: runs compilation of the file up until [phase] of compilation.");
            eprintln!("    phases: ast, vague, resolved, trivial, llvmir");
            process::exit(64);
        }
    }
    println!("Task completed sucessfully.");
    println!("{}", compiler.borrow_performance_counters());
}
