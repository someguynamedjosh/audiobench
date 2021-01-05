use crate::engine::controls::{FloatInRangeControl, UpdateRequest};
use crate::engine::parts as ep;
use crate::gui::constants::*;
use crate::gui::module_widgets;
use crate::gui::ui_widgets::TextField;
use crate::gui::{GuiScreen, InteractionHint, MouseMods, Tooltip};
use crate::registry::save_data::Patch;
use shared_util::prelude::*;

// Describes an action that should be performed on an instance level.
pub enum InstanceAction {
    Sequence(Vec<InstanceAction>),
    /// Indicates the structure of the graph has changed and it should be reloaded.
    ReloadStructure,
    /// Indicates a value has changed, so the aux input data should be recollected.
    ReloadFloatInRangeControlDynData,
    ReloadControlDynData,
    /// Changes the name of the current patch. Asserts if the current patch is not writable.
    RenamePatch(String),
    SavePatch(Box<dyn FnMut(&Rcrc<Patch>)>),
    NewPatch(Box<dyn FnMut(&Rcrc<Patch>)>),
    LoadPatch(Rcrc<Patch>, Box<dyn FnMut()>),
    SimpleCallback(Box<dyn FnMut()>),
    CopyPatchToClipboard,
    PastePatchFromClipboard(Box<dyn FnMut(&Rcrc<Patch>)>),
}

// Describes an action the GUI object should perform. Prevents passing a bunch of arguments to
// MouseAction functions for each action that needs to modify something in the GUI.
pub enum GuiAction {
    Sequence(Vec<GuiAction>),
    OpenMenu(Box<dyn module_widgets::PopupMenu>),
    SwitchScreen(GuiScreen),
    AddModule(ep::Module),
    RemoveModule(Rcrc<ep::Module>),
    FocusTextField(Rcrc<TextField>),
    Elevate(InstanceAction),
    OpenWebpage(String),
}

// TODO: Organize this?
pub enum MouseAction {
    None,
    Sequence(Vec<MouseAction>),
    ManipulateControl(Rcrc<FloatInRangeControl>, f32),
    ManipulateLane(Rcrc<FloatInRangeControl>, usize),
    ManipulateLaneStart(Rcrc<FloatInRangeControl>, usize, f32),
    ManipulateLaneEnd(Rcrc<FloatInRangeControl>, usize, f32),
    ManipulateIntBox {
        callback: Box<dyn FnMut(i32) -> UpdateRequest>,
        min: i32,
        max: i32,
        click_delta: i32,
        // The user needs to drag across multiple pixels to increse the value by one. This value
        // keeps track of what the value would be if it were a float and not an int.
        float_value: f32,
        code_reload_requested: bool,
    },
    MutateControl(Box<dyn FnOnce() -> UpdateRequest>),
    ContinuouslyMutateControl {
        mutator: Box<dyn FnMut(f32, Option<f32>) -> (UpdateRequest, Option<Tooltip>)>,
        code_reload_requested: bool,
    },
    MoveModule(Rcrc<ep::Module>, (f32, f32)),
    PanOffset(Rcrc<(f32, f32)>),
    ConnectInput(Rcrc<ep::Module>, usize),
    ConnectOutput(Rcrc<ep::Module>, usize),
    OpenMenu(Box<dyn module_widgets::PopupMenu>),
    SwitchScreen(GuiScreen),
    AddModule(ep::Module),
    RemoveModule(Rcrc<ep::Module>),
    RemoveLane(Rcrc<FloatInRangeControl>, usize),
    RenamePatch(String),
    SavePatch(Box<dyn FnMut(&Rcrc<Patch>)>),
    NewPatch(Box<dyn FnMut(&Rcrc<Patch>)>),
    LoadPatch(Rcrc<Patch>, Box<dyn FnMut()>),
    FocusTextField(Rcrc<TextField>),
    Scaled(Box<MouseAction>, Rcrc<f32>),
    SimpleCallback(Box<dyn FnMut()>),
    CopyPatchToClipboard,
    PastePatchFromClipboard(Box<dyn FnMut(&Rcrc<Patch>)>),
    OpenWebpage(String),
}

fn get_snap_steps(mods: &MouseMods) -> Option<f32> {
    if mods.shift {
        Some(if mods.precise {
            PRECISE_SNAP_STEPS
        } else {
            SNAP_STEPS
        })
    } else {
        None
    }
}

