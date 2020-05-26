use crate::engine::parts as ep;
use crate::gui::constants::*;
use crate::gui::module_widgets;
use crate::gui::{GuiScreen, InteractionHint, MouseMods, Tooltip};
use crate::util::*;

// Describes an action that should be performed on an instance level.
pub enum InstanceAction {
    /// Indicates the structure of the graph has changed and it should be reloaded.
    ReloadStructure,
    /// Indicates a value has changed, so the aux input data should be recollected.
    ReloadAuxData,
    SavePatch,
}

// Describes an action the GUI object should perform. Prevents passing a bunch of arguments to
// MouseAction functions for each action that needs to modify something in the GUI.
pub enum GuiAction {
    OpenMenu(Box<module_widgets::KnobEditor>),
    SwitchScreen(GuiScreen),
    AddModule(ep::Module),
    RemoveModule(Rcrc<ep::Module>),
    Elevate(InstanceAction),
}

pub enum MouseAction {
    None,
    ManipulateControl(Rcrc<ep::Control>, f32),
    ManipulateLane(Rcrc<ep::Control>, usize),
    ManipulateLaneStart(Rcrc<ep::Control>, usize, f32),
    ManipulateLaneEnd(Rcrc<ep::Control>, usize, f32),
    ManipulateIntControl {
        cref: Rcrc<ep::ComplexControl>,
        min: i32,
        max: i32,
        click_delta: i32,
        // The user needs to drag across multiple pixels to increse the value by one. This value
        // keeps track of what the value would be if it were a float and not an int.
        float_value: f32,
    },
    MoveModule(Rcrc<ep::Module>, (i32, i32)),
    PanOffset(Rcrc<(i32, i32)>),
    ConnectInput(Rcrc<ep::Module>, usize),
    ConnectOutput(Rcrc<ep::Module>, usize),
    OpenMenu(Box<module_widgets::KnobEditor>),
    SwitchScreen(GuiScreen),
    AddModule(ep::Module),
    RemoveModule(Rcrc<ep::Module>),
    RemoveLane(Rcrc<ep::Control>, usize),
    SavePatch,
}

impl MouseAction {
    pub fn is_none(&self) -> bool {
        if let Self::None = self {
            true
        } else {
            false
        }
    }

