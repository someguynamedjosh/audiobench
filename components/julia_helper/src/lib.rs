pub use jlrs::prelude::*;

use scones::make_constructor;
use shared_util::{Clip, Position};

const JULIA_DELIMS: &'static [(&'static str, &'static str)] = &[
    ("function", "end"),
    ("if", "end"),
    ("for", "end"),
    ("while", "end"),
    ("try", "end"),
    ("begin", "end"),
];

#[make_constructor]
#[derive(Clone, Debug)]
pub struct FilePosition {
    pub filename: String,
    pub position: Position,
}

impl<'a> From<&'a str> for FilePosition {
    fn from(other: &'a str) -> Self {
        Self::new(other.to_owned(), Position::new_start())
    }
}

/// Represents a segment of Julia code loaded from a file, which can be broken down into smaller
/// pieces.
#[make_constructor((.., data: String))]
#[derive(Clone, Debug)]
pub struct FileClip {
    filename: String,
    #[value(data.into())]
    data: Clip,
}

impl FileClip {
    pub fn clip_section(&self, start: &str, end: &str) -> Option<(FileClip, (FileClip, FileClip))> {
        let mut searcher = self.data.search();

        let clip_start = searcher.start_clip();
        searcher.goto_pattern_start(start);
        if searcher.at_end() {
            return None;
        }
        let before = searcher.end_clip(clip_start);

        searcher.skip_n(start.len());
        if searcher.at_end() {
            return None;
        }
        let clip_start = searcher.start_clip();
        searcher.skip_blocks(JULIA_DELIMS, end);
        // We should still have at least `end.len()` characters remaining.
        if searcher.at_end() {
            return None;
        }
        let clip = searcher.end_clip(clip_start);

        searcher.skip_n(end.len());
        let clip_start = searcher.start_clip();
        searcher.goto_end();
        let after = searcher.end_clip(clip_start);

        let clip = FileClip {
            filename: self.filename.clone(),
            data: clip,
        };
        let before = FileClip {
            filename: self.filename.clone(),
            data: before,
        };
        let after = FileClip {
            filename: self.filename.clone(),
            data: after,
        };
        Some((clip, (before, after)))
    }

    pub fn get_position(&self) -> FilePosition {
        FilePosition::new(self.filename.clone(), self.data.get_source_position())
    }

    pub fn contains(&self, pattern: &str) -> bool {
        self.data.as_str().contains(pattern)
    }
}

/// Represents a segment of Julia code assemblied from zero or more smaller pieces. Keeps track of
/// the sources of individual pieces so that stack traces can be mapped back to their original
/// sources.
#[make_constructor]
#[derive(Clone, Debug)]
pub struct GeneratedCode {
    /// Where different pieces of the code start, 0 = pos in catted_source, 1 = original pos.
    #[value(Vec::new())]
    code_map: Vec<(Position, FilePosition)>,
    /// What line/col the code currently ends on.
    #[value(Position::new_start())]
    current_end: Position,
    /// The complete code with all pieces appended together.
    #[value(String::new())]
    catted_source: String,
}

impl GeneratedCode {
    pub fn from_unique_source(source_name: &str, content: &str) -> Self {
        let mut this = Self::new();
        this.append(content, source_name);
        this
    }

    pub fn append(&mut self, code: &str, from: impl Into<FilePosition>) {
        let from = from.into();
        let start = self.current_end;
        self.code_map.push((start, from));
        self.current_end = self.current_end.after_str(&code);
        self.catted_source.push_str(&code);
    }

    pub fn append_clip(&mut self, snippet: &FileClip) {
        self.append(&snippet.data.as_str(), snippet.get_position());
    }

    pub fn as_str(&self) -> &str {
        &self.catted_source
    }
}

#[macro_export]
macro_rules! include_packed_library {
    ($name:literal) => {{
        const CODE: &'static str = include_str!(concat!(
            env!("PROJECT_ROOT"),
            "/dependencies/julia_packages/",
            $name,
            ".jl"
        ));
        GeneratedCode::from_unique_source(concat!("packed/", $name, "/__entry__.jl"), CODE)
    }};
}

/// Creating more than one of these at a time will raise a panic.
pub struct ExecutionEngine {
    julia: Julia,
    global_code_segments: Vec<GeneratedCode>,
}

// If you get random segfaults this might need to be bigger.
const STACK_SIZE: usize = 8192;
/// Code to run a function and return any produced exceptions as a string including a backtrace
/// instead of just the raw exception argument.
const EE_ENV: &'static str = r#"
function __error_format_helper__(fn_to_run, arguments...)
    try
        fn_to_run(arguments...)
    catch error
        # Turn the error into a string describing the error (including a backtrace)
        # https://discourse.julialang.org/t/is-it-possible-to-get-an-exception-message-as-a-string/3201/4
        bt = catch_backtrace()
        new_error = sprint(showerror, error, bt)
        throw(new_error)
    end