fn maybe_snap_value(value: f32, range: (f32, f32), mods: &MouseMods) -> f32 {
    if let Some(steps) = get_snap_steps(mods) {
        value.snap(range.0, range.1, steps)
    } else {
        value
    }
}

impl MouseAction {
    pub fn is_none(&self) -> bool {
        if let Self::None = self {
            true
        } else {
            false
        }
    }

    // This will return self if !self.allow_scaled()
    pub fn scaled(self, scale: Rcrc<f32>) -> Self {
        if self.allow_scaled() {
            Self::Scaled(Box::new(self), scale)
        } else {
            self
        }
    }

    pub fn allow_scaled(&self) -> bool {
        match self {
            Self::MoveModule(..) => true,
            Self::PanOffset(..) => true,
            _ => false,
        }
    }

    pub(in crate::gui) fn on_drag(
        &mut self,
        delta: (f32, f32),
        mods: &MouseMods,
    ) -> (Option<GuiAction>, Option<Tooltip>) {
        match self {
            Self::Sequence(actions) => {
                for action in actions {
                    action.on_drag(delta, mods);
                }
            }
            Self::ManipulateControl(control, tracking) => {
                let delta = delta.0 - delta.1;
                let mut delta = delta / RANGE_DRAG_PIXELS;
                if mods.precise {
                    delta *= 0.2;
                }

                let mut control_ref = control.borrow_mut();
                let range = control_ref.range;
                let delta = delta * (range.1 - range.0) as f32;
                *tracking = (*tracking + delta).clam(range.0, range.1);
                control_ref.value = maybe_snap_value(*tracking, range, mods);
                for lane in &mut control_ref.automation {
                    lane.range.0 = (lane.range.0 + delta).clam(range.0, range.1);
                    lane.range.1 = (lane.range.1 + delta).clam(range.0, range.1);
                }
                return (
                    Some(GuiAction::Elevate(InstanceAction::ReloadFloatInRangeControlDynData)),
                    Some(Tooltip {
                        text: format!(
                            "{}{}",
                            format_decimal(control_ref.value, 4),
                            control_ref.suffix
                        ),
                        interaction: InteractionHint::LeftClickAndDrag
                            | InteractionHint::Alt
                            | InteractionHint::Shift,
                    }),
                );
            }
            Self::ManipulateLane(control, lane_index) => {
                let delta = delta.0 - delta.1;
                let mut delta = delta as f32 / RANGE_DRAG_PIXELS;
                if mods.precise {
                    delta *= 0.2;
                }

                let mut control_ref = control.borrow_mut();
                let range = control_ref.range;
                let delta = delta * (range.1 - range.0) as f32;
                let lane = &mut control_ref.automation[*lane_index];
                lane.range.0 = (lane.range.0 + delta).clam(range.0, range.1);
                lane.range.1 = (lane.range.1 + delta).clam(range.0, range.1);
                let tttext = format!(
                    "{0}{2} to {1}{2}",
                    format_decimal(lane.range.0, 4),
                    format_decimal(lane.range.1, 4),
                    control_ref.suffix,
                );
                return (
                    Some(GuiAction::Elevate(InstanceAction::ReloadFloatInRangeControlDynData)),
                    Some(Tooltip {
                        text: tttext,
                        interaction: InteractionHint::LeftClickAndDrag.into(),
                    }),
                );
            }
            Self::ManipulateLaneStart(control, lane_index, tracking) => {
                let delta = delta.0 - delta.1;
                let mut delta = delta as f32 / RANGE_DRAG_PIXELS;
                if mods.precise {
                    delta *= 0.2;
                }

                let mut control_ref = control.borrow_mut();
                let range = control_ref.range;
                let delta = delta * (range.1 - range.0) as f32;
                let lane = &mut control_ref.automation[*lane_index];
                *tracking = (*tracking + delta).clam(range.0, range.1);
                lane.range.0 = maybe_snap_value(*tracking, range, mods);
                let tttext = format!(
                    "{0}{2} to {1}{2}",
                    format_decimal(lane.range.0, 4),
                    format_decimal(lane.range.1, 4),
                    control_ref.suffix,
                );
                return (
                    Some(GuiAction::Elevate(InstanceAction::ReloadFloatInRangeControlDynData)),
                    Some(Tooltip {
                        text: tttext,
                        interaction: InteractionHint::LeftClickAndDrag
                            | InteractionHint::Alt
                            | InteractionHint::Shift,
                    }),
                );
            }
            Self::ManipulateLaneEnd(control, lane_index, tracking) => {
                let delta = delta.0 - delta.1;
                let mut delta = delta as f32 / RANGE_DRAG_PIXELS;
                if mods.precise {
                    delta *= 0.2;
                }

                let mut control_ref = control.borrow_mut();
                let range = control_ref.range;
                let delta = delta * (range.1 - range.0) as f32;
                let lane = &mut control_ref.automation[*lane_index];
                *tracking = (*tracking + delta).clam(range.0, range.1);
                lane.range.1 = maybe_snap_value(*tracking, range, mods);
                let tttext = format!(
                    "{0}{2} to {1}{2}",
                    format_decimal(lane.range.0, 4),
                    format_decimal(lane.range.1, 4),
                    control_ref.suffix,
                );
                return (
                    Some(GuiAction::Elevate(InstanceAction::ReloadFloatInRangeControlDynData)),
                    Some(Tooltip {
                        text: tttext,
                        interaction: InteractionHint::LeftClickAndDrag
                            | InteractionHint::Alt
                            | InteractionHint::Shift,
                    }),
                );
            }
            Self::ContinuouslyMutateControl {
                mutator,
                code_reload_requested,
            } => {
                let mut delta = delta.0 - delta.1;
                if mods.precise {
                    delta *= 0.2;
                }
                let (update, tooltip) = mutator(delta, get_snap_steps(mods));
                let action = match update {
                    UpdateRequest::Nothing => None,
                    UpdateRequest::UpdateDynData => {
                        Some(GuiAction::Elevate(InstanceAction::ReloadControlDynData))
                    }
                    UpdateRequest::UpdateCode => {
                        *code_reload_requested = true;
                        None
                    }
                };
                return (action, tooltip);
            }
            Self::ManipulateIntBox {
                callback,
                min,
                max,
                float_value,
                code_reload_requested,
                ..
            } => {
                let delta = delta.0 - delta.1;
                let mut delta = delta as f32 / DISCRETE_STEP_PIXELS;
                if mods.precise {
                    delta *= 0.2;
                }
                *float_value += delta;
                *float_value = float_value.min(*max as f32);
                *float_value = float_value.max(*min as f32);
                match callback(*float_value as i32) {
                    UpdateRequest::Nothing => (),
                    UpdateRequest::UpdateDynData => {
                        return (
                            Some(GuiAction::Elevate(InstanceAction::ReloadControlDynData)),
                            None,
                        );
                    }
                    UpdateRequest::UpdateCode => {
                        *code_reload_requested = true;
                    }
                }
            }
            Self::MoveModule(module, tracking) => {
                *tracking = tracking.add(delta);
                let mut module_ref = module.borrow_mut();
                if mods.shift {
                    const SPACING: f32 = (grid(1) + GRID_P) as f32;
                    module_ref.pos = tracking.sub((tracking.0 % SPACING, tracking.1 % SPACING));
                } else {
                    module_ref.pos = *tracking;
                }
                return (
                    None,
                    Some(Tooltip {
                        text: "".to_owned(),
                        interaction: InteractionHint::LeftClickAndDrag | InteractionHint::Shift,
                    }),
                );
            }
            Self::PanOffset(offset) => {
                let mut offset_ref = offset.borrow_mut();
                offset_ref.0 += delta.0;
                offset_ref.1 += delta.1;
            }
            Self::Scaled(base, scale) => {
                let scale = *scale.borrow();
                return base.on_drag((delta.0 / scale, delta.1 / scale), mods);
            }
            _ => (),
        }
        (None, None)
    }

