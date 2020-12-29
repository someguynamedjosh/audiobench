use super::codegen::{self, CodeGenResult};
use super::data_routing::{AutoconDynDataCollector, FeedbackDisplayer, StaticonDynDataCollector};
use super::data_transfer::{DataFormat, GlobalData, GlobalParameters};
use super::julia_thread;
use super::parts::ModuleGraph;
use super::program_wrapper::{AudiobenchExecutor, AudiobenchExecutorBuilder, NoteTracker};
use crate::registry::{save_data::Patch, Registry};
use julia_helper::GeneratedCode;
use shared_util::{perf_counter::sections, prelude::*};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const DEFAULT_CHANNELS: usize = 2;
const DEFAULT_BUFFER_LENGTH: usize = 512;
const DEFAULT_SAMPLE_RATE: usize = 44100;
const FEEDBACK_UPDATE_INTERVAL: Duration = Duration::from_millis(50);

type PreferredPerfCounter = shared_util::perf_counter::SimplePerfCounter;

struct UiThreadData {
    module_graph: Rcrc<ModuleGraph>,
    autocon_dyn_data_collector: AutoconDynDataCollector,
    staticon_dyn_data_collector: StaticonDynDataCollector,
    feedback_displayer: FeedbackDisplayer,
    current_patch_save_data: Rcrc<Patch>,
}

pub(super) struct CrossThreadData {
    pub julia_thread_status: julia_thread::Status,
    /// We put this in with the other ctd instead of giving each thread its own copy so that we
    /// don't get a situation where two threads send two requests at the same time thinking the
    /// thread is not busy (julia_thread_status) and expecting to get a result back quickly.
    pub julia_request_input: SyncSender<julia_thread::Request>,
    pub global_params: GlobalParameters,
    // new_source: Option<(GeneratedCode, DataFormat)>,
    // new_autocon_dyn_data: Option<Vec<f32>>,
    // new_staticon_dyn_data: Option<Vec<()>>, // Previously OwnedIOData
    // new_feedback_data: Option<Vec<f32>>,
    pub critical_error: Option<String>,
    pub perf_counter: PreferredPerfCounter,
}

struct AudioThreadData {
    audio_buffer: Vec<f32>,
    global_data: GlobalData,
    last_feedback_data_update: Instant,
    audio_response_output: Receiver<julia_thread::AudioResponse>,
}

pub struct Engine {
    utd: UiThreadData,
    ctd_mux: Arc<Mutex<CrossThreadData>>,
    atd: AudioThreadData,
}

impl Engine {
    // MISC METHODS ================================================================================
    pub fn perf_counter_report(&self) -> String {
        self.ctd_mux.lock().unwrap().perf_counter.report()
    }

    pub fn new(registry: &mut Registry) -> Result<Self, String> {
        let mut module_graph = ModuleGraph::new();
        let global_params = GlobalParameters {
            channels: DEFAULT_CHANNELS,
            buffer_length: DEFAULT_BUFFER_LENGTH,
            sample_rate: DEFAULT_SAMPLE_RATE,
        };
        let default_patch = Rc::clone(
            registry
                .get_patch_by_name("Factory:patches/default.abpatch")
                .ok_or("Could not find Factory:patches/default.abpatch".to_owned())?,
        );
        default_patch
            .borrow()
            .restore_note_graph(&mut module_graph, registry)
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
        } = codegen::generate_code(&module_graph, &global_params).map_err(|_| {
            format!(concat!(
                "Default patch contains feedback loops!\n",
                "This is a critical error, please submit a bug report containing this ",
                "error.",
            ),)
        })?;

        let (reqi, reqo) = mpsc::sync_channel(0);
        let (audio_resi, audio_reso) = mpsc::sync_channel(0);

        let utd = UiThreadData {
            module_graph: rcrc(module_graph),
            autocon_dyn_data_collector,
            staticon_dyn_data_collector,
            feedback_displayer,
            current_patch_save_data: default_patch,
        };

        let atd = AudioThreadData {
            audio_buffer: vec![0.0; data_format.global_params.buffer_length * 2],
            global_data: GlobalData::new(),
            last_feedback_data_update: Instant::now(),
            audio_response_output: audio_reso,
        };