end

function __load_code_helper__(code, filename)
    try
        Main.include_string(Main, code, filename)
    catch error
        bt = catch_backtrace()
        throw(sprint(showerror, error, bt))
    end
end

module UnpackedDependencies end
"#;

impl ExecutionEngine {
    pub fn new() -> Self {
        const ERR: &'static str =
            "Tried to create an ExecutionEngine while Julia was already running!";
        let mut this = Self {
            julia: unsafe { Julia::init(STACK_SIZE).expect(ERR) },
            global_code_segments: Vec::new(),
        };
        let env_code = GeneratedCode::from_unique_source("__execution_engine__", EE_ENV);
        // We can't use add_global_code yet because it relies on code from EE_ENV.
        this.julia
            .dynamic_frame(|global, frame| {
                let code_str = Value::new(frame, EE_ENV).unwrap();
                let tracker_filename = Value::new(frame, "__global_code_0__.jl").unwrap();
                let main_module = Module::main(global);
                let include_string = Module::base(global).function("include_string").unwrap();
                include_string
                    .call3(frame, main_module.as_value(), code_str, tracker_filename)
                    .unwrap()
                    .unwrap();
                Ok(())
            })
            .unwrap();
        this.global_code_segments.push(env_code);
        this
    }

    /// Executes the specified code clip in the global scope such that it will affect the
    /// execution of all Julia code executed after this call.
    pub fn add_global_code(&mut self, code: GeneratedCode) -> Result<(), String> {
        let tracker_filename = format!("__global_code_{}__.jl", self.global_code_segments.len());
        let res = self
            .julia
            .frame(STACK_SIZE - 10, |global, frame| {
                let code_str = Value::new(frame, code.as_str()).unwrap();
                let tracker_filename = Value::new(frame, tracker_filename).unwrap();
                let main_module = Module::main(global);
                let include_helper = main_module.function("__load_code_helper__").unwrap();
                Ok(include_helper
                    .call2(frame, code_str, tracker_filename)
                    .unwrap())
            })
            .unwrap();
        self.global_code_segments.push(code);
        match res {
            Ok(..) => Ok(()),
            Err(err) => Err(Self::format_error(err, &self.global_code_segments[..])),
        }
    }

    fn format_error(error: Value, segments: &[GeneratedCode]) -> String {
        let raw_error = error
            .cast::<JuliaString>()
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned();
        let mut error = &raw_error[..];
        let mut result = String::new();
        while let Some(index) = error.find("__global_code_") {
            let before = &error[..index];
            result.push_str(before);
            let file_start = index + "__global_code_".len();
            let file_end = file_start + (&error[file_start..]).find("__.jl:").unwrap();
            let file = (&error[file_start..file_end]).parse::<usize>().unwrap();
            let line_start = file_end + "__.jl:".len();
            let line_end = line_start
                + (&error[line_start..])
                    .find("\n")
                    .unwrap_or(error.len() - line_start);
            let line = (&error[line_start..line_end])
                .trim()
                .parse::<usize>()
                .unwrap();
            let source_segment = &segments[file];
            assert!(source_segment.code_map.len() > 0);
            let mut candidate = source_segment.code_map[0].clone();
            for (gen, og) in &source_segment.code_map {
                if gen.line > line {
                    break;
                } else {
                    candidate = (gen.clone(), og.clone());
                }
            }
            let real_line = line + candidate.1.position.line - candidate.0.line;
            result.push_str(&format!("{}:{}", candidate.1.filename, real_line));
            error = &error[line_end..];
        }
        result.push_str(&error);
        result
    }