    pub(in crate::gui) fn on_drop(self, target: DropTarget) -> Option<GuiAction> {
        match self {
            Self::Sequence(actions) => {
                return Some(GuiAction::Sequence(
                    actions
                        .into_iter()
                        .filter_map(|action| action.on_drop(target.clone()))
                        .collect(),
                ));
            }
            Self::ConnectInput(in_module, in_index) => {
                unimplemented!();
                // let mut in_ref = in_module.borrow_mut();
                // let template_ref = in_ref.template.borrow();
                // let in_type = template_ref.inputs[in_index].get_type();
                // drop(template_ref);
                // if let DropTarget::Output(out_module, out_index) = target {
                //     let out_type =
                //         out_module.borrow().template.borrow().outputs[out_index].get_type();
                //     if in_type == out_type {
                //         in_ref.inputs[in_index] = ep::InputConnection::Wire(out_module, out_index);
                //     }
                // // Only change to a default if it used to be connected.
                // } else if let ep::InputConnection::Wire(..) = &in_ref.inputs[in_index] {
                //     let default = in_ref.template.borrow().default_inputs[in_index];
                //     in_ref.inputs[in_index] = ep::InputConnection::Default(default);
                // }
                // return Some(GuiAction::Elevate(InstanceAction::ReloadStructure));
            }
            Self::ConnectOutput(out_module, out_index) => {
                unimplemented!();
                // let out_type = out_module.borrow().template.borrow().outputs[out_index].get_type();
                // if let DropTarget::Input(in_module, in_index) = target {
                //     let mut in_ref = in_module.borrow_mut();
                //     let in_type = in_ref.template.borrow().inputs[in_index].get_type();
                //     if in_type == out_type {
                //         in_ref.inputs[in_index] = ep::InputConnection::Wire(out_module, out_index);
                //     }
                // } else if let DropTarget::FloatInRangeControl(control) = target {
                //     if out_type == ep::JackType::Audio {
                //         let mut control_ref = control.borrow_mut();
                //         let range = control_ref.range;
                //         control_ref.automation.push(ep::AutomationLane {
                //             connection: (out_module, out_index),
                //             range,
                //         });
                //     }
                // }
                // return Some(GuiAction::Elevate(InstanceAction::ReloadStructure));
            }
            Self::ManipulateIntBox {
                mut callback,
                float_value,
                code_reload_requested,
                ..
            } => {
                let request = callback(float_value as i32);
                // This takes priority over everything.
                if code_reload_requested {
                    return Some(GuiAction::Elevate(InstanceAction::ReloadStructure));
                }
                return match request {
                    UpdateRequest::Nothing => None,
                    UpdateRequest::UpdateDynData => {
                        Some(GuiAction::Elevate(InstanceAction::ReloadControlDynData))
                    }
                    UpdateRequest::UpdateCode => {
                        Some(GuiAction::Elevate(InstanceAction::ReloadStructure))
                    }
                };
            }
            Self::Scaled(base, ..) => {
                return base.on_drop(target);
            }
            Self::SimpleCallback(callback) => {
                return Some(GuiAction::Elevate(InstanceAction::SimpleCallback(callback)));
            }
            _ => (),
        }
        None
    }

