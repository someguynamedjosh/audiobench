use super::base::CrossThreadData;
use super::data_transfer::GlobalParameters;
use super::program_wrapper::{AudiobenchExecutor, AudiobenchExecutorBuilder};
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
    Render,
    StartNote { index: usize, velocity: f32 },
    ReleaseNote { index: usize },
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
        request_pipe,
        audio_response_pipe,
    };
    thread.entry();
}

struct JuliaThread {
    ctd_mux: Arc<Mutex<CrossThreadData>>,
    executor: AudiobenchExecutor,
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
                Request::Render => self.render(),
                Request::StartNote { index, velocity } => {
                    let mut ctd = self.ctd_mux.lock().unwrap();
                    let static_index = ctd.notes.start_note(index, velocity);
                    self.executor
                        .reset_static_data(static_index)
                        .expect("TODO: Handle error.");
                }
                Request::ReleaseNote { index } => {
                    let mut ctd = self.ctd_mux.lock().unwrap();
                    let static_index = ctd.notes.release_note(index);
                }
            }
        }
    }

    fn render(&mut self) {
        let mut ctd = self.ctd_mux.lock().unwrap();
        let mut output = vec![0.0; ctd.global_params.channels * ctd.global_params.buffer_length];
        let CrossThreadData {
            global_data,
            notes,
            perf_counter,
            ..
        } = &mut *ctd;
        let result =
            self.executor
                .execute(false, global_data, notes, &mut output[..], perf_counter);
        let feedback_updated = match result {
            Ok(v) => v,
            Err(err) => unimplemented!("Handle Julia error."),
        };
        self.audio_response_pipe
            .send(AudioResponse { audio: output });
        drop(ctd);
        self.set_status(Status::Ready);
    }
}
