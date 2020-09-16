use crate::high_level::problem::CompileProblem;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::time::Instant;

pub const FAKE_BUILTIN_SOURCE: &str = r#"
DATA_TYPE AUTO;
DATA_TYPE BOOL;
DATA_TYPE INT;
DATA_TYPE FLOAT;
DATA_TYPE VOID;
"#;

#[derive(Default)]
pub struct PerformanceCounter {
    pub time: u128,
    pub num_invocations: u32,
}

impl Display for PerformanceCounter {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}ms ({} invocations)",
            self.time, self.num_invocations
        )
    }
}

#[derive(Default)]
pub struct PerformanceCounters {
    pub(crate) ast: PerformanceCounter,
    vague: PerformanceCounter,
    resolved: PerformanceCounter,
    trivial: PerformanceCounter,
    llvmir: PerformanceCounter,
}

impl Display for PerformanceCounters {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        writeln!(formatter, "          Performance")?;
        writeln!(formatter, "     ast: {}", self.ast)?;
        writeln!(formatter, "   vague: {}", self.vague)?;
        writeln!(formatter, "resolved: {}", self.resolved)?;
        writeln!(formatter, " trivial: {}", self.trivial)?;
        write!(formatter, "  llvmir: {}", self.llvmir)
    }
}

pub struct SourceSet {
    sources: Vec<(String, String)>,
    source_indices: HashMap<String, usize>,
}

impl SourceSet {
    fn new() -> Self {
        let mut new = Self {
            sources: Vec::new(),
            source_indices: HashMap::new(),
        };
        new.add_source(
            "(internal code) builtins".to_owned(),
            FAKE_BUILTIN_SOURCE.to_owned(),
        );
        new
    }

    pub fn add_source(&mut self, name: String, content: String) {
        if let Some(existing_index) = self.source_indices.get(&name) {
            self.sources[*existing_index].1 = content;
        } else {
            self.source_indices.insert(name.clone(), self.sources.len());
            self.sources.push((name, content));
        }
    }

    pub fn add_source_from_file(&mut self, file_path: String) -> std::io::Result<()> {
        let file_content = std::fs::read_to_string(&file_path)?;
        self.add_source(file_path, file_content);
        Ok(())
    }

    pub(crate) fn find_source(&self, name: &str) -> Option<usize> {
        self.source_indices.get(name).cloned()
    }

    pub(crate) fn borrow_source(&self, index: usize) -> &(String, String) {
        &self.sources[index]
    }

    fn find_source_err(&self, name: &str) -> Result<&str, String> {
        if let Some(index) = self.find_source(name) {
            Ok(&self.borrow_source(index).1)
        } else {
            Err(format!("Failed to find a source named {}", name))
        }
    }
}

pub struct Compiler {
    source_set: SourceSet,
    performance_counters: PerformanceCounters,
    error_width: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            source_set: SourceSet::new(),
            performance_counters: Default::default(),
            error_width: 80,
        }
    }

    pub fn set_error_width(&mut self, width: usize) {
        self.error_width = width;
    }

    pub fn add_source(&mut self, name: String, content: String) {
        self.source_set.add_source(name, content)
    }

    pub fn add_source_from_file(&mut self, file_path: String) -> std::io::Result<()> {
        self.source_set.add_source_from_file(file_path)
    }

    pub fn borrow_performance_counters(&self) -> &PerformanceCounters {
        &self.performance_counters
    }

    fn format_error<T>(&self, result: Result<T, CompileProblem>) -> Result<T, String> {
        result.map_err(|e| e.format(self.error_width, &self.source_set))
    }

    pub fn compile_to_ast_impl<'a>(
        pc: &mut PerformanceCounters,
        source: &'a str,
        file_id: usize,
    ) -> Result<crate::ast::structure::Program<'a>, CompileProblem> {
        let timer = Instant::now();
        let result = crate::ast::ingest(source, file_id);
        pc.ast.time += timer.elapsed().as_millis();
        pc.ast.num_invocations += 1;
        result
    }

    pub fn compile_to_ast<'a>(
        &'a mut self,
        source_name: &str,
    ) -> Result<crate::ast::structure::Program<'a>, String> {
        let source = self.source_set.find_source_err(source_name)?;
        let source_id = self.source_set.find_source(source_name).unwrap(); // yeah its hacky
        let result = Self::compile_to_ast_impl(&mut self.performance_counters, source, source_id);
        self.format_error(result)
    }

    #[cfg(not(feature = "no-vague"))]
    pub fn compile_to_vague(
        &mut self,
        source_name: &str,
    ) -> Result<crate::vague::structure::Program, String> {
        let source = self.source_set.find_source_err(source_name)?;
        let source_id = self.source_set.find_source(source_name).unwrap(); // woo yay its 2am
        let result = Self::compile_to_ast_impl(&mut self.performance_counters, source, source_id);
        let mut source = self.format_error(result)?;

        let timer = Instant::now();
        let result = crate::vague::ingest(
            &mut source,
            &self.source_set,
            &mut self.performance_counters,
        );
        self.performance_counters.vague.time += timer.elapsed().as_millis();
        self.performance_counters.vague.num_invocations += 1;
        self.format_error(result)
    }

    #[cfg(not(feature = "no-resolved"))]
    pub fn compile_to_resolved(
        &mut self,
        source_name: &str,
    ) -> Result<crate::resolved::structure::Program, String> {
        let mut source = self.compile_to_vague(source_name)?;
        let timer = Instant::now();
        let result = crate::resolved::ingest(&mut source);
        self.performance_counters.resolved.time += timer.elapsed().as_millis();
        self.performance_counters.resolved.num_invocations += 1;
        self.format_error(result)
    }

    #[cfg(not(feature = "no-trivial"))]
    pub fn compile_to_trivial(
        &mut self,
        source_name: &str,
    ) -> Result<crate::trivial::structure::Program, String> {
        let mut source = self.compile_to_resolved(source_name)?;
        let timer = Instant::now();
        let result = crate::trivial::ingest(&mut source, &self.source_set);
        self.performance_counters.trivial.time += timer.elapsed().as_millis();
        self.performance_counters.trivial.num_invocations += 1;
        self.format_error(result)
    }

    #[cfg(not(feature = "no-llvmir"))]
    pub fn compile_to_llvmir(
        &mut self,
        source_name: &str,
    ) -> Result<crate::llvmir::structure::Program, String> {
        let mut source = self.compile_to_trivial(source_name)?;
        let timer = Instant::now();
        let result = crate::llvmir::ingest(&mut source);
        self.performance_counters.llvmir.time += timer.elapsed().as_millis();
        self.performance_counters.llvmir.num_invocations += 1;
        Ok(result)
    }

    #[cfg(not(feature = "no-llvmir"))]
    pub fn compile(
        &mut self,
        source_name: &str,
    ) -> Result<crate::llvmir::structure::Program, String> {
        self.compile_to_llvmir(source_name)
    }
}