    pub(in crate::gui) fn on_click(self) -> Option<GuiAction> {
        match self {
            Self::Sequence(actions) => {
                return Some(GuiAction::Sequence(
                    actions
                        .into_iter()
                        .filter_map(|action| action.on_click())
                        .collect(),
                ));
            }
            Self::OpenMenu(menu) => return Some(GuiAction::OpenMenu(menu)),
            Self::SwitchScreen(screen_index) => return Some(GuiAction::SwitchScreen(screen_index)),
            Self::AddModule(module) => return Some(GuiAction::AddModule(module)),
            Self::RemoveModule(module) => return Some(GuiAction::RemoveModule(module)),
            Self::FocusTextField(field) => return Some(GuiAction::FocusTextField(field)),
            Self::RenamePatch(name) => {
                return Some(GuiAction::Elevate(InstanceAction::RenamePatch(name)))
            }
            Self::SavePatch(callback) => {
                return Some(GuiAction::Elevate(InstanceAction::SavePatch(callback)))
            }
            Self::NewPatch(callback) => {
                return Some(GuiAction::Elevate(InstanceAction::NewPatch(callback)))
            }
            Self::LoadPatch(patch, callback) => {
                return Some(GuiAction::Elevate(InstanceAction::LoadPatch(
                    patch, callback,
                )))
            }
            Self::RemoveLane(control, lane) => {
                control.borrow_mut().automation.remove(lane);
                return Some(GuiAction::Elevate(InstanceAction::ReloadStructure));
            }
            Self::ConnectInput(module, input_index) => {
                unimplemented!();
                // let mut module_ref = module.borrow_mut();
                // let num_options = module_ref.template.borrow().inputs[input_index]
                //     .borrow_default_options()
                //     .len();
                // if let ep::InputConnection::Default(index) = &mut module_ref.inputs[input_index] {
                //     *index += 1;
                //     *index %= num_options;
                // }
                // return Some(GuiAction::Elevate(InstanceAction::ReloadStructure));
            }
            Self::ManipulateIntBox {
                mut callback,
                min,
                max,
                click_delta,
                mut float_value,
                code_reload_requested,
            } => {
                float_value += click_delta as f32;
                float_value = float_value.min(max as f32);
                float_value = float_value.max(min as f32);
                let request = callback(float_value as i32);
                // This takes priority over everything.
                if code_reload_requested {
                    return Some(GuiAction::Elevate(InstanceAction::ReloadStructure));
                }
                return match request {
                    UpdateRequest::Nothing => None,
                    UpdateRequest::UpdateDynData => {
                        Some(GuiAction::Elevate(InstanceAction::ReloadControlDynData))
                    }
                    UpdateRequest::UpdateCode => {
                        Some(GuiAction::Elevate(InstanceAction::ReloadStructure))
                    }
                };
            }
            Self::MutateControl(mutator) => {
                return match mutator() {
                    UpdateRequest::Nothing => None,
                    UpdateRequest::UpdateDynData => {
                        Some(GuiAction::Elevate(InstanceAction::ReloadControlDynData))
                    }
                    UpdateRequest::UpdateCode => {
                        Some(GuiAction::Elevate(InstanceAction::ReloadStructure))
                    }
                };
            }
            Self::Scaled(base, ..) => {
                return base.on_click();
            }
            Self::SimpleCallback(callback) => {
                return Some(GuiAction::Elevate(InstanceAction::SimpleCallback(callback)));
            }
            Self::CopyPatchToClipboard => {
                return Some(GuiAction::Elevate(InstanceAction::CopyPatchToClipboard))
            }
            Self::PastePatchFromClipboard(callback) => {
                return Some(GuiAction::Elevate(InstanceAction::PastePatchFromClipboard(
                    callback,
                )))
            }
            Self::OpenWebpage(url) => return Some(GuiAction::OpenWebpage(url)),
            _ => (),
        }
        None
    }

