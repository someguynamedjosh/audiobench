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
    let program = compiler.compile("arithmetic.ns").unwrap();
    let mut inputs: Inputs = Default::default();
    let mut outputs: Outputs = Default::default();
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
            inputs.a = a;
            inputs.b = b;
            program
                .execute_data(&mut inputs, &mut outputs, &mut static_data)
                .unwrap();
            assert!(outputs.sum == a + b);
            assert!(outputs.difference == a - b);
            assert!(outputs.product == a * b);
            assert!(outputs.fraction == a / b);
            assert!(outputs.remainder == a % b);
            assert!(outputs.sin == a.sin());
            assert!(outputs.cos == a.cos());
            if a > 0.0 {
                assert!(outputs.sqrt == a.sqrt());
            }
            assert!(outputs.abs == a.abs());
            assert!(outputs.floor == a.floor());
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
        let program = match compiler.compile(&name) {
            Ok(program) => program,
            Err(message) => panic!("Failed to compile {}:\n{}", &name, message),
        };
        unsafe {
            let mut static_data = program.create_static_data().unwrap();
            let mut in_dat = Vec::new();
            let mut out_dat = Vec::new();
            program.execute_raw(&mut in_dat[..], &mut out_dat[..], &mut static_data).unwrap();
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
