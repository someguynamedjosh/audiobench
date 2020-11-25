use super::codegen::{self, CodeGenResult};
use super::data_routing::{AutoconDynDataCollector, FeedbackDisplayer, StaticonDynDataCollector};
use super::data_transfer::{DataFormat, HostData, HostFormat};
use super::parts::ModuleGraph;
use super::program_wrapper::{AudiobenchCompiler, AudiobenchProgram, NoteTracker};
use crate::registry::{save_data::Patch, Registry};
use nodespeak::llvmir::structure::OwnedIOData;
use shared_util::{perf_counter::sections, prelude::*};
use std::time::{Duration, Instant};

const DEFAULT_BUFFER_LENGTH: usize = 512;
const DEFAULT_SAMPLE_RATE: usize = 44100;
const FEEDBACK_UPDATE_INTERVAL: Duration = Duration::from_millis(50);

type PreferredPerfCounter = shared_util::perf_counter::SimplePerfCounter;

struct UiThreadData {
    registry: Rcrc<Registry>,
    module_graph: Rcrc<ModuleGraph>,
    autocon_dyn_data_collector: AutoconDynDataCollector,
    staticon_dyn_data_collector: StaticonDynDataCollector,
    feedback_displayer: FeedbackDisplayer,
    current_patch_save_data: Rcrc<Patch>,
}

struct CrossThreadData {
    host_format: HostFormat,
    notes: NoteTracker,
    new_source: Option<(String, DataFormat)>,
    new_autocon_dyn_data: Option<Vec<f32>>,
    new_staticon_dyn_data: Option<Vec<OwnedIOData>>,
    new_feedback_data: Option<Vec<f32>>,
    critical_error: Option<String>,
    perf_counter: PreferredPerfCounter,
    // Set if the audio thread has triggered something that requires recompiling.
    request_recompile: bool,
    currently_compiling: bool,
}

struct AudioThreadData {
    compiler: AudiobenchCompiler,
    current_program: Option<AudiobenchProgram>,
    host_data: HostData,
    audio_buffer: Vec<f32>,
    last_feedback_data_update: Instant,
}

pub struct UiThreadEngine {
    data: UiThreadData,
    ctd_mux: Arcmux<CrossThreadData>,
}

pub struct AudioThreadEngine {
    data: AudioThreadData,
    ctd_mux: Arcmux<CrossThreadData>,
}

pub fn new_engine(
    registry_ptr: Rcrc<Registry>,
) -> Result<(Rcrc<UiThreadEngine>, Rcrc<AudioThreadEngine>), String> {
    let registry = registry_ptr.borrow_mut();
    let mut module_graph = ModuleGraph::new();
    let host_format = HostFormat {
        buffer_len: DEFAULT_BUFFER_LENGTH,
        sample_rate: DEFAULT_SAMPLE_RATE,
    };
    let default_patch = Rc::clone(
        registry
            .get_patch_by_name("factory:patches/default.abpatch")
            .ok_or("Could not find factory:patches/default.abpatch".to_owned())?,
    );
    default_patch
        .borrow()
        .restore_note_graph(&mut module_graph, &*registry)
        .map_err(|err| {
            format!(
                concat!(
                    "Default patch failed to load!\n",
                    "This is a critical error, please submit a bug report containing this ",
                    "error:\n\n{}",
                ),
                err
            )
        })?;
    let CodeGenResult {
        code,
        autocon_dyn_data_collector,
        staticon_dyn_data_collector,
        feedback_displayer,
        data_format,
    } = codegen::generate_code(&module_graph, &host_format).map_err(|_| {
        format!(concat!(
            "Default patch contains feedback loops!\n",
            "This is a critical error, please submit a bug report containing this ",
            "error.",
        ),)
    })?;
    let utd = UiThreadData {
        registry: Rc::clone(&registry_ptr),
        module_graph: rcrc(module_graph),
        autocon_dyn_data_collector,
        staticon_dyn_data_collector,
        feedback_displayer,
        current_patch_save_data: default_patch,
    };

    let mut compiler = AudiobenchCompiler::new(&*registry);
    let program = compiler.compile(code, data_format.clone()).map_err(|err| {
        format!(
            concat!(
                "Default patch failed to compile!\n",
                "This is a critical error, please submit a bug report containing this ",
                "error:\n\n{}"
            ),
            err
        )
    })?;
    let atd = AudioThreadData {
        compiler,
        current_program: Some(program),
        host_data: HostData::new(),
        audio_buffer: vec![0.0; data_format.host_format.buffer_len * 2],
        last_feedback_data_update: Instant::now(),
    };

    let ctd = CrossThreadData {
        host_format,
        notes: NoteTracker::new(data_format.clone()),
        new_source: None,
        new_autocon_dyn_data: None,
        new_staticon_dyn_data: None,
        new_feedback_data: None,
        critical_error: None,
        perf_counter: PreferredPerfCounter::new(),
        request_recompile: false,
        currently_compiling: false,
    };
    let ctd_mux = arcmux(ctd);

    Ok((
        rcrc(UiThreadEngine {
            data: utd,
            ctd_mux: Arc::clone(&ctd_mux),
        }),
        rcrc(AudioThreadEngine {
            data: atd,
            ctd_mux: Arc::clone(&ctd_mux),
        }),
    ))
}

