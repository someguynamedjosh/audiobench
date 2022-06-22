use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, Sender, TrySendError};
use crossbeam_utils::atomic::AtomicCell;
use observatory::{observable, ObservablePtr};
use shared_util::prelude::*;

use crate::{
    engine::{
        codegen::{self, CodeGenResult},
        data_transfer::{
            DynDataCollector, FeedbackData, FeedbackDisplayer, GlobalData, GlobalParameters, IOData,
        },
        parts::{Module, ModuleGraph},
        processing_thread,
    },
    registry::{save_data::Patch, Registry},
};

use super::codegen::GeneratedCode;

const DEFAULT_CHANNELS: usize = 2;
const DEFAULT_BUFFER_LENGTH: usize = 512;
const DEFAULT_SAMPLE_RATE: usize = 44100;
const FEEDBACK_UPDATE_INTERVAL: Duration = Duration::from_millis(50);

type PreferredPerfCounter = shared_util::perf_counter::SimplePerfCounter;

struct UiThreadData {
    registry: Rcrc<Registry>,
    module_graph: Rcrc<ModuleGraph>,
    dyn_data_collector: DynDataCollector,
    feedback_displayer: FeedbackDisplayer,
    current_patch_save_data: ObservablePtr<Rcrc<Patch>>,
    posted_errors: Vec<String>,
    processing_errors: Receiver<String>,
}

pub(super) struct Communication {
    pub processing_thread_status: AtomicCell<processing_thread::Status>,

    pub new_global_params: AtomicCell<Option<()>>,
    pub new_note_graph_code: AtomicCell<Option<(GeneratedCode, Vec<IOData>)>>,
    pub new_dyn_data: AtomicCell<Option<Vec<IOData>>>,
    pub new_feedback: AtomicCell<Option<FeedbackData>>,
    pub do_dummy_note: AtomicCell<bool>,
    pub do_dummy_note_once: AtomicCell<bool>,
    pub module_view_index: AtomicCell<usize>,

    pub global_params: AtomicCell<GlobalParameters>,
    pub note_events: Mutex<Vec<processing_thread::NoteEvent>>,
    pub processing_render_pipe: Sender<processing_thread::RenderRequest>,
    pub processing_poll_pipe: Sender<()>,
}

struct AudioThreadData {
    global_data: GlobalData,
    last_feedback_data_update: Instant,
    audio_response_output: Receiver<processing_thread::AudioResponse>,
}

pub struct UiThreadEngine {
    data: UiThreadData,
    comms: Arc<Communication>,
}

pub struct AudioThreadEngine {
    data: AudioThreadData,
    comms: Arc<Communication>,
}

