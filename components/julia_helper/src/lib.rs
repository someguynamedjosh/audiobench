use jlrs::prelude::*;
use jlrs::traits::IntoJulia;
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
    pub fn clip_section(&self, start: &str, end: &str) -> Option<FileClip> {
        let mut searcher = self.data.search();
        searcher.goto_pattern_end(start);
        if searcher.at_end() {
            return None;
        }
        let clip_start = searcher.start_clip();
        searcher.skip_blocks(JULIA_DELIMS, end);
        // We should still have at least `end.len()` characters remaining.
        if searcher.at_end() {
            return None;
        }
        let data = searcher.end_clip(clip_start);
        Some(FileClip {
            filename: self.filename.clone(),
            data,
        })
    }

    pub fn get_position(&self) -> FilePosition {
        FilePosition::new(self.filename.clone(), self.data.get_source_position())
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
        this.append(content, source_name.into());
        this
    }

    pub fn append(&mut self, code: &str, from: FilePosition) {
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

/// Creating more than one of these at a time will raise a panic.
pub struct ExecutionEngine {
    julia: Julia,
    global_code_segments: Vec<GeneratedCode>,
}

const STACK_SIZE: usize = 128;
/// Code to run a function and return any produced exceptions as a string including a backtrace
/// instead of just the raw exception argument.
const RUNNER: &'static str = r#"
using Base.StackTraces
function __error_format_helper__(fn_to_run, argument)
    try
        fn_to_run(argument)
    catch error
        # Turn the error into a string describing the error (including a backtrace)
        # https://discourse.julialang.org/t/is-it-possible-to-get-an-exception-message-as-a-string/3201/4
        bt = catch_backtrace()
        throw(sprint(showerror, error, bt))
    end
end
"#;

impl ExecutionEngine {
    pub fn new() -> Self {
        const ERR: &'static str =
            "Tried to create an ExecutionEngine while Julia was already running!";
        let mut this = Self {
            julia: unsafe { Julia::init(STACK_SIZE).expect(ERR) },
            global_code_segments: Vec::new(),
        };
        let runner_code = GeneratedCode::from_unique_source("__execution_engine__", RUNNER);
        this.add_global_code(runner_code).unwrap();
        this
    }

    /// Executes the specified code clip in the global scope such that it will affect the
    /// execution of all Julia code executed after this call.
    pub fn add_global_code(&mut self, code: GeneratedCode) -> JlrsResult<()> {
        let tracker_filename = format!("__global_code_{}.jl", self.global_code_segments.len());
        self.julia.dynamic_frame(|global, frame| {
            let code_str = Value::new(frame, code.as_str())?;
            let tracker_filename = Value::new(frame, tracker_filename)?;
            let main_module = Module::main(global);
            let include_string = Module::base(global).function("include_string")?;
            include_string
                .call3(frame, main_module.as_value(), code_str, tracker_filename)?
                .unwrap();
            Ok(())
        })?;
        self.global_code_segments.push(code);
        Ok(())
    }

    fn format_error(error: Value) -> String {
        let err = error.cast::<String>().unwrap();
        err
    }

    /// Calls a Julia function, passing the output to a provided function. (It cannot outlive that
    /// function in the general case, though specific data types may allow this.)
    pub fn call_fn<I, O, F>(
        &mut self,
        path: &[&str],
        input: I,
        convert_result: F,
    ) -> Result<O, String>
    where
        I: IntoJulia,
        F: for<'f, 'd> FnOnce(Value<'f, 'd>) -> JlrsResult<O>,
    {
        let r = self.julia.frame(STACK_SIZE - 10, |global, frame| {
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
            let input = Value::new(frame, input);
            let input = match input {
                Ok(v) => v,
                Err(err) => {
                    return Ok(Err(format!(
                        concat!(
                            "ERROR: Failed to convert the provided value before sending it ",
                            "to Julia, caused by:\nERROR: {:?}"
                        ),
                        err
                    )))
                }
            };
            let result = wrapper.call(frame, &mut [func, input]).unwrap();
            Ok(match result {
                Ok(value) => convert_result(value).map_err(|_err| {
                    format!(
                        "ERROR: Function returned unexpected type {}.",
                        value.type_name()
                    )
                }),
                Err(err) => Err(Self::format_error(err)),
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

        let value = ee.call_fn(&["Main", "increment"], 12i32, |v| v.cast::<i32>());
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
            .call_fn(&["Main", "domath"], input, |v| v.cast::<OutData>())
            .unwrap();
        assert_eq!(output.sum, 8);
        assert_eq!(output.product, 15);

        let error = ee
            .call_fn(&["Main", "throw_error"], input, |_| Ok(()))
            .unwrap_err();
        println!("{}", error);
        assert!(error.contains("throw_error"));
        assert!(error.contains("__global_code_0.jl:5"));
        assert!(error.contains("__global_code_1.jl:14"));
    }
}
