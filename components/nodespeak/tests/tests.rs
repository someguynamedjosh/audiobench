extern crate nodespeak;

#[test]
fn arithmetic() {
    #[repr(C)]
    #[derive(Default)]
    struct Inputs {
        a: f32,
        b: f32,
    }

    #[repr(C)]
    #[derive(Default)]
    struct Outputs {
        sum: f32,
        difference: f32,
        product: f32,
        fraction: f32,
        remainder: f32,
        sin: f32,
        cos: f32,
        sqrt: f32,
        abs: f32,
        floor: f32,
    }

    let mut compiler = nodespeak::Compiler::new();
    compiler.add_source(
        "arithmetic.ns".to_owned(),
        include_str!("arithmetic.ns").to_owned(),
    );
    let mut program = compiler.compile("arithmetic.ns").unwrap();
    unsafe {
        let mut static_data = program.create_static_data().unwrap();
        for (a, b) in &[
            (0.0, 1.0),
            (99.0, 32.0),
            (0.1, 0.3),
            (-3.0, 10.0),
            (1e-3, 1e8),
            (-3e5, 10.0),
        ] {
            let (a, b) = (*a, *b);
            let inputs = program.borrow_input_packer_mut();
            inputs.set_argument(0, a.into());
            inputs.set_argument(1, b.into());
            program.execute(&mut static_data).unwrap();
            let outputs = program.borrow_output_unpacker();
            assert!(outputs.get_argument(0).unwrap_float() == a + b);
            assert!(outputs.get_argument(1).unwrap_float() == a - b);
            assert!(outputs.get_argument(2).unwrap_float() == a * b);
            assert!(outputs.get_argument(3).unwrap_float() == a / b);
            assert!(outputs.get_argument(4).unwrap_float() == a % b);
            assert!(outputs.get_argument(5).unwrap_float() == a.sin());
            assert!(outputs.get_argument(6).unwrap_float() == a.cos());
            if a > 0.0 {
                assert!(outputs.get_argument(7).unwrap_float() == a.sqrt());
            }
            assert!(outputs.get_argument(8).unwrap_float() == a.abs());
            assert!(outputs.get_argument(9).unwrap_float() == a.floor());
        }
    };
}

#[test]
fn compile_ok() {
    for entry in std::fs::read_dir("tests/compile_ok/").unwrap() {
        let path = if let Ok(entry) = entry {
            entry.path()
        } else {
            continue;
        };
        let name = path.to_str().unwrap().to_owned();
        let mut compiler = nodespeak::Compiler::new();
        compiler.add_source(name.clone(), std::fs::read_to_string(&name).unwrap());
        if let Err(message) = compiler.compile(&name) {
            panic!("Failed to compile {}:\n{}", &name, message);
        }
    }
}

#[test]
fn assert_ok() {
    for entry in std::fs::read_dir("tests/assert_ok/").unwrap() {
        let path = if let Ok(entry) = entry {
            entry.path()
        } else {
            continue;
        };
        let name = path.to_str().unwrap().to_owned();
        let mut compiler = nodespeak::Compiler::new();
        compiler.add_source(name.clone(), std::fs::read_to_string(&name).unwrap());
        let mut program = match compiler.compile(&name) {
            Ok(program) => program,
            Err(message) => panic!("Failed to compile {}:\n{}", &name, message),
        };
        unsafe {
            let mut static_data = program.create_static_data().unwrap();
            program.execute(&mut static_data).unwrap();
        }
    }
}

#[test]
fn compile_err() {
    for entry in std::fs::read_dir("tests/compile_err/").unwrap() {
        let path = if let Ok(entry) = entry {
            entry.path()
        } else {
            continue;
        };
        let name = path.to_str().unwrap().to_owned();
        let mut compiler = nodespeak::Compiler::new();
        let code = std::fs::read_to_string(&name).unwrap();
        compiler.add_source(name.clone(), code.clone());
        if let Ok(..) = compiler.compile(&name) {
            panic!("{} compiled successfully:\n{}", &name, code);
        }
    }
}