pub fn new_engine(
    registry_ptr: Rcrc<Registry>,
) -> Result<(Rcrc<UiThreadEngine>, Rcrc<AudioThreadEngine>), String> {
    let registry = registry_ptr.borrow_mut();
    let mut module_graph = ModuleGraph::new();
    let global_params = GlobalParameters {
        channels: DEFAULT_CHANNELS,
        buffer_length: DEFAULT_BUFFER_LENGTH,
        sample_rate: DEFAULT_SAMPLE_RATE,
    };
    let default_patch = Rc::clone(
        registry
            .get_patch_by_name("Factory:patches/Default.abpatch")
            .ok_or("Could not find Factory:patches/Default.abpatch".to_owned())?,
    );
    default_patch
        .borrow()
        .restore_note_graph(&mut module_graph, &*registry)
        .map_err(|_| {
            format!(concat!(
                "Default patch failed to load!\n",
                "This is a critical error, please submit a bug report containing this ",
                "error:\n\nPatch data is corrupt.",
            ))
        })?;
    let CodeGenResult {
        code,
        dyn_data_collector,
        feedback_displayer,
        ..
    } = codegen::generate_code(&module_graph, &global_params).map_err(|_| {
        format!(concat!(
            "Default patch contains feedback loops!\n",
            "This is a critical error, please submit a bug report containing this ",
            "error.",
        ),)
    })?;
    let dyn_data = dyn_data_collector.collect();

    let (renderi, rendero) = crossbeam_channel::bounded(0);
    let (polli, pollo) = crossbeam_channel::bounded(0xFF);
    let (audio_resi, audio_reso) = crossbeam_channel::bounded(0);
    let (jerrori, jerroro) = crossbeam_channel::unbounded();

    let utd = UiThreadData {
        registry: Rc::clone(&registry_ptr),
        module_graph: rcrc(module_graph),
        dyn_data_collector,
        feedback_displayer,
        current_patch_save_data: observable(default_patch),
        posted_errors: Vec::new(),
        processing_errors: jerroro,
    };

    let atd = AudioThreadData {
        global_data: GlobalData::new(),
        last_feedback_data_update: Instant::now(),
        audio_response_output: audio_reso,
    };

    let global_params_2 = global_params.clone();
    let comms = Communication {
        processing_thread_status: AtomicCell::new(processing_thread::Status::Busy),

        new_global_params: Default::default(),
        new_note_graph_code: Default::default(),
        new_dyn_data: Default::default(),
        new_feedback: Default::default(),
        do_dummy_note: AtomicCell::new(false),
        do_dummy_note_once: AtomicCell::new(false),
        module_view_index: AtomicCell::new(0),

        global_params: AtomicCell::new(global_params),
        note_events: Default::default(),
        processing_render_pipe: renderi,
        processing_poll_pipe: polli,
    };
    let comms = Arc::new(comms);

    let registry_source = codegen::generate_registry_code(&*registry)?;
    let comms2 = Arc::clone(&comms);
    let processing_executor = move || {
        processing_thread::entry(
            comms2,
            global_params_2,
            dyn_data,
            rendero,
            pollo,
            audio_resi,
            jerrori,
        );
    };
    std::thread::Builder::new()
        .name("processing_executor".to_owned())
        .spawn(processing_executor)
        .unwrap();

    Ok((
        rcrc(UiThreadEngine {
            data: utd,
            comms: Arc::clone(&comms),
        }),
        rcrc(AudioThreadEngine {
            data: atd,
            comms: Arc::clone(&comms),
        }),
    ))
}

impl UiThreadEngine {
    pub fn get_processing_thread_status(&self) -> processing_thread::Status {
        self.comms.processing_thread_status.load()
    }

    pub fn rename_current_patch(&mut self, name: String) {
        assert!(self
            .data
            .current_patch_save_data
            .borrow_untracked()
            .borrow()
            .is_writable());
        let patch_ref_ref = self.data.current_patch_save_data.borrow_untracked();
        let mut patch_ref = patch_ref_ref.borrow_mut();
        patch_ref.set_name(name);
    }

    pub fn post_error(&mut self, message: String) {
        self.data.posted_errors.push(message);
    }

    pub fn take_posted_errors(&mut self) -> Vec<String> {
        let mut errors = std::mem::take(&mut self.data.posted_errors);
        while let Ok(error) = self.data.processing_errors.try_recv() {
            errors.push(error);
        }
        errors
    }

    pub fn borrow_registry(&self) -> &Rcrc<Registry> {
        &self.data.registry
    }

    pub fn save_current_patch(&mut self) {
        assert!(self
            .data
            .current_patch_save_data
            .borrow_untracked()
            .borrow()
            .is_writable());
        let patch_ref_ref = self.data.current_patch_save_data.borrow_untracked();
        let mut patch_ref = patch_ref_ref.borrow_mut();
        let reg = self.data.registry.borrow();
        patch_ref.save_note_graph(&*self.data.module_graph.borrow(), &*reg);
        patch_ref.write().unwrap();
    }

    pub fn borrow_current_patch(&self) -> &ObservablePtr<Rcrc<Patch>> {
        &self.data.current_patch_save_data
    }

    pub fn serialize_current_patch(&self) -> String {
        let patch_ref_ref = self.data.current_patch_save_data.borrow_untracked();
        let patch_ref = patch_ref_ref.borrow();
        let reg = self.data.registry.borrow();
        // Use a dummy patch so we don't overwrite the actual save data of the current
        // patch without the user explicitly clicking 'save'.
        let mut dummy_patch = Patch::new_dummy(patch_ref.borrow_name().to_owned());
        dummy_patch.save_note_graph(&*self.data.module_graph.borrow(), &*reg);
        dummy_patch.serialize()
    }