impl UiThreadEngine {
    pub fn perf_counter_report(&self) -> String {
        self.ctd_mux.lock().unwrap().perf_counter.report()
    }

    pub fn clone_critical_error(&self) -> Option<String> {
        self.ctd_mux.lock().unwrap().critical_error.clone()
    }

    pub fn is_currently_compiling(&self) -> bool {
        self.ctd_mux.lock().unwrap().currently_compiling
    }

    pub fn rename_current_patch(&mut self, name: String) {
        assert!(self.data.current_patch_save_data.borrow().is_writable());
        let mut patch_ref = self.data.current_patch_save_data.borrow_mut();
        patch_ref.set_name(name);
        patch_ref.write().unwrap();
    }

    pub fn borrow_registry(&self) -> &Rcrc<Registry> {
        &self.data.registry
    }

    pub fn save_current_patch(&mut self) {
        assert!(self.data.current_patch_save_data.borrow().is_writable());
        let mut patch_ref = self.data.current_patch_save_data.borrow_mut();
        let reg = self.data.registry.borrow();
        patch_ref.save_note_graph(&*self.data.module_graph.borrow(), &*reg);
        patch_ref.write().unwrap();
    }

    pub fn borrow_current_patch(&self) -> &Rcrc<Patch> {
        &self.data.current_patch_save_data
    }

    pub fn serialize_current_patch(&self) -> String {
        let mut patch_ref = self.data.current_patch_save_data.borrow_mut();
        let reg = self.data.registry.borrow();
        patch_ref.save_note_graph(&*self.data.module_graph.borrow(), &*reg);
        patch_ref.serialize()
    }

    pub fn save_current_patch_with_new_name(&mut self) -> &Rcrc<Patch> {
        let mut reg = self.data.registry.borrow_mut();
        let patch = self.borrow_current_patch().borrow();
        let name = shared_util::increment_name(patch.borrow_name());
        let new_patch = Rc::clone(reg.create_new_user_patch());
        let mut new_patch_ref = new_patch.borrow_mut();
        new_patch_ref.set_name(name);
        new_patch_ref.save_note_graph(&*self.data.module_graph.borrow(), &*reg);
        new_patch_ref.write().unwrap();
        drop(new_patch_ref);
        drop(patch);
        drop(reg);
        // Don't reload anything because we are just copying the current patch data.
        self.data.current_patch_save_data = new_patch;
        &self.data.current_patch_save_data
    }

    pub fn new_patch_from_clipboard(
        &mut self,
        clipboard_data: &[u8],
    ) -> Result<&Rcrc<Patch>, String> {
        let mut reg = self.data.registry.borrow_mut();
        let new_patch = Rc::clone(reg.create_new_user_patch());
        let mut new_patch_ref = new_patch.borrow_mut();
        new_patch_ref.load_from_serialized_data(clipboard_data, &*reg)?;
        let name = format!("{} (pasted)", new_patch_ref.borrow_name());
        new_patch_ref.set_name(name);
        drop(new_patch_ref);
        drop(reg);
        self.load_patch(Rc::clone(&new_patch))?;
        Ok(&self.data.current_patch_save_data)
    }

