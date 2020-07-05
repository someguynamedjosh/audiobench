use crate::engine::parts as ep;
use crate::gui::constants::*;
use crate::gui::module_widgets;
use crate::gui::ui_widgets::TextField;
use crate::gui::{GuiScreen, InteractionHint, MouseMods, Tooltip};
use crate::registry::save_data::Patch;
use crate::util::*;

// Describes an action that should be performed on an instance level.
pub enum InstanceAction {
    Sequence(Vec<InstanceAction>),
    /// Indicates the structure of the graph has changed and it should be reloaded.
    ReloadStructure,
    /// Indicates a value has changed, so the aux input data should be recollected.
    ReloadAuxData,
    /// Changes the name of the current patch. Asserts if the current patch is not writable.
    RenamePatch(String),
    SavePatch,
    NewPatch(Box<dyn Fn(&Rcrc<Patch>)>),
    LoadPatch(Rcrc<Patch>),
    SimpleCallback(Box<dyn Fn()>),
}

// Describes an action the GUI object should perform. Prevents passing a bunch of arguments to
// MouseAction functions for each action that needs to modify something in the GUI.
pub enum GuiAction {
    Sequence(Vec<GuiAction>),
    OpenMenu(Box<module_widgets::KnobEditor>),
    SwitchScreen(GuiScreen),
    AddModule(ep::Module),
    RemoveModule(Rcrc<ep::Module>),
    FocusTextField(Rcrc<TextField>),
    Elevate(InstanceAction),
}

// TODO: Organize this?
pub enum MouseAction {
    None,
    Sequence(Vec<MouseAction>),
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
    ManipulateHertzControl {
        cref: Rcrc<ep::ComplexControl>,
        min: f32,
        max: f32,
        precise_value: f32,
    },
    ManipulateDurationControl {
        cref: Rcrc<ep::ComplexControl>,
        precise_value: f32,
        denominator: bool,
    },
    SetComplexControl(Rcrc<ep::ComplexControl>, String),
    MoveModule(Rcrc<ep::Module>, (f32, f32)),
    PanOffset(Rcrc<(f32, f32)>),
    ConnectInput(Rcrc<ep::Module>, usize),
    ConnectOutput(Rcrc<ep::Module>, usize),
    OpenMenu(Box<module_widgets::KnobEditor>),
    SwitchScreen(GuiScreen),
    AddModule(ep::Module),
    RemoveModule(Rcrc<ep::Module>),
    RemoveLane(Rcrc<ep::Control>, usize),
    RenamePatch(String),
    SavePatch,
    NewPatch(Box<dyn Fn(&Rcrc<Patch>)>),
    LoadPatch(Rcrc<Patch>),
    FocusTextField(Rcrc<TextField>),
    Scaled(Box<MouseAction>, Rcrc<f32>),
    SimpleCallback(Box<dyn Fn()>),
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
                // How many pixels the user must drag across to cover the entire range of the knob.
                const DRAG_PIXELS: f32 = 200.0;
                let mut delta = delta / DRAG_PIXELS;
                if mods.precise {
                    delta *= 0.2;
                }

