#[derive(Debug)]
#[repr(C)]
struct Input {
    a: f32,
}

#[derive(Debug, Default)]
#[repr(C)]
struct Output {
    x: f32,
}

extern crate nodespeak;

fn main() {
    let mut compiler = nodespeak::Compiler::new();
    compiler
        .add_source_from_file("examples/scratch.ns".to_owned())
        .expect("Failed to read file.");

    let compiled = match compiler.compile_to_llvmir("examples/scratch.ns") {
        Ok(program) => program,
        Err(error) => {
            eprintln!("Compile failed:\n{}", error);
            std::process::exit(101);
        }
    };

    let mut input = Input { a: 0.0 };
    let mut output: Output = Default::default();
    let mut static_data = unsafe {
        match compiled.create_static_data() {
            Ok(data) => data,
            Err(description) => {
                eprintln!("Failed to create static data: {}", description);
                std::process::exit(101);
            }
        }
    };

    println!("in:\n{:?}\nout:\n{:?}", input, output);
    for increment in 0..10 {
        println!("\nExecuting..");
        unsafe {
            if let Err(description) =
                compiled.execute_data(&mut input, &mut output, &mut static_data)
            {
                eprintln!("Failed to execute program: {}", description);
            }
        }
        println!("\nin:\n{:?}\nout:\n{:?}", input, output);
    }
}