    pub fn load_patch(&mut self, patch: Rcrc<Patch>) -> Result<(), String> {
        let reg = self.data.registry.borrow();
        self.data.current_patch_save_data = patch;
        self.data
            .current_patch_save_data
            .borrow()
            .restore_note_graph(&mut *self.data.module_graph.borrow_mut(), &*reg)?;
        drop(reg);
        self.recompile();
        Ok(())
    }

    pub fn borrow_module_graph_ref(&self) -> &Rcrc<ModuleGraph> {
        &self.data.module_graph
    }

    pub fn recompile(&mut self) {
        let mut ctd = self.ctd_mux.lock().unwrap();

        let module_graph_ref = self.data.module_graph.borrow();
        let section = ctd.perf_counter.begin_section(&sections::GENERATE_CODE);
        let new_gen = codegen::generate_code(&*module_graph_ref, &ctd.host_format)
            .map_err(|_| format!("The note graph cannot contain feedback loops"));
        ctd.perf_counter.end_section(section);
        let new_gen = new_gen.expect("TODO: Nice error.");
        drop(module_graph_ref);
        ctd.new_source = Some((new_gen.code, new_gen.data_format.clone()));
        let section = ctd
            .perf_counter
            .begin_section(&sections::COLLECT_AUTOCON_DATA);
        ctd.new_autocon_dyn_data = Some(new_gen.autocon_dyn_data_collector.collect_data());
        ctd.perf_counter.end_section(section);
        let section = ctd
            .perf_counter
            .begin_section(&sections::COLLECT_STATICON_DATA);
        ctd.new_staticon_dyn_data = Some(new_gen.staticon_dyn_data_collector.collect_data());
        ctd.perf_counter.end_section(section);
        ctd.new_feedback_data = None;
        self.data.autocon_dyn_data_collector = new_gen.autocon_dyn_data_collector;
        self.data.staticon_dyn_data_collector = new_gen.staticon_dyn_data_collector;
        self.data.feedback_displayer = new_gen.feedback_displayer;
    }

    /// Recompiles everything if the audio thread has encountered something that requires
    /// recompiling. This method exists because compilation is started by the UI thread.
    pub fn recompile_if_requested_by_audio_thread(&mut self) {
        let mut ctd = self.ctd_mux.lock().unwrap();
        if ctd.request_recompile {
            ctd.request_recompile = false;
            drop(ctd);
            self.recompile();
        }
    }

    pub fn reload_autocon_dyn_data(&mut self) {
        let mut ctd = self.ctd_mux.lock().unwrap();

        let section = ctd
            .perf_counter
            .begin_section(&sections::COLLECT_AUTOCON_DATA);
        ctd.new_autocon_dyn_data = Some(self.data.autocon_dyn_data_collector.collect_data());
        ctd.perf_counter.end_section(section);
    }

    pub fn reload_staticon_dyn_data(&mut self) {
        let mut ctd = self.ctd_mux.lock().unwrap();

        let section = ctd
            .perf_counter
            .begin_section(&sections::COLLECT_STATICON_DATA);
        ctd.new_staticon_dyn_data = Some(self.data.staticon_dyn_data_collector.collect_data());
        ctd.perf_counter.end_section(section);
    }

    /// Feedback data is generated on the audio thread. This method uses a mutex to retrieve that
    /// data and copy it so that it can be displayed in the GUI. Nothing will happen if there is no
    /// new data so this is okay to call relatively often. It also does not block on waiting for
    /// the mutex.
    pub fn display_new_feedback_data(&mut self) {
        if let Ok(mut ctd) = self.ctd_mux.try_lock() {
            if let Some(data) = ctd.new_feedback_data.take() {
                self.data.feedback_displayer.display_feedback(&data[..]);
            }
        }
    }
}

impl AudioThreadEngine {
    pub fn set_host_format(&mut self, buffer_len: usize, sample_rate: usize) {
        let mut ctd = self.ctd_mux.lock().unwrap();

        // Avoid recompiling if there was no change.
        if buffer_len != ctd.host_format.buffer_len || sample_rate != ctd.host_format.sample_rate {
            ctd.host_format.buffer_len = buffer_len;
            ctd.host_format.sample_rate = sample_rate;
            ctd.request_recompile = true;
        }
    }

    pub fn start_note(&mut self, index: usize, velocity: f32) {
        if let Some(program) = &mut self.data.current_program {
            let mut ctd = self.ctd_mux.lock().unwrap();
            match program.create_static_data() {
                Ok(data) => ctd.notes.start_note(data, index, velocity),
                Err(error) => ctd.critical_error = Some(error),
            }
        }
    }

