use super::controls::{AnyControl, FloatInRangeControl};
use super::parts::Module;
use shared_util::prelude::*;

// This packages changes made by the user to static controls into a format that can be read
// by the nodespeak parameter, so that trivial changes don't necessitate a recompile.
pub(super) struct ControlDynDataCollector {
    ordered_controls: Vec<AnyControl>,
}

impl ControlDynDataCollector {
    pub(super) fn new(ordered_controls: Vec<AnyControl>) -> Self {
        Self { ordered_controls }
    }

    // Previously OwnedIOData
    pub(super) fn collect_data(&self) -> Vec<()> {
        self.ordered_controls
            .iter()
            .filter_map(|control| {
                None
                // let control = control.borrow();
                // if control.is_static_only() {
                //     None
                // } else {
                //     Some(control.borrow_data().package_dyn_data().to_owned())
                // }
            })
            .collect()
    }
}

pub(super) struct FeedbackDisplayer {
    ordered_modules: Vec<Rcrc<Module>>,
    data_length: usize,
}

impl FeedbackDisplayer {
    pub(super) fn new(ordered_modules: Vec<Rcrc<Module>>, data_length: usize) -> Self {
        Self {
            ordered_modules,
            data_length,
        }
    }

    pub(super) fn display_feedback(&mut self, feedback_data: &[f32]) {
        assert!(feedback_data.len() == self.data_length);
        let mut data_pos = 0;
        for module in &self.ordered_modules {
            let module_ref = module.borrow_mut();
            let module_data_length = module_ref.template.borrow().feedback_data_len;
            if let Some(data_ptr) = &module_ref.feedback_data {
                let slice = &feedback_data[data_pos..data_pos + module_data_length];
                data_ptr.borrow_mut().clone_from_slice(slice);
            }
            data_pos += module_data_length;
        }
        debug_assert!(data_pos == self.data_length);
    }
}
