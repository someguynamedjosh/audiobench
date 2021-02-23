use crate::engine::{
    data_transfer::{GlobalData, GlobalParameters, IOData},
    program_wrapper::{AudiobenchExecutor, NoteTracker},
    Communication,
};
use crossbeam_channel::{Receiver, Sender};
use julia_helper::GeneratedCode;
use std::sync::Arc;

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
    pub data: GlobalData,
    pub do_feedback: bool,
}

pub struct AudioResponse {
    pub audio: Vec<f32>,
    // feedback_data: Option<Vec<f32>>
}

pub(super) fn entry(
    comms: Arc<Communication>,
    global_params: GlobalParameters,
    registry_source: GeneratedCode,
    default_patch_code: GeneratedCode,
    dyn_data: Vec<IOData>,
    render_pipe: Receiver<RenderRequest>,
    poll_pipe: Receiver<()>,
    audio_response_pipe: Sender<AudioResponse>,
    error_report_pipe: Sender<String>,
) {
    let executor = AudiobenchExecutor::new(registry_source, &global_params).map_err(|err| {
        format!(
            "Failed to initialize execution environment! (See message log for details.)\n\n{}",
            err
        )
    });
    let mut executor = match executor {
        Ok(value) => value,
        Err(err) => {
            error_report_pipe.send(err);
            comms.julia_thread_status.store(Status::Error);
            panic!("Unrecoverable error.");
        }
    };
    let res = executor
        .change_generated_code(default_patch_code)
        .map_err(|err| {
            format!(
                "Default patch failed to compile! (See message log for details.)\n\n{}",
                err
            )
        });
    if let Err(err) = res {
        error_report_pipe.send(err);
        comms.julia_thread_status.store(Status::Error);
        panic!("Unrecoverable error.");
    }

    let mut thread = JuliaThread {
        comms,
        executor,
        global_params,
        dyn_data,
        notes: NoteTracker::new(),
        render_pipe,
        poll_pipe,
        audio_response_pipe,
        error_report_pipe,
    };
    thread.entry();
}

struct JuliaThread {
    comms: Arc<Communication>,
    executor: AudiobenchExecutor,
    global_params: GlobalParameters,
    dyn_data: Vec<IOData>,
    notes: NoteTracker,
    render_pipe: Receiver<RenderRequest>,
    poll_pipe: Receiver<()>,
    audio_response_pipe: Sender<AudioResponse>,
    error_report_pipe: Sender<String>,
}

// TODO: Preheat JIT unimplemented!()

impl JuliaThread {
    fn set_status(&self, status: Status) {
        self.comms.julia_thread_status.store(status);
    }

    fn entry(&mut self) {
        self.set_status(Status::Ready);
        loop {
            crossbeam_channel::select! {
                recv(self.render_pipe) -> msg => {
                    if let Ok(request) = msg {
                        self.render(request.data, request.do_feedback);
                    } else {
                        break;
                    }
                }
                recv(self.poll_pipe) -> msg => {
                    if msg.is_err() {
                        break;
                    }
                    // Drain the channel of extra requests before polling comms.
                    while let Ok(_) = self.poll_pipe.try_recv() { }
                    self.poll_comms();
                }
            }
            self.set_status(Status::Ready);
        }
        self.set_status(Status::Error);
    }

    fn report_julia_error(&mut self, message: String) {
        self.error_report_pipe.send(message);
        self.set_status(Status::Error);
    }

    fn poll_comms(&mut self) {
        self.set_status(Status::Busy);
        if let Some(_) = self.comms.new_global_params.take() {
            let params = self.comms.global_params.load();
            let result = self.executor.change_parameters(&params);
            if let Err(err) = result {
                let message = format!(
                    "Failed to load new parameter code, see message log for details.\n\n{}",
                    err
                );
                let message = self.report_julia_error(message);
                panic!("Unrecoverable error.");
            }
            self.global_params = params;
        } else if let Some((code, dyn_data)) = self.comms.new_note_graph_code.take() {
            self.notes.silence_all();
            self.dyn_data = dyn_data;
            let res = self.executor.change_generated_code(code);
            if let Err(err) = res {
                let message = format!(
                    "Failed to load new patch code, see message log for details.\n\n{}",
                    err
                );
                self.report_julia_error(message);
                panic!("Unrecoverable error.");
            }
        } else if let Some(data) = self.comms.new_dyn_data.take() {
            self.dyn_data = data;
        }
    }

    fn render(&mut self, global_data: GlobalData, do_feedback: bool) {
        self.set_status(Status::Rendering);
        let mut nel = self.comms.note_events.lock().unwrap();
        let note_events = std::mem::replace(&mut *nel, Default::default());
        drop(nel);
        for event in note_events {
            match event {
                NoteEvent::StartNote { index, velocity } => {
                    let static_index = self.notes.start_note(index, velocity);
                    let result = self.executor.reset_static_data(static_index);
                    if let Err(err) = result {
                        let message = format!(
                            "Encountered Julia error, see message log for details.\n\n{}",
                            err
                        );
                        self.report_julia_error(message);
                        // This error is "recoverable"
                    }
                }
                NoteEvent::ReleaseNote { index } => {
                    self.notes.release_note(index);
                }
            }
        }

        let mut output = vec![0.0; self.global_params.channels * self.global_params.buffer_length];
        let result = self.executor.execute(
            do_feedback,
            &global_data,
            &mut self.notes,
            &self.dyn_data[..],
            &mut output[..],
        );
        let new_feedback_data = match result {
            Ok(v) => v,
            Err(err) => {
                let message = format!(
                    "Encountered Julia error while executing, see message log for details.\n\n{}",
                    err
                );
                self.report_julia_error(message);
                // This error is "recoverable"
                None
            }
        };
        if new_feedback_data.is_some() {
            self.comms.new_feedback.store(new_feedback_data);
        }
        self.audio_response_pipe
            .send(AudioResponse { audio: output })
            .unwrap();
    }
}
