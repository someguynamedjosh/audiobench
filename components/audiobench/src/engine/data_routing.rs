use super::parts::{Autocon, Module};
use super::static_controls::Staticon;
use nodespeak::llvmir::structure::OwnedIOData;
use shared_util::prelude::*;

// This packages changes made by the user to knobs and automation into a format that can be read
// by the nodespeak parameter, so that trivial changes don't necessitate a recompile.
pub(super) struct AutoconDynDataCollector {
    ordered_controls: Vec<Rcrc<Autocon>>,
    data_length: usize,
}

impl AutoconDynDataCollector {
    pub(super) fn new(ordered_controls: Vec<Rcrc<Autocon>>, data_length: usize) -> Self {
        Self {
            ordered_controls,
            data_length,
        }
    }

    pub(super) fn collect_data(&self) -> Vec<f32> {
        let mut data = Vec::with_capacity(self.data_length);
        for control in &self.ordered_controls {
            let control_ref = control.borrow();
            if control_ref.automation.len() == 0 {
                data.push(control_ref.value);
            } else {
                let num_lanes = control_ref.automation.len();
                let multiplier = 1.0 / num_lanes as f32;
                for lane in &control_ref.automation {
                    // algebraic simplification of remapping value [-1, 1] -> [0, 1] -> [min, max]
                    let a = (lane.range.1 - lane.range.0) / 2.0;
                    let b = a + lane.range.0;
                    data.push(a * multiplier);
                    data.push(b * multiplier);
                }
            }
        }
        debug_assert!(data.len() == self.data_length);
        data
    }
}

// This packages changes made by the user to static controls into a format that can be read
// by the nodespeak parameter, so that trivial changes don't necessitate a recompile.
pub(super) struct StaticonDynDataCollector {
    ordered_controls: Vec<Rcrc<Staticon>>,
}

impl StaticonDynDataCollector {
    pub(super) fn new(ordered_controls: Vec<Rcrc<Staticon>>) -> Self {
        Self { ordered_controls }
    }

    pub(super) fn collect_data(&self) -> Vec<OwnedIOData> {
        self.ordered_controls
            .iter()
            .filter_map(|staticon| {
                let staticon = staticon.borrow();
                if staticon.is_static_only() {
                    None
                } else {
                    Some(staticon.borrow_data().package_dyn_data().to_owned())
                }
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
        assert_eq!(feedback_data.len(), self.data_length);
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
        debug_assert_eq!(data_pos, self.data_length);
    }
}