                let mut control_ref = control.borrow_mut();
                let range = control_ref.range;
                let delta = delta * (range.1 - range.0) as f32;
                *tracking = (*tracking + delta).clam(range.0, range.1);
                let steps = if mods.precise { 60.0 } else { 12.0 };
                if mods.shift {
                    let r08 = tracking.from_range_to_range(range.0, range.1, 0.0, steps + 0.8);
                    let snapped = (r08 - 0.4).round();
                    control_ref.value = snapped.from_range_to_range(0.0, steps, range.0, range.1);
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
                        interaction: InteractionHint::LeftClickAndDrag
                            | InteractionHint::Alt
                            | InteractionHint::Shift,
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
                let steps = if mods.precise { 60.0 } else { 12.0 };
                if mods.shift {
                    let r08 = tracking.from_range_to_range(range.0, range.1, 0.0, steps + 0.8);
                    let snapped = (r08 - 0.4).round();
                    lane.range.0 = snapped.from_range_to_range(0.0, steps, range.0, range.1);
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
                        interaction: InteractionHint::LeftClickAndDrag
                            | InteractionHint::Alt
                            | InteractionHint::Shift,
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
                let steps = if mods.precise { 60.0 } else { 12.0 };
                if mods.shift {
                    let r08 = tracking.from_range_to_range(range.0, range.1, 0.0, steps + 0.8);
                    let snapped = (r08 - 0.4).round();
                    lane.range.1 = snapped.from_range_to_range(0.0, steps, range.0, range.1);
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
                        interaction: InteractionHint::LeftClickAndDrag
                            | InteractionHint::Alt
                            | InteractionHint::Shift,
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
            Self::ManipulateHertzControl {
                cref,
                min,
                max,
                precise_value,
            } => {
                // How many pixels the user must drag across to change the value by an octave.
                const PIXELS_PER_OCTAVE: f32 = 40.0;
                let delta = delta.0 - delta.1;
                let mut delta = delta as f32 / PIXELS_PER_OCTAVE;
                if mods.precise {
                    delta *= 0.2;
                }
                let multiplier = (2.0f32).powf(delta);
                *precise_value *= multiplier;
                *precise_value = precise_value.min(*max as f32);
                *precise_value = precise_value.max(*min as f32);
                let decimals = if *precise_value < 1.0 {
                    4
                } else if *precise_value < 10.0 {
                    3
                } else if *precise_value < 100.0 {
                    2
                } else if *precise_value < 1000.0 {
                    1
                } else {
                    0
                };
                let str_value = format!("{:.1$}", *precise_value, decimals);
                if str_value != cref.borrow().value {
                    cref.borrow_mut().value = str_value;
                }
            }
            Self::ManipulateDurationControl {
                cref,
                precise_value,
                denominator,
            } => {
                // If we are editing a fraction...
                if cref.borrow().value.contains("/") {
                    // How many pixels the user must drag across to change the value by 1.
                    const DRAG_PIXELS: f32 = 12.0;
                    let delta = delta.0 - delta.1;
                    let mut delta = delta as f32 / DRAG_PIXELS;
                    if mods.precise {
                        delta *= 0.2;
                    }
                    *precise_value += delta;
                    *precise_value = precise_value.min(98.0);
                    *precise_value = precise_value.max(1.0);
                    let str_value = cref.borrow().value.clone();
                    let slash_pos = str_value.find('/').unwrap();
                    let mut num = (str_value[..slash_pos]).parse::<f32>().unwrap() as i32;
                    let mut den = (str_value[slash_pos + 1..]).parse::<f32>().unwrap() as i32;
                    if *denominator {
                        den = *precise_value as i32;
                    } else {
                        num = *precise_value as i32;
                    }
                    let str_value = format!("{}.0/{}.0", num, den);
                    if str_value != cref.borrow().value {
                        cref.borrow_mut().value = str_value;
                    }
                } else {
                    // How many pixels the user must drag across to double the value
                    const PIXELS_PER_OCTAVE: f32 = 40.0;
                    let delta = delta.0 - delta.1;
                    let mut delta = delta as f32 / PIXELS_PER_OCTAVE;
                    if mods.precise {
                        delta *= 0.2;
                    }
                    let multiplier = (2.0f32).powf(delta);
                    *precise_value *= multiplier;
                    *precise_value = precise_value.min(99.8);
                    *precise_value = precise_value.max(0.0003);
                    let decimals = if *precise_value < 0.999 {
                        3
                    } else if *precise_value < 9.99 {
                        2
                    } else if *precise_value < 99.9 {
                        1
                    } else {
                        0
                    };
                    let str_value = format!("{:.1$}", *precise_value, decimals);
                    if str_value != cref.borrow().value {
                        cref.borrow_mut().value = str_value;
                    }
                }
                return (
                    None,
                    Some(Tooltip {
                        text: "".to_owned(),
                        interaction: InteractionHint::LeftClickAndDrag | InteractionHint::Alt,
                    }),
                );
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
                // Only change to a default if it used to be connected.
                } else if let ep::InputConnection::Wire(..) = &in_ref.inputs[in_index] {
                    let default = in_ref.template.borrow().default_inputs[in_index];
                    in_ref.inputs[in_index] = ep::InputConnection::Default(default);
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
            Self::ManipulateHertzControl { .. } => {
                return Some(GuiAction::Elevate(InstanceAction::ReloadStructure))
            }
            Self::ManipulateDurationControl { .. } => {
                return Some(GuiAction::Elevate(InstanceAction::ReloadStructure))
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
            Self::SavePatch => return Some(GuiAction::Elevate(InstanceAction::SavePatch)),
            Self::NewPatch(callback) => {
                return Some(GuiAction::Elevate(InstanceAction::NewPatch(callback)))
            }
            Self::LoadPatch(patch) => {
                return Some(GuiAction::Elevate(InstanceAction::LoadPatch(patch)))
            }
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
            Self::SetComplexControl(control, value) => {
                let mut control_ref = control.borrow_mut();
                if control_ref.value != value {
                    control_ref.value = value;
                    return Some(GuiAction::Elevate(InstanceAction::ReloadStructure));
                    // If not, don't bother reloading the structure, which causes a notable hiccup.
                }
            }
            Self::Scaled(base, ..) => {
                return base.on_click();
            }
            Self::SimpleCallback(callback) => {
                return Some(GuiAction::Elevate(InstanceAction::SimpleCallback(callback)));
            }
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
            Self::ManipulateIntControl { cref, .. } | Self::ManipulateHertzControl { cref, .. } => {
                let mut cref = cref.borrow_mut();
                cref.value = cref.default.clone();
                return Some(GuiAction::Elevate(InstanceAction::ReloadStructure));
            }
            Self::SetComplexControl(control, ..) => {
                let mut control_ref = control.borrow_mut();
                let value = control_ref.default.clone();
                if control_ref.value != value {
                    control_ref.value = value;
                    return Some(GuiAction::Elevate(InstanceAction::ReloadStructure));
                    // If not, don't bother reloading the structure, which causes a notable hiccup.
                }
            }
            Self::Scaled(base, ..) => {
                return base.on_double_click();
            }
            Self::SimpleCallback(callback) => {
                return Some(GuiAction::Elevate(InstanceAction::SimpleCallback(callback)));
            }
            _ => return self.on_click(),
        }
        None
    }
}

#[derive(Clone)]
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