    /// Calls a Julia function, passing the output to a provided function. (It cannot outlive that
    /// function in the general case, though specific data types may allow this.)
    pub fn call_fn<O, IF, OF>(
        &mut self,
        path: &[&str],
        make_inputs: IF,
        convert_result: OF,
    ) -> Result<O, String>
    where
        IF: for<'f> FnOnce(
            &mut StaticFrame<'f, jlrs::mode::Sync>,
            &mut Vec<Value<'f, 'f>>,
        ) -> JlrsResult<()>,
        OF: for<'f> FnOnce(&mut StaticFrame<'f, jlrs::mode::Sync>, Value<'f, 'f>) -> JlrsResult<O>,
    {
        let Self {
            julia,
            global_code_segments,
            ..
        } = self;
        let r = julia.frame(STACK_SIZE - 10, |global, frame| {
            let mut module = Module::main(global);
            let wrapper = module.function("__error_format_helper__").unwrap();
            let path_len = path.len();
            for submodule_name in &path[..path_len - 1] {
                let m = module.submodule(*submodule_name);
                module = match m {
                    Ok(v) => v,
                    Err(..) => {
                        return Ok(Err(format!(
                            "ERROR: There is no module named {}.",
                            submodule_name
                        )))
                    }
                };
            }
            let func = module.function(path[path_len - 1]);
            let func = match func {
                Ok(v) => v,
                Err(..) => {
                    return Ok(Err(format!(
                        "ERROR: There is no function named {} in the module.",
                        path[path_len - 1]
                    )))
                }
            };
            let mut inputs = Vec::new();
            inputs.push(func);
            if let Err(err) = make_inputs(frame, &mut inputs) {
                return Ok(Err(format!(
                    "ERROR: Failed to create inputs to Julia, caused by:\nERROR: {:?}",
                    err
                )));
            };
            let result = wrapper.call(frame, &mut inputs).unwrap();
            Ok(match result {
                Ok(value) => convert_result(frame, value).map_err(|_err| {
                    format!(
                        "ERROR: Function returned unexpected type {}.",
                        value.type_name()
                    )
                }),
                Err(err) => Err(Self::format_error(err, &global_code_segments[..])),
            })
        });
        r.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clips() {
        let mut code = GeneratedCode::new();
        let clip = FileClip::new(
            "Test.txt".to_owned(),
            r#"Hello world!
Scope Start
    Here is a scope where things go
Scope End"#
                .to_owned(),
        );
        code.append_clip(&clip.clip_section("Scope Start", "Scope End").unwrap());
        assert_eq!(code.as_str(), "\n    Here is a scope where things go\n");
    }

    const TEST_CODE: &'static str = r#"
        begin
            increment(x) = x + one(x)
            struct InData
                a::Int32
                b::Int32
            end
            struct OutData
                sum::Int32
                product::Int32
            end
            domath(x::InData) = OutData(x.a + x.b, x.a * x.b)
            function throw_error(a)
                @assert false
            end
        end
    "#;

    #[test]
    fn run_code() {
        let mut ee = ExecutionEngine::new();
        let mut code = GeneratedCode::new();
        code.append_clip(&FileClip::new(
            "test_code.jl".to_owned(),
            TEST_CODE.to_owned(),
        ));
        ee.add_global_code(code).unwrap();

        let value = ee.call_fn(
            &["Main", "increment"],
            |frame, args| {
                args.push(Value::new(frame, 12i32)?);
                Ok(())
            },
            |_, v| v.cast::<i32>(),
        );
        assert_eq!(value.unwrap(), 13);

        #[repr(C)]
        #[derive(Clone, Copy, JuliaStruct, IntoJulia)]
        #[jlrs(julia_type = "Main.InData")]
        struct InData {
            a: i32,
            b: i32,
        }
        #[repr(C)]
        #[derive(Clone, Copy, JuliaStruct)]
        #[jlrs(julia_type = "Main.OutData")]
        struct OutData {
            sum: i32,
            product: i32,
        }

        let input = InData { a: 3, b: 5 };
        let output = ee
            .call_fn(
                &["Main", "domath"],
                |frame, args| {
                    args.push(Value::new(frame, input)?);
                    Ok(())
                },
                |_, v| v.cast::<OutData>(),
            )
            .unwrap();
        assert_eq!(output.sum, 8);
        assert_eq!(output.product, 15);

        let error = ee
            .call_fn(
                &["Main", "throw_error"],
                |frame, args| {
                    args.push(Value::new(frame, input)?);
                    Ok(())
                },
                |_, _| Ok(()),
            )
            .unwrap_err();
        assert!(error.contains("throw_error"));
        assert!(error.contains("__global_code_0__.jl:4"));
        assert!(error.contains("__global_code_1__.jl:14"));

        let lib = include_packed_library!("StaticArrays");
        ee.add_global_code(lib).unwrap();
        let code = GeneratedCode::from_unique_source(
            "asdf",
            "using Main.UnpackedDependencies.StaticArrays",
        );
        ee.add_global_code(code).unwrap();
        let code = GeneratedCode::from_unique_source("asdf", "SA_F32[1, 2, 3]");
        ee.add_global_code(code).unwrap();
        let code = GeneratedCode::from_unique_source("asdf", "SA_F32[1, 2, 3][4]");
        let error = ee.add_global_code(code).unwrap_err();
        assert!(error.contains("packed/StaticArrays/SVector.jl:40"));
        assert!(error.contains("__global_code_0__.jl:15"));
    }
}