    pub fn release_note(&mut self, index: usize) {
        self.ctd_mux.lock().unwrap().notes.release_note(index)
    }

    pub fn set_pitch_wheel(&mut self, new_pitch_wheel: f32) {
        assert!(
            new_pitch_wheel >= -1.0 && new_pitch_wheel <= 1.0,
            "{} is not a valid pitch wheel value.",
            new_pitch_wheel
        );
        self.data.host_data.pitch_wheel_value = new_pitch_wheel;
    }

    pub fn set_control(&mut self, index: usize, value: f32) {
        assert!(
            value >= -1.0 && value <= 1.0,
            "{} is not a valid control value.",
            value
        );
        assert!(index < 128, "{} is not a valid control index.", index);
        self.data.host_data.controller_values[index] = value;
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.data.host_data.bpm = bpm;
    }

    pub fn set_song_time(&mut self, time: f32) {
        self.data.host_data.song_time = time;
    }

    pub fn set_song_beats(&mut self, beats: f32) {
        self.data.host_data.song_beats = beats;
    }

    pub fn render_audio(&mut self) -> &[f32] {
        let mut ctd = self.ctd_mux.lock().unwrap();

        // Don't run the program if we are waiting for it to be recompiled.
        if ctd.request_recompile {
            return &self.data.audio_buffer[..];
        }

        if let Some((code, data_format)) = ctd.new_source.take() {
            ctd.notes.set_data_format(data_format.clone());
            let section = ctd.perf_counter.begin_section(&sections::COMPILE_CODE);
            ctd.currently_compiling = true;
            // Compilation takes a while. Drop ctd so that other threads can use it.
            drop(ctd);
            self.data.compiler.reset_performance_counters();
            match self.data.compiler.compile(code, data_format) {
                Ok(program) => {
                    ctd = self.ctd_mux.lock().unwrap();
                    self.data.current_program = Some(program);
                }
                Err(err) => {
                    ctd = self.ctd_mux.lock().unwrap();
                    self.data.current_program = None;
                    ctd.critical_error = Some(format!(
                    concat!(
                        "Note graph failed to compile!\n",
                        "This is a critical error, please submit a bug report containing this error ",
                        "message:\n\n{}",
                    ),
                    err
                ));
                }
            }
            ctd.currently_compiling = false;
            ctd.perf_counter.end_section(section);
            // The compiler has its own performance counters, this method adds anything they
            // measure to our global performance counter.
            self.data
                .compiler
                .tally_performance_counters(&mut ctd.perf_counter);
        }
        if let Some(program) = &mut self.data.current_program {
            let mut input_packer = program.get_input_packer();
            if let Some(new_autocon_dyn_data) = ctd.new_autocon_dyn_data.take() {
                input_packer.set_autocon_dyn_data(&new_autocon_dyn_data[..]);
            }
            if let Some(new_staticon_dyn_data) = ctd.new_staticon_dyn_data.take() {
                input_packer.set_staticon_dyn_data(&new_staticon_dyn_data[..]);
            }
        }
        let audio_buf_len = ctd.host_format.buffer_len * 2;
        if self.data.audio_buffer.len() != audio_buf_len {
            self.data.audio_buffer.resize(audio_buf_len, 0.0);
        }
        let update_feedback_data =
            self.data.last_feedback_data_update.elapsed() > FEEDBACK_UPDATE_INTERVAL;
        if update_feedback_data {
            self.data.last_feedback_data_update = Instant::now();
        }
        if let Some(program) = &mut self.data.current_program {
            let CrossThreadData {
                notes,
                perf_counter,
                ..
            } = &mut *ctd;
            let result = program.execute(
                update_feedback_data,
                &mut self.data.host_data,
                notes,
                &mut self.data.audio_buffer[..],
                perf_counter,
            );
            if let Err(err) = result {
                ctd.critical_error = Some(err);
                self.data.current_program = None;
            } else if let Ok(true) = result {
                // Returns true if new feedback data was written.
                if let Some(data) = program.get_output_unpacker().borrow_feedback_data() {
                    ctd.new_feedback_data = Some(Vec::from(data));
                }
            }
        }
        &self.data.audio_buffer[..]
    }
}
