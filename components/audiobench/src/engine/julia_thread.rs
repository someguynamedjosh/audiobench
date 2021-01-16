use super::base::CrossThreadData;
use super::data_transfer::{GlobalData, GlobalParameters, IOData};
use super::program_wrapper::{AudiobenchExecutor, AudiobenchExecutorBuilder, NoteTracker};
use julia_helper::GeneratedCode;
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Status {
    Ready,
    Busy,
    Error,
}

pub enum Request {
    Render(GlobalData),
    ChangeGlobalParams(GlobalParameters),
    ChangeGeneratedCode {
        code: GeneratedCode,
        dyn_data: Vec<IOData>,
    },
    StartNote {
        index: usize,
        velocity: f32,
    },
    ReleaseNote {
        index: usize,
    },
}

pub struct AudioResponse {
    pub audio: Vec<f32>,
    // feedback_data: Option<Vec<f32>>
}

pub(super) fn entry(
    ctd_mux: Arc<Mutex<CrossThreadData>>,
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
            let mut ctd = ctd_mux.lock().unwrap();
            ctd.julia_thread_status = Status::Error;
            ctd.critical_error = Some(err);
            return;
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
        let mut ctd = ctd_mux.lock().unwrap();
        ctd.julia_thread_status = Status::Error;
        ctd.critical_error = Some(err);
        return;
    }

    let mut thread = JuliaThread {
        ctd_mux,
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
    ctd_mux: Arc<Mutex<CrossThreadData>>,
    executor: AudiobenchExecutor,
    global_params: GlobalParameters,
    dyn_data: Vec<IOData>,
    notes: NoteTracker,
    request_pipe: Receiver<Request>,
    audio_response_pipe: SyncSender<AudioResponse>,
}

impl JuliaThread {
    fn set_status(&mut self, status: Status) {
        // let mut ctd = self.ctd_mux.lock().unwrap();
        // ctd.julia_thread_status = status;
        // drop(ctd);
    }

    fn entry(&mut self) {
        self.set_status(Status::Ready);
        while let Ok(request) = self.request_pipe.recv() {
            self.set_status(Status::Busy);
            match request {
                Request::Render(global_data) => self.render(global_data),
                Request::ChangeGlobalParams(params) => {
                    self.executor
                        .change_parameters(&params)
                        .expect("TODO: Handle error.");
                    self.global_params = params;
                }
                Request::ChangeGeneratedCode { code, dyn_data } => {
                    self.dyn_data = dyn_data;
                    let res = self.executor.change_generated_code(code).map_err(|err| {
                        format!("Error encountered while loading new patch code:\n{}", err)
                    });
                    if let Err(err) = res {
                        let mut ctd = self.ctd_mux.lock().unwrap();
                        ctd.julia_thread_status = Status::Error;
                        ctd.critical_error = Some(err);
                        return;
                    }
                }
                Request::StartNote { index, velocity } => {
                    let static_index = self.notes.start_note(index, velocity);
                    self.executor
                        .reset_static_data(static_index)
                        .expect("TODO: Handle error.");
                }
                Request::ReleaseNote { index } => {
                    self.notes.release_note(index);
                }
            }
        }
    }

    fn render(&mut self, global_data: GlobalData) {
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
        self.set_status(Status::Ready);
    }
}