    pub(in crate::gui) fn on_drag(
        &mut self,
        delta: (i32, i32),
        mods: &MouseMods,
    ) -> (Option<GuiAction>, Option<Tooltip>) {
        match self {
            Self::ManipulateControl(control, tracking) => {
                let delta = delta.0 - delta.1;
                // How many pixels the user must drag across to cover the entire range of the knob.
                const DRAG_PIXELS: f32 = 200.0;
                let mut delta = delta as f32 / DRAG_PIXELS;
                if mods.precise {
                    delta *= 0.2;
                }

                let mut control_ref = control.borrow_mut();
                let range = control_ref.range;
                let delta = delta * (range.1 - range.0) as f32;
                *tracking = (*tracking + delta).clam(range.0, range.1);
                if mods.shift {
                    let r08 = tracking.from_range_to_range(range.0, range.1, 0.0, 8.8);
                    let snapped = (r08 - 0.4).round();
                    control_ref.value = snapped.from_range_to_range(0.0, 8.0, range.0, range.1);
                } else {
                    control_ref.value = *tracking;
                }
                for lane in &mut control_ref.automation {
                    lane.range.0 = (lane.range.0 + delta).clam(range.0, range.1);
                    lane.range.1 = (lane.range.1 + delta).clam(range.0, range.1);
                }
                return (
                    Some(GuiAction::Elevate(InstanceAction::ReloadAuxData)),
                    Some(Tooltip {
                        text: format!(
                            "{}{}",
                            format_decimal(control_ref.value, 4),
                            control_ref.suffix
                        ),
                        interaction: InteractionHint::LeftClickAndDrag.into(),
                    }),
                );
            }
            Self::ManipulateLane(control, lane_index) => {
                let delta = delta.0 - delta.1;
                // How many pixels the user must drag across to cover the entire range of the knob.
                const DRAG_PIXELS: f32 = 200.0;
                let mut delta = delta as f32 / DRAG_PIXELS;
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
                    Some(GuiAction::Elevate(InstanceAction::ReloadAuxData)),
                    Some(Tooltip {
                        text: tttext,
                        interaction: InteractionHint::LeftClickAndDrag.into(),
                    }),
                );
            }
            Self::ManipulateLaneStart(control, lane_index, tracking) => {
                let delta = delta.0 - delta.1;
                // How many pixels the user must drag across to cover the entire range of the knob.
                const DRAG_PIXELS: f32 = 200.0;
                let mut delta = delta as f32 / DRAG_PIXELS;
                if mods.precise {
                    delta *= 0.2;
                }

                let mut control_ref = control.borrow_mut();
                let range = control_ref.range;
                let delta = delta * (range.1 - range.0) as f32;
                let lane = &mut control_ref.automation[*lane_index];
                *tracking = (*tracking + delta).clam(range.0, range.1);
                if mods.shift {
                    let r08 = tracking.from_range_to_range(range.0, range.1, 0.0, 8.8);
                    let snapped = (r08 - 0.4).round();
                    lane.range.0 = snapped.from_range_to_range(0.0, 8.0, range.0, range.1);
                } else {
                    lane.range.0 = *tracking;
                }
                let tttext = format!(
                    "{0}{2} to {1}{2}",
                    format_decimal(lane.range.0, 4),
                    format_decimal(lane.range.1, 4),
                    control_ref.suffix,
                );
                return (
                    Some(GuiAction::Elevate(InstanceAction::ReloadAuxData)),
                    Some(Tooltip {
                        text: tttext,
                        interaction: InteractionHint::LeftClickAndDrag.into(),
                    }),
                );
            }
            Self::ManipulateLaneEnd(control, lane_index, tracking) => {
                let delta = delta.0 - delta.1;
                // How many pixels the user must drag across to cover the entire range of the knob.
                const DRAG_PIXELS: f32 = 200.0;
                let mut delta = delta as f32 / DRAG_PIXELS;
                if mods.precise {
                    delta *= 0.2;
                }

                let mut control_ref = control.borrow_mut();
                let range = control_ref.range;
                let delta = delta * (range.1 - range.0) as f32;
                let lane = &mut control_ref.automation[*lane_index];
                *tracking = (*tracking + delta).clam(range.0, range.1);
                if mods.shift {
                    let r08 = tracking.from_range_to_range(range.0, range.1, 0.0, 8.8);
                    let snapped = (r08 - 0.4).round();
                    lane.range.1 = snapped.from_range_to_range(0.0, 8.0, range.0, range.1);
                } else {
                    lane.range.1 = *tracking;
                }
                let tttext = format!(
                    "{0}{2} to {1}{2}",
                    format_decimal(lane.range.0, 4),
                    format_decimal(lane.range.1, 4),
                    control_ref.suffix,
                );
                return (
                    Some(GuiAction::Elevate(InstanceAction::ReloadAuxData)),
                    Some(Tooltip {
                        text: tttext,
                        interaction: InteractionHint::LeftClickAndDrag.into(),
                    }),
                );
            }
            Self::ManipulateIntControl {
                cref,
                min,
                max,
                float_value,
                ..
            } => {
                // How many pixels the user must drag across to change the value by 1.
                const DRAG_PIXELS: f32 = 12.0;
                let delta = delta.0 - delta.1;
                let mut delta = delta as f32 / DRAG_PIXELS;
                if mods.precise {
                    delta *= 0.2;
                }
                *float_value += delta;
                *float_value = float_value.min(*max as f32);
                *float_value = float_value.max(*min as f32);
                let str_value = format!("{}", *float_value as i32);
                if str_value != cref.borrow().value {
                    cref.borrow_mut().value = str_value;
                }
            }
            Self::MoveModule(module, tracking) => {
                *tracking = tracking.add(delta);
                let mut module_ref = module.borrow_mut();
                if mods.shift {
                    const SPACING: i32 = grid(1) + GRID_P;
                    module_ref.pos = tracking.sub((tracking.0 % SPACING, tracking.1 % SPACING));
                } else {
                    module_ref.pos = *tracking;
                }
            }
            Self::PanOffset(offset) => {
                let mut offset_ref = offset.borrow_mut();
                offset_ref.0 += delta.0;
                offset_ref.1 += delta.1;
            }
            _ => (),
        }
        (None, None)
    }

    pub(in crate::gui) fn on_drop(self, target: DropTarget) -> Option<GuiAction> {
        match self {
            Self::ConnectInput(in_module, in_index) => {
                let mut in_ref = in_module.borrow_mut();
                let template_ref = in_ref.template.borrow();
                let in_type = template_ref.inputs[in_index].get_type();
                drop(template_ref);
                if let DropTarget::Output(out_module, out_index) = target {
                    let out_type =
                        out_module.borrow().template.borrow().outputs[out_index].get_type();
                    if in_type == out_type {
                        in_ref.inputs[in_index] = ep::InputConnection::Wire(out_module, out_index);
                    }
                } else {
                    in_ref.inputs[in_index] = ep::InputConnection::Default(0);
                }
                return Some(GuiAction::Elevate(InstanceAction::ReloadStructure));
            }
            Self::ConnectOutput(out_module, out_index) => {
                let out_type = out_module.borrow().template.borrow().outputs[out_index].get_type();
                if let DropTarget::Input(in_module, in_index) = target {
                    let mut in_ref = in_module.borrow_mut();
                    let in_type = in_ref.template.borrow().inputs[in_index].get_type();
                    if in_type == out_type {
                        in_ref.inputs[in_index] = ep::InputConnection::Wire(out_module, out_index);
                    }
                } else if let DropTarget::Control(control) = target {
                    if out_type == ep::JackType::Audio {
                        let mut control_ref = control.borrow_mut();
                        let range = control_ref.range;
                        control_ref.automation.push(ep::AutomationLane {
                            connection: (out_module, out_index),
                            range,
                        });
                    }
                }
                return Some(GuiAction::Elevate(InstanceAction::ReloadStructure));
            }
            Self::ManipulateIntControl { .. } => {
                return Some(GuiAction::Elevate(InstanceAction::ReloadStructure))
            }
            _ => (),
        }
        None
    }

    pub(in crate::gui) fn on_click(self) -> Option<GuiAction> {
        match self {
            Self::OpenMenu(menu) => return Some(GuiAction::OpenMenu(menu)),
            Self::SwitchScreen(screen_index) => return Some(GuiAction::SwitchScreen(screen_index)),
            Self::AddModule(module) => return Some(GuiAction::AddModule(module)),
            Self::RemoveModule(module) => return Some(GuiAction::RemoveModule(module)),
            Self::SavePatch => return Some(GuiAction::Elevate(InstanceAction::SavePatch)),
            Self::RemoveLane(control, lane) => {
                control.borrow_mut().automation.remove(lane);
                return Some(GuiAction::Elevate(InstanceAction::ReloadStructure));
            }
            Self::ConnectInput(module, input_index) => {
                let mut module_ref = module.borrow_mut();
                let num_options = module_ref.template.borrow().inputs[input_index]
                    .borrow_default_options()
                    .len();
                if let ep::InputConnection::Default(index) = &mut module_ref.inputs[input_index] {
                    *index += 1;
                    *index %= num_options;
                }
                return Some(GuiAction::Elevate(InstanceAction::ReloadStructure));
            }
            Self::ManipulateIntControl {
                cref,
                min,
                max,
                click_delta,
                mut float_value,
            } => {
                float_value += click_delta as f32;
                float_value = float_value.min(max as f32);
                float_value = float_value.max(min as f32);
                let str_value = format!("{}", float_value as i32);
                if str_value != cref.borrow().value {
                    cref.borrow_mut().value = str_value;
                }
                return Some(GuiAction::Elevate(InstanceAction::ReloadStructure));
            }
            _ => (),
        }
        None
    }

    pub(in crate::gui) fn on_double_click(self) -> Option<GuiAction> {
        match self {
            Self::ManipulateControl(control, ..) => {
                let mut cref = control.borrow_mut();
                cref.value = cref.default;
                return Some(GuiAction::Elevate(InstanceAction::ReloadAuxData));
            }
            Self::ManipulateLane(control, lane) => {
                let mut cref = control.borrow_mut();
                cref.automation[lane].range = cref.range;
                return Some(GuiAction::Elevate(InstanceAction::ReloadAuxData));
            }
            Self::ManipulateLaneStart(control, lane, ..) => {
                let mut cref = control.borrow_mut();
                cref.automation[lane].range.0 = cref.range.0;
                return Some(GuiAction::Elevate(InstanceAction::ReloadAuxData));
            }
            Self::ManipulateLaneEnd(control, lane, ..) => {
                let mut cref = control.borrow_mut();
                cref.automation[lane].range.1 = cref.range.1;
                return Some(GuiAction::Elevate(InstanceAction::ReloadAuxData));
            }
            Self::ManipulateIntControl { cref, .. } => {
                let mut cref = cref.borrow_mut();
                cref.value = cref.default.clone();
                return Some(GuiAction::Elevate(InstanceAction::ReloadStructure));
            }
            _ => return self.on_click(),
        }
    }
}

pub enum DropTarget {
    None,
    Control(Rcrc<ep::Control>),
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
