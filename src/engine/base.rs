use crate::engine;
use crate::engine::codegen;
use crate::engine::{execution::ExecEnvironment, note_manager::NoteManager};
use crate::util::*;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const DEFAULT_BUFFER_LENGTH: i32 = 512;
const DEFAULT_SAMPLE_RATE: i32 = 44100;
const FEEDBACK_UPDATE_INTERVAL: Duration = Duration::from_millis(50);

struct CrossThreadData {
    // Set by UI thread, read by audio thread.
    buffer_length: i32,
    sample_rate: i32,
    /// This value is set to Some() when the audio rendering code should be recompiled.
    new_module_graph_code: Option<(String, Vec<f32>, usize)>,
    /// This value is set to Some() when the aux input values should change.
    new_aux_input_values: Option<Vec<f32>>,
    /// This value is set to Some() when the audio rendering thread has posted new feedback data.
    new_feedback_data: Option<Vec<f32>>,
    note_manager: NoteManager,
}

impl CrossThreadData {
    fn new() -> Self {
        Self {
            buffer_length: DEFAULT_BUFFER_LENGTH,
            sample_rate: DEFAULT_SAMPLE_RATE,
            new_module_graph_code: None,
            new_aux_input_values: None,
            new_feedback_data: None,
            note_manager: NoteManager::new(),
        }
    }
}

pub struct Engine {
    // Only read/mutated by UI thread.
    module_graph: Rcrc<engine::parts::ModuleGraph>,
    aux_data_collector: codegen::AuxDataCollector,
    feedback_displayer: codegen::FeedbackDisplayer,

    // Shared.
    ctd_mux: Mutex<CrossThreadData>,

    // Only read/mutated by audio thread.
    executor: ExecEnvironment,
    rendered_audio: Vec<f32>,
    last_feedback_data_update: Instant,
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

        let mut executor = ExecEnvironment::new(&registry);
        let gen_result =
            codegen::generate_code(&module_graph, DEFAULT_BUFFER_LENGTH, DEFAULT_SAMPLE_RATE)
                .expect("TODO: Nice error");
        println!("{}", gen_result.code);

        if let Err(problem) = executor.compile(
            gen_result.code,
            gen_result.aux_data_collector.collect_data(),
            DEFAULT_BUFFER_LENGTH as usize,
            DEFAULT_SAMPLE_RATE,
            gen_result.feedback_displayer.get_data_length(),
        ) {
            eprintln!("ERROR: Basic setup failed to compile:");
            eprintln!("{}", problem);
            std::process::abort();
        }

        Self {
            module_graph: rcrc(module_graph),
            aux_data_collector: gen_result.aux_data_collector,
            feedback_displayer: gen_result.feedback_displayer,
            ctd_mux: Mutex::new(CrossThreadData::new()),
            executor,
            rendered_audio: Vec::new(),
            last_feedback_data_update: Instant::now(),
        }
    }

    pub fn borrow_module_graph_ref(&self) -> &Rcrc<engine::parts::ModuleGraph> {
        &self.module_graph
    }

    pub fn reload_structure(&mut self) {
        let mut ctd = self.ctd_mux.lock().unwrap();

        ctd.note_manager.silence_all();
        let module_graph_ref = self.module_graph.borrow();
        let new_gen =
            codegen::generate_code(&*module_graph_ref, ctd.buffer_length, ctd.sample_rate)
                .expect("TODO: Nice error");
        drop(module_graph_ref);
        ctd.new_module_graph_code = Some((
            new_gen.code,
            new_gen.aux_data_collector.collect_data(),
            new_gen.feedback_displayer.get_data_length(),
        ));
        ctd.new_feedback_data = None;
        self.aux_data_collector = new_gen.aux_data_collector;
        self.feedback_displayer = new_gen.feedback_displayer;
    }

    pub fn reload_values(&mut self) {
        let mut ctd = self.ctd_mux.lock().unwrap();

        ctd.new_aux_input_values = Some(self.aux_data_collector.collect_data());
    }

    /// Feedback data is generated on the audio thread. This method uses a mutex to retrieve that
    /// data and copy it so that it can be displayed in the GUI. Nothing will happen if there is no
    /// new data so this is okay to call relatively often. It also does not block on waiting for 
    /// the mutex.
    pub fn display_new_feedback_data(&mut self) {
        if let Ok(mut ctd) = self.ctd_mux.try_lock() {
            if let Some(data) = ctd.new_feedback_data.take() {
                self.feedback_displayer.display_feedback(&data[..]);
            }
        }
    }

    pub fn set_buffer_length_and_sample_rate(&mut self, buffer_length: i32, sample_rate: i32) {
        let mut ctd = self.ctd_mux.lock().unwrap();

        // Avoid recompiling if there was no change.
        if buffer_length != ctd.buffer_length || sample_rate != ctd.sample_rate {
            ctd.buffer_length = buffer_length;
            ctd.sample_rate = sample_rate;
            drop(ctd);
            self.reload_structure();
        }
    }

    pub fn note_on(&mut self, index: i32, velocity: f32) {
        self.ctd_mux
            .lock()
            .unwrap()
            .note_manager
            .note_on(&mut self.executor, index, velocity)
    }

    pub fn note_off(&mut self, index: i32) {
        self.ctd_mux.lock().unwrap().note_manager.note_off(index)
    }

    pub fn render_audio(&mut self) -> &[f32] {
        let mut ctd = self.ctd_mux.lock().unwrap();

        if let Some(new_aux_data) = ctd.new_aux_input_values.take() {
            self.executor.change_aux_input_data(&new_aux_data[..]);
        }
        if let Some((code, starting_aux_data, feedback_data_len)) = ctd.new_module_graph_code.take()
        {
            println!("{}", code);
            let result = self.executor.compile(
                code,
                starting_aux_data,
                ctd.buffer_length as usize,
                ctd.sample_rate,
                feedback_data_len,
            );
            if let Err(err) = result {
                eprintln!("Compile failed!");
                eprintln!("{}", err);
                panic!("TODO: Nice error.")
            }
        }
        if self.rendered_audio.len() != ctd.buffer_length as usize * 2 {
            self.rendered_audio
                .resize(ctd.buffer_length as usize * 2, 0.0);
        }
        let update_feedback_data =
            self.last_feedback_data_update.elapsed() > FEEDBACK_UPDATE_INTERVAL;
        if update_feedback_data {
            self.last_feedback_data_update = Instant::now();
        }
        let feedback_data = ctd
            .note_manager
            .render_all_notes(
                &mut self.executor,
                &mut self.rendered_audio[..],
                update_feedback_data,
            )
            .expect("TODO: Nice error.");
        // This can still be None even if update_feedback_data == true when no notes are playing.
        if let Some(data) = feedback_data {
            ctd.new_feedback_data = Some(data);
        }
        &self.rendered_audio[..]
    }
}