    pub(in crate::gui) fn on_double_click(self) -> Option<GuiAction> {
        match self {
            Self::Sequence(actions) => {
                return Some(GuiAction::Sequence(
                    actions
                        .into_iter()
                        .filter_map(|action| action.on_double_click())
                        .collect(),
                ));
            }
            Self::ManipulateControl(control, ..) => {
                let mut cref = control.borrow_mut();
                cref.value = cref.default;
                return Some(GuiAction::Elevate(InstanceAction::ReloadFloatInRangeControlDynData));
            }
            Self::ManipulateLane(control, lane) => {
                let mut cref = control.borrow_mut();
                cref.automation[lane].range = cref.range;
                return Some(GuiAction::Elevate(InstanceAction::ReloadFloatInRangeControlDynData));
            }
            Self::ManipulateLaneStart(control, lane, ..) => {
                let mut cref = control.borrow_mut();
                cref.automation[lane].range.0 = cref.range.0;
                return Some(GuiAction::Elevate(InstanceAction::ReloadFloatInRangeControlDynData));
            }
            Self::ManipulateLaneEnd(control, lane, ..) => {
                let mut cref = control.borrow_mut();
                cref.automation[lane].range.1 = cref.range.1;
                return Some(GuiAction::Elevate(InstanceAction::ReloadFloatInRangeControlDynData));
            }
            Self::Scaled(base, ..) => {
                return base.on_double_click();
            }
            Self::SimpleCallback(callback) => {
                return Some(GuiAction::Elevate(InstanceAction::SimpleCallback(callback)));
            }
            _ => return self.on_click(),
        }
    }
}

#[derive(Clone)]
pub enum DropTarget {
    None,
    FloatInRangeControl(Rcrc<FloatInRangeControl>),
    Input(Rcrc<ep::Module>, usize),
    Output(Rcrc<ep::Module>, usize),
}

impl DropTarget {
    pub fn is_none(&self) -> bool {
        if let Self::None = self {
            true
        } else {
            false
        }
    }
}