        let global_params_2 = global_params.clone();
        let ctd = CrossThreadData {
            julia_thread_status: julia_thread::Status::Busy,
            julia_request_input: reqi,
            global_params,
            critical_error: None,
            perf_counter: PreferredPerfCounter::new(),
        };
        let ctd_mux = Arc::new(Mutex::new(ctd));

        let executor_builder = AudiobenchExecutorBuilder::new(registry);
        let julia_ctd_mux = Arc::clone(&ctd_mux);
        let julia_executor = move || {
            julia_thread::entry(
                julia_ctd_mux,
                global_params_2,
                executor_builder,
                code,
                reqo,
                audio_resi,
            );
        };
        std::thread::Builder::new()
            .name("julia_executor".to_owned())
            .spawn(julia_executor);

        Ok(Self { utd, ctd_mux, atd })
    }

    pub fn clone_critical_error(&self) -> Option<String> {
        self.ctd_mux.lock().unwrap().critical_error.clone()
    }

    pub fn is_julia_thread_busy(&self) -> bool {
        self.ctd_mux.lock().unwrap().julia_thread_status == julia_thread::Status::Busy
    }

    // UI THREAD METHODS ===========================================================================
    pub fn rename_current_patch(&mut self, name: String) {
        assert!(self.utd.current_patch_save_data.borrow().is_writable());
        let mut patch_ref = self.utd.current_patch_save_data.borrow_mut();
        patch_ref.set_name(name);
        patch_ref.write().unwrap();
    }

    pub fn save_current_patch(&mut self, registry: &Registry) {
        assert!(self.utd.current_patch_save_data.borrow().is_writable());
        let mut patch_ref = self.utd.current_patch_save_data.borrow_mut();
        patch_ref.save_note_graph(&*self.utd.module_graph.borrow(), registry);
        patch_ref.write().unwrap();
    }

    pub fn borrow_current_patch(&self) -> &Rcrc<Patch> {
        &self.utd.current_patch_save_data
    }

    pub fn serialize_current_patch(&self, registry: &Registry) -> String {
        let mut patch_ref = self.utd.current_patch_save_data.borrow_mut();
        patch_ref.save_note_graph(&*self.utd.module_graph.borrow(), registry);
        patch_ref.serialize()
    }

    pub fn new_patch(&mut self, registry: &mut Registry) -> &Rcrc<Patch> {
        let new_patch = Rc::clone(registry.create_new_user_patch());
        let mut new_patch_ref = new_patch.borrow_mut();
        new_patch_ref.set_name("New Patch".to_owned());
        new_patch_ref.save_note_graph(&*self.utd.module_graph.borrow(), registry);
        new_patch_ref.write().unwrap();
        drop(new_patch_ref);
        // Don't reload anything because we are just copying the current patch data.
        self.utd.current_patch_save_data = new_patch;
        &self.utd.current_patch_save_data
    }

    pub fn new_patch_from_clipboard(
        &mut self,
        registry: &mut Registry,
        clipboard_data: &[u8],
    ) -> Result<&Rcrc<Patch>, String> {
        let new_patch = Rc::clone(registry.create_new_user_patch());
        let mut new_patch_ref = new_patch.borrow_mut();
        new_patch_ref.load_from_serialized_data(clipboard_data, registry)?;
        let name = format!("{} (pasted)", new_patch_ref.borrow_name());
        new_patch_ref.set_name(name);
        drop(new_patch_ref);
        self.load_patch(registry, Rc::clone(&new_patch))?;
        Ok(&self.utd.current_patch_save_data)
    }

    pub fn load_patch(&mut self, registry: &Registry, patch: Rcrc<Patch>) -> Result<(), String> {
        self.utd.current_patch_save_data = patch;
        self.utd
            .current_patch_save_data
            .borrow()
            .restore_note_graph(&mut *self.utd.module_graph.borrow_mut(), registry)?;
        self.recompile()?;
        Ok(())
    }

    pub fn borrow_module_graph_ref(&self) -> &Rcrc<ModuleGraph> {
        &self.utd.module_graph
    }

    pub fn recompile(&mut self) -> Result<(), String> {
        let mut ctd = self.ctd_mux.lock().unwrap();

        let module_graph_ref = self.utd.module_graph.borrow();
        let section = ctd.perf_counter.begin_section(&sections::GENERATE_CODE);
        let new_gen = codegen::generate_code(&*module_graph_ref, &ctd.global_params)
            .map_err(|_| format!("The note graph cannot contain feedback loops"))?;
        ctd.perf_counter.end_section(section);
        drop(module_graph_ref);
        // ctd.new_source = Some((new_gen.code, new_gen.data_format.clone()));
        let section = ctd
            .perf_counter
            .begin_section(&sections::COLLECT_AUTOCON_DATA);
        // ctd.new_autocon_dyn_data = Some(new_gen.autocon_dyn_data_collector.collect_data());
        ctd.perf_counter.end_section(section);
        let section = ctd
            .perf_counter
            .begin_section(&sections::COLLECT_STATICON_DATA);
        // ctd.new_staticon_dyn_data = Some(new_gen.staticon_dyn_data_collector.collect_data());
        ctd.perf_counter.end_section(section);
        // ctd.new_feedback_data = None;
        self.utd.autocon_dyn_data_collector = new_gen.autocon_dyn_data_collector;
        self.utd.staticon_dyn_data_collector = new_gen.staticon_dyn_data_collector;
        self.utd.feedback_displayer = new_gen.feedback_displayer;
        Ok(())
    }

    pub fn reload_autocon_dyn_data(&mut self) {
        let mut ctd = self.ctd_mux.lock().unwrap();

        let section = ctd
            .perf_counter
            .begin_section(&sections::COLLECT_AUTOCON_DATA);
        // ctd.new_autocon_dyn_data = Some(self.utd.autocon_dyn_data_collector.collect_data());
        ctd.perf_counter.end_section(section);
    }

    pub fn reload_staticon_dyn_data(&mut self) {
        let mut ctd = self.ctd_mux.lock().unwrap();

        let section = ctd
            .perf_counter
            .begin_section(&sections::COLLECT_STATICON_DATA);
        // ctd.new_staticon_dyn_data = Some(self.utd.staticon_dyn_data_collector.collect_data());
        ctd.perf_counter.end_section(section);
    }

    /// Feedback data is generated on the audio thread. This method uses a mutex to retrieve that
    /// data and copy it so that it can be displayed in the GUI. Nothing will happen if there is no
    /// new data so this is okay to call relatively often. It also does not block on waiting for
    /// the mutex.
    pub fn display_new_feedback_data(&mut self) {
        if let Ok(mut ctd) = self.ctd_mux.try_lock() {
            // if let Some(data) = ctd.new_feedback_data.take() {
            //     self.utd.feedback_displayer.display_feedback(&data[..]);
            // }
        }
    }

    // AUDIO THREAD METHODS ========================================================================
    pub fn set_global_params(&mut self, buffer_length: usize, sample_rate: usize) {
        let mut ctd = self.ctd_mux.lock().unwrap();

        // Avoid recompiling if there was no change.
        if buffer_length != ctd.global_params.buffer_length
            || sample_rate != ctd.global_params.sample_rate
        {
            ctd.global_params.buffer_length = buffer_length;
            ctd.global_params.sample_rate = sample_rate;
            drop(ctd);
            // This only errs if we have a feedback loop. Changing meta params does not introduce
            // feedback loops.
            // TODO: This is only supposed to be called from the UI thread.
            self.recompile().unwrap();
        }
    }

    pub fn start_note(&mut self, index: usize, velocity: f32) {
        // TODO: Don't wait for julia thread.
        let mut ctd = self.ctd_mux.lock().unwrap();
        ctd.julia_request_input
            .send(julia_thread::Request::StartNote { index, velocity })
            .unwrap();
    }

    pub fn release_note(&mut self, index: usize) {
        // TODO: Don't wait for julia thread.
        let mut ctd = self.ctd_mux.lock().unwrap();
        ctd.julia_request_input
            .send(julia_thread::Request::ReleaseNote { index })
            .unwrap();
    }

    pub fn set_pitch_wheel(&mut self, new_pitch_wheel: f32) {
        assert!(
            new_pitch_wheel >= -1.0 && new_pitch_wheel <= 1.0,
            "{} is not a valid pitch wheel value.",
            new_pitch_wheel
        );
        self.atd.global_data.pitch_wheel = new_pitch_wheel;
    }

    pub fn set_control(&mut self, index: usize, value: f32) {
        assert!(
            value >= -1.0 && value <= 1.0,
            "{} is not a valid control value.",
            value
        );
        assert!(index < 128, "{} is not a valid control index.", index);
        self.atd.global_data.controller_values[index] = value;
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.atd.global_data.bpm = bpm;
    }

    pub fn set_elapsed_time(&mut self, time: f32) {
        self.atd.global_data.elapsed_time = time;
    }

    pub fn set_elapsed_beats(&mut self, beats: f32) {
        self.atd.global_data.elapsed_beats = beats;
    }

    pub fn render_audio(&mut self) -> Vec<f32> {
        // TODO: Also this.
        // if let Some((code, data_format)) = ctd.new_source.take() {
        //     ctd.notes.set_data_format(data_format.clone());
        //     let section = ctd.perf_counter.begin_section(&sections::COMPILE_CODE);
        //     ctd.currently_compiling = true;
        //     // Compilation takes a while. Drop ctd so that other threads can use it.
        //     drop(ctd);
        //     let res = self.atd.executor.change_generated_code(code);
        //     ctd = self.ctd_mux.lock().unwrap();
        //     if let Err(err) = res {
        //         ctd.critical_error = Some(format!(
        //             concat!(
        //                 "Note graph failed to compile!\n",
        //                 "This is a critical error, please submit a bug report containing this error ",
        //                 "message:\n\n{}",
        //             ),
        //             err
        //         ));
        //     }
        //     ctd.currently_compiling = false;
        //     ctd.perf_counter.end_section(section);
        // }
        // TODO: This.
        // if let Some(program) = &mut self.atd.current_program {
        //     let mut input_packer = program.get_input_packer();
        //     if let Some(new_autocon_dyn_data) = ctd.new_autocon_dyn_data.take() {
        //         input_packer.set_autocon_dyn_data(&new_autocon_dyn_data[..]);
        //     }
        //     if let Some(new_staticon_dyn_data) = ctd.new_staticon_dyn_data.take() {
        //         input_packer.set_staticon_dyn_data(&new_staticon_dyn_data[..]);
        //     }
        // }
        let update_feedback_data =
            self.atd.last_feedback_data_update.elapsed() > FEEDBACK_UPDATE_INTERVAL;
        if update_feedback_data {
            self.atd.last_feedback_data_update = Instant::now();
        }
        let mut ctd = self.ctd_mux.lock().unwrap();
        let ok = ctd
            .julia_request_input
            .try_send(julia_thread::Request::Render(self.atd.global_data.clone()))
            .is_ok();
        let buf_time =
            ctd.global_params.buffer_length as f32 / ctd.global_params.sample_rate as f32;
        drop(ctd);
        self.atd.global_data.elapsed_time += buf_time;
        self.atd.global_data.elapsed_beats += buf_time * self.atd.global_data.bpm / 60.0;
        if ok {
            if let Ok(result) = self.atd.audio_response_output.recv() {
                return result.audio;
            }
        }
        let mut ctd = self.ctd_mux.lock().unwrap();
        let size = ctd.global_params.channels * ctd.global_params.buffer_length;
        vec![0.0; size]
        // if self.atd.executor.is_generated_code_loaded() {
        //     let CrossThreadData {
        //         notes,
        //         perf_counter,
        //         ..
        //     } = &mut *ctd;
        //     let result = self.atd.executor.execute(
        //         update_feedback_data,
        //         &mut self.atd.global_data,
        //         notes,
        //         &mut self.atd.audio_buffer[..],
        //         perf_counter,
        //     );
        //     if let Err(err) = result {
        //         ctd.critical_error = Some(err);
        //     // TODO: Clear program?
        //     } else if let Ok(true) = result {
        //         // Returns true if new feedback data was written.
        //         // TODO: This.
        //         // ctd.new_feedback_data = Some(Vec::from(
        //         //     program.get_output_unpacker().borrow_feedback_data(),
        //         // ));
        //     }
        // }
    }
}