    pub fn save_current_patch_with_new_name(&mut self) {
        let mut reg = self.data.registry.borrow_mut();
        let patch_ref_ref = self.data.current_patch_save_data.borrow_untracked();
        let patch_ref = patch_ref_ref.borrow();

        let mut name = shared_util::increment_name(patch_ref.borrow_name());
        let mut original = false;
        while !original {
            original = true;
            for patch in reg.borrow_patches() {
                if patch.borrow().borrow_name() == &name {
                    original = false;
                    break;
                }
            }
            if !original {
                name = shared_util::increment_name(&name);
            }
        }

        let new_patch = Rc::clone(reg.create_new_user_patch());
        let mut new_patch_ref = new_patch.borrow_mut();
        new_patch_ref.set_name(name);
        new_patch_ref.save_note_graph(&*self.data.module_graph.borrow(), &*reg);
        new_patch_ref.write().unwrap();
        drop(new_patch_ref);
        drop(patch_ref);
        drop(patch_ref_ref);
        drop(reg);
        // Don't reload anything because we are just copying the current patch data.
        self.data.current_patch_save_data.set(new_patch);
    }

    pub fn new_patch_from_clipboard(&mut self, clipboard_data: &[u8]) -> Result<(), ()> {
        let mut reg = self.data.registry.borrow_mut();
        let new_patch = Rc::clone(reg.create_new_user_patch());
        let mut new_patch_ref = new_patch.borrow_mut();
        let res = new_patch_ref.deserialize(clipboard_data);
        if let Err(err) = res {
            drop(new_patch_ref);
            drop(reg);
            self.post_error(err);
            return Err(());
        }
        let name = format!("{} (pasted)", new_patch_ref.borrow_name());
        new_patch_ref.set_name(name);
        drop(new_patch_ref);
        drop(reg);
        let res = self.load_patch(Rc::clone(&new_patch));
        if let Err(..) = res {
            self.post_error(format!("ERROR: Patch data is corrupt."));
            return Err(());
        }
        Ok(())
    }

    pub fn load_patch(&mut self, patch: Rcrc<Patch>) -> Result<(), ()> {
        let reg = self.data.registry.borrow();
        self.data.current_patch_save_data.set(patch);
        let res = self
            .data
            .current_patch_save_data
            .borrow_untracked()
            .borrow()
            .restore_note_graph(&mut *self.data.module_graph.borrow_mut(), &*reg);
        drop(reg);
        if let Err(..) = res {
            self.post_error(format!("ERROR: Patch data is corrupt."));
            return Err(());
        }
        self.data.module_graph.borrow().rebuild_widget();
        self.regenerate_code();
        Ok(())
    }

    pub fn borrow_module_graph_ref(&self) -> &Rcrc<ModuleGraph> {
        &self.data.module_graph
    }

    pub fn regenerate_code(&mut self) {
        let module_graph_ref = self.data.module_graph.borrow();
        let params = self.comms.global_params.load();
        let new_gen = codegen::generate_code(&*module_graph_ref, &params);
        let new_gen = if let Ok(value) = new_gen {
            value
        } else {
            drop(module_graph_ref);
            self.post_error("Module graph contains feedback loops.".to_owned());
            return;
        };
        drop(module_graph_ref);
        self.comms.new_dyn_data.store(None);
        let dyn_data = new_gen.dyn_data_collector.collect();
        self.comms
            .new_note_graph_code
            .store(Some((new_gen.code, dyn_data)));
        self.comms.processing_poll_pipe.send(()).unwrap();
        self.data.dyn_data_collector = new_gen.dyn_data_collector;
        self.data.feedback_displayer = new_gen.feedback_displayer;
    }

    pub fn reload_dyn_data(&mut self) {
        let data = self.data.dyn_data_collector.collect();
        self.comms.new_dyn_data.store(Some(data));
        self.comms.processing_poll_pipe.send(()).unwrap();
        self.set_dummy_note_active(true);
    }

    /// Feedback data is generated on the audio thread. This method uses a mutex
    /// to retrieve that data and copy it so that it can be displayed in the
    /// GUI. Nothing will happen if there is no new data so this is okay to
    /// call relatively often. It also does not block on waiting for
    /// the mutex.
    pub fn display_new_feedback_data(&mut self) {
        if let Some(data) = self.comms.new_feedback.take() {
            if let Some(widget) = &self.data.module_graph.borrow().current_widget {
                let widget = Rc::clone(widget);
                self.data.feedback_displayer.display(data, widget);
            }
        }
    }

