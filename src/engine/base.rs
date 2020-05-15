use crate::engine;
use crate::util::*;
use std::sync::Mutex;

const DEFAULT_BUFFER_LENGTH: i32 = 512;
const DEFAULT_SAMPLE_RATE: i32 = 44100;

pub struct Engine {
    // Only read/mutated by UI thread.
    module_graph: Rcrc<engine::parts::ModuleGraph>,
    buffer_length: i32,
    sample_rate: i32,
    // Shared.
    /// This value is set to Some() when the audio rendering code should be recompiled.
    new_module_graph_code: Mutex<Option<String>>,
    // Only read/mutated by audio thread.
    executor: engine::execution::ExecEnvironment,
}

impl Engine {
    pub fn new(registry: &engine::registry::Registry) -> Self {
        let mut module_graph = engine::parts::ModuleGraph::new();
        let mut input = registry.borrow_module("base:note_input").unwrap().clone();
        input.pos = (10, 5);
        module_graph.adopt_module(input);
        let mut env = registry.borrow_module("base:envelope").unwrap().clone();
        env.pos = (300, 100);
        module_graph.adopt_module(env);
        let mut osc = registry.borrow_module("base:oscillator").unwrap().clone();
        osc.pos = (50, 20);
        module_graph.adopt_module(osc);
        let mut osc = registry.borrow_module("base:oscillator").unwrap().clone();
        osc.pos = (50, 200);
        module_graph.adopt_module(osc);
        let mut output = registry.borrow_module("base:note_output").unwrap().clone();
        output.pos = (90, 90);
        module_graph.adopt_module(output);

        let mut executor = engine::execution::ExecEnvironment::new(&registry);
        let code = module_graph
            .generate_code(DEFAULT_BUFFER_LENGTH, DEFAULT_SAMPLE_RATE)
            .expect("TODO: Nice error.");
        println!("{}", code);

        if let Err(problem) =
            executor.compile(code, DEFAULT_BUFFER_LENGTH as usize, DEFAULT_SAMPLE_RATE)
        {
            eprintln!("ERROR: Basic setup failed to compile:");
            eprintln!("{}", problem);
            std::process::abort();
        }

        Self {
            module_graph: rcrc(module_graph),
            buffer_length: DEFAULT_BUFFER_LENGTH,
            sample_rate: DEFAULT_SAMPLE_RATE,
            new_module_graph_code: Mutex::new(None),
            executor,
        }
    }

    pub fn borrow_module_graph_ref(&self) -> &Rcrc<engine::parts::ModuleGraph> {
        &self.module_graph
    }

    pub fn mark_module_graph_dirty(&mut self) {
        let new_code = self
            .module_graph
            .borrow()
            .generate_code(self.buffer_length, self.sample_rate)
            .expect("TODO: Nice error.");
        let mut code_ref = self.new_module_graph_code.lock().unwrap();
        *code_ref = Some(new_code);
    }

    pub fn set_buffer_length_and_sample_rate(&mut self, buffer_length: i32, sample_rate: i32) {
        // Avoid recompiling if there was no change.
        if buffer_length != self.buffer_length || sample_rate != self.sample_rate {
            self.buffer_length = buffer_length;
            self.sample_rate = sample_rate;
            self.mark_module_graph_dirty();
        }
    }

    pub fn note_on(&mut self, index: i32, velocity: f32) {
        self.executor.note_on(index, velocity)
    }

    pub fn note_off(&mut self, index: i32) {
        self.executor.note_off(index)
    }

    pub fn render_audio(&mut self) -> &[f32] {
        let mut new_code = self.new_module_graph_code.lock().unwrap();
        if let Some(code) = new_code.take() {
            println!("{}", code);
            self.executor
                .compile(code, self.buffer_length as usize, self.sample_rate)
                .expect("TODO: Nice error.");
        }
        self.executor.execute().expect("TODO: Nice error.")
    }
}
