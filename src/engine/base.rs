use crate::engine;
use crate::util::*;
use std::sync::{Mutex};

pub struct Engine {
    // Only read/mutated by UI thread.
    module_graph: Rcrc<engine::parts::ModuleGraph>,
    // Shared.
    /// This value is set to Some() when the audio rendering code should be recompiled.
    new_module_graph_code: Mutex<Option<String>>,
    // Only read/mutated by audio thread.
    registry: engine::registry::Registry,
    executor: engine::execution::ExecEnvironment,
}

impl Engine {
    pub fn new() -> (Self, Result<(), String>) {
        let (registry, base_lib_status) = engine::registry::Registry::new();

        let mut module_graph = engine::parts::ModuleGraph::new();
        if base_lib_status.is_ok() {
            let mut input = registry.borrow_module("base:note_input").unwrap().clone();
            input.pos = (10, 5);
            module_graph.adopt_module(input);
            let mut osc = registry.borrow_module("base:oscillator").unwrap().clone();
            osc.pos = (50, 20);
            module_graph.adopt_module(osc);
            let mut output = registry.borrow_module("base:note_output").unwrap().clone();
            output.pos = (90, 90);
            module_graph.adopt_module(output);
        }

        let mut executor = engine::execution::ExecEnvironment::new(&registry);
        let code = module_graph.generate_code(512).expect("TODO: Nice error.");
        println!("{}", code);

        if let Err(problem) = executor.compile(code) {
            eprintln!("ERROR: Basic setup failed to compile:");
            eprintln!("{}", problem);
            std::process::abort();
        }

        (
            Self {
                module_graph: rcrc(module_graph),
                new_module_graph_code: Mutex::new(None),
                registry,
                executor,
            },
            base_lib_status,
        )
    }

    pub fn borrow_module_graph_ref(&self) -> &Rcrc<engine::parts::ModuleGraph> {
        &self.module_graph
    }

    pub fn mark_module_graph_dirty(&mut self) {
        let new_code = self.module_graph.borrow().generate_code(512).expect("TODO: Nice error.");
        let mut code_ref = self.new_module_graph_code.lock().unwrap();
        *code_ref = Some(new_code);
    }

    pub fn render_audio(&mut self) -> &[f32] {
        let mut new_code = self.new_module_graph_code.lock().unwrap();
        if let Some(code) = new_code.take() {
            self.executor.compile(code).expect("TODO: Nice error.");
        }
        self.executor.execute().expect("TODO: Nice error.")
    }

    pub fn borrow_registry(&self) -> &engine::registry::Registry {
        &self.registry
    }
}