    pub fn set_dummy_note_active(&self, should_be_active: bool) {
        self.comms.do_dummy_note.store(should_be_active);
    }

    pub fn activate_dummy_note_once(&self) {
        self.set_dummy_note_active(false);
        self.comms.do_dummy_note_once.store(true);
    }

    pub fn set_module_view(&self, module: &Rcrc<Module>) {
        let index = self
            .data
            .module_graph
            .borrow()
            .borrow_modules()
            .iter()
            .position(|other| Rc::ptr_eq(module, other));
        if let Some(index) = index {
            self.comms.module_view_index.store(index);
        }
    }

    pub fn virtual_keyboard_note(&self, index: usize, down: bool) {
        let mut events = self.comms.note_events.lock().unwrap();
        let event = if down {
            processing_thread::NoteEvent::StartNote {
                index,
                velocity: 1.0,
            }
        } else {
            processing_thread::NoteEvent::ReleaseNote { index }
        };
        events.push(event);
    }
}

impl AudioThreadEngine {
    // AUDIO THREAD METHODS
    // ========================================================================
    pub fn set_global_params(&mut self, buffer_length: usize, sample_rate: usize) {
        let mut params = self.comms.global_params.load();

        // Avoid recompiling if there was no change.
        if buffer_length != params.buffer_length || sample_rate != params.sample_rate {
            params.buffer_length = buffer_length;
            params.sample_rate = sample_rate;
            self.comms.new_global_params.store(Some(()));
            self.comms.global_params.store(params);
            self.comms.processing_poll_pipe.send(()).unwrap();
        }
    }

    pub fn start_note(&mut self, index: usize, velocity: f32) {
        let mut queue = self.comms.note_events.lock().unwrap();
        queue.push(processing_thread::NoteEvent::StartNote { index, velocity });
    }

    pub fn release_note(&mut self, index: usize) {
        let mut queue = self.comms.note_events.lock().unwrap();
        queue.push(processing_thread::NoteEvent::ReleaseNote { index });
    }

    pub fn set_pitch_wheel(&mut self, new_pitch_wheel: f32) {
        assert!(
            new_pitch_wheel >= -1.0 && new_pitch_wheel <= 1.0,
            "{} is not a valid pitch wheel value.",
            new_pitch_wheel
        );
        self.data.global_data.pitch_wheel = new_pitch_wheel;
    }

    pub fn set_control(&mut self, index: usize, value: f32) {
        assert!(
            value >= -1.0 && value <= 1.0,
            "{} is not a valid control value.",
            value
        );
        assert!(index < 128, "{} is not a valid control index.", index);
        self.data.global_data.controller_values[index] = value;
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.data.global_data.bpm = bpm;
    }

    pub fn set_elapsed_time(&mut self, time: f32) {
        self.data.global_data.elapsed_time = time;
    }

    pub fn set_elapsed_beats(&mut self, beats: f32) {
        self.data.global_data.elapsed_beats = beats;
    }

    pub fn render_audio(&mut self) -> Vec<f32> {
        let update_feedback_data =
            self.data.last_feedback_data_update.elapsed() > FEEDBACK_UPDATE_INTERVAL;
        if update_feedback_data {
            self.data.last_feedback_data_update = Instant::now();
        }

        // The thread will only be marked as busy if it is doing something that takes a
        // long time, e.g. compiling code.
        let mut ready = self.comms.processing_thread_status.load().is_ready();
        if ready {
            let data = self.data.global_data.clone();
            let request = processing_thread::RenderRequest {
                data,
                do_feedback: update_feedback_data,
            };
            let res = self.comms.processing_render_pipe.try_send(request);
            match res {
                Ok(()) => (),
                Err(TrySendError::Full(..)) => ready = false,
                Err(TrySendError::Disconnected(..)) => panic!("processing thread has shut down."),
            }
        }

        let params = self.comms.global_params.load();
        let buf_time = params.buffer_length as f32 / params.sample_rate as f32;
        self.data.global_data.elapsed_time += buf_time;
        self.data.global_data.elapsed_beats += buf_time * self.data.global_data.bpm / 60.0;

        if ready {
            self.data.audio_response_output.recv().unwrap().audio
        } else {
            let size = params.channels * params.buffer_length;
            vec![0.0; size]
        }
    }
}
