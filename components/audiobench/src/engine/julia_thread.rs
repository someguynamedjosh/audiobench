use super::data_transfer::{GlobalData, GlobalParameters, IOData};
use super::program_wrapper::{AudiobenchExecutor, AudiobenchExecutorBuilder, NoteTracker};
use super::Communication;
use julia_helper::GeneratedCode;
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Status {
    Ready,
    Busy,
    Rendering,
    Error,
}

impl Status {
    pub fn is_ready(&self) -> bool {
        self == &Self::Ready
    }
}

pub enum NoteEvent {
    StartNote { index: usize, velocity: f32 },
    ReleaseNote { index: usize },
}

pub struct RenderRequest {
    data: GlobalData,
}

pub enum Request {
    PollComms,
    Render(GlobalData),
}

pub struct AudioResponse {
    pub audio: Vec<f32>,
    // feedback_data: Option<Vec<f32>>
}

pub(super) fn entry(
    comms: Arc<Communication>,
    global_params: GlobalParameters,
    executor_builder: AudiobenchExecutorBuilder,
    default_patch_code: GeneratedCode,
    dyn_data: Vec<IOData>,
    request_pipe: Receiver<Request>,
    audio_response_pipe: SyncSender<AudioResponse>,
) {
    let executor = executor_builder.build(&global_params).map_err(|err| {
        format!(
            concat!(
                "Failed to initialize execution environment!\n",
                "This is a critical error, please submit a bug report containing this ",
                "error:\n\n{}"
            ),
            err
        )
    });
    let mut executor = match executor {
        Ok(value) => value,
        Err(err) => {
            comms.julia_thread_status.store(Status::Error);
            unimplemented!();
            // let mut ctd = ctd_mux.lock().unwrap();
            // ctd.julia_thread_status = Status::Error;
            // ctd.critical_error = Some(err);
            // return;
        }
    };
    let res = executor
        .change_generated_code(default_patch_code)
        .map_err(|err| {
            format!(
                concat!(
                    "Default patch failed to compile!\n",
                    "This is a critical error, please submit a bug report containing this ",
                    "error:\n\n{}"
                ),
                err
            )
        });
    if let Err(err) = res {
        comms.julia_thread_status.store(Status::Error);
        unimplemented!();
        // let mut ctd = ctd_mux.lock().unwrap();
        // ctd.julia_thread_status = Status::Error;
        // ctd.critical_error = Some(err);
        // return;
    }

    let mut thread = JuliaThread {
        comms,
        executor,
        global_params,
        dyn_data,
        notes: NoteTracker::new(),
        request_pipe,
        audio_response_pipe,
    };
    thread.entry();
}

struct JuliaThread {
    comms: Arc<Communication>,
    executor: AudiobenchExecutor,
    global_params: GlobalParameters,
    dyn_data: Vec<IOData>,
    notes: NoteTracker,
    request_pipe: Receiver<Request>,
    audio_response_pipe: SyncSender<AudioResponse>,
}

// TODO: Preheat JIT unimplemented!()

impl JuliaThread {
    fn set_status(&self, status: Status) {
        self.comms.julia_thread_status.store(status);
    }

    fn entry(&mut self) {
        self.set_status(Status::Ready);
        while let Ok(request) = self.request_pipe.recv() {
            match request {
                Request::PollComms => self.poll_comms(),
                Request::Render(global_data) => self.render(global_data),
            }
            self.set_status(Status::Ready);
        }
    }

    fn poll_comms(&mut self) {
        self.set_status(Status::Busy);
        if let Some(_) = self.comms.new_global_params.take() {
            let params = self.comms.global_params.load();
            self.executor
                .change_parameters(&params)
                .expect("TODO: Handle error.");
            self.global_params = params;
        } else if let Some((code, dyn_data)) = self.comms.new_note_graph_code.take() {
            self.notes.silence_all();
            self.dyn_data = dyn_data;
            let res = self
                .executor
                .change_generated_code(code)
                .map_err(|err| format!("Error encountered while loading new patch code:\n{}", err));
            if let Err(err) = res {
                self.set_status(Status::Error);
                unimplemented!("Julia error:\n{}", err);
                // let mut ctd = self.ctd_mux.lock().unwrap();
                // ctd.julia_thread_status = Status::Error;
                // ctd.critical_error = Some(err);
                // return;
            }
        } else if let Some(data) = self.comms.new_dyn_data.take() {
            self.dyn_data = data;
        }
    }

    fn render(&mut self, global_data: GlobalData) {
        self.set_status(Status::Rendering);
        let mut nel = self.comms.note_events.lock().unwrap();
        let note_events = std::mem::replace(&mut *nel, Default::default());
        drop(nel);
        for event in note_events {
            match event {
                NoteEvent::StartNote { index, velocity } => {
                    let static_index = self.notes.start_note(index, velocity);
                    self.executor
                        .reset_static_data(static_index)
                        .expect("TODO: Handle error.");
                }
                NoteEvent::ReleaseNote { index } => {
                    self.notes.release_note(index);
                }
            }
        }

        let mut output = vec![0.0; self.global_params.channels * self.global_params.buffer_length];
        let result = self.executor.execute(
            false,
            &global_data,
            &mut self.notes,
            &self.dyn_data[..],
            &mut output[..],
        );
        let feedback_updated = match result {
            Ok(v) => v,
            Err(err) => unimplemented!("Handle Julia error:\n{}", err),
        };
        self.audio_response_pipe
            .send(AudioResponse { audio: output })
            .unwrap();
    }
}
