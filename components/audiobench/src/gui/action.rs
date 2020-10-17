use crate::engine::parts as ep;
use crate::engine::static_controls as staticons;
use crate::gui::constants::*;
use crate::gui::module_widgets;
use crate::gui::ui_widgets::TextField;
use crate::gui::{GuiScreen, InteractionHint, MouseMods, Tooltip};
use crate::registry::save_data::Patch;
use scones::make_constructor;
use shared_util::prelude::*;

// Describes an action that should be performed on an instance level.
pub enum InstanceRequest {
    /// Indicates the structure of the graph has changed and it should be reloaded.
    ReloadStructure,
    /// Indicates a value has changed, so the aux input data should be recollected.
    ReloadAutoconDynData,
    ReloadStaticonDynData,
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
pub enum GuiRequest {
    ShowTooltip(Tooltip),
    OpenMenu(Box<dyn module_widgets::PopupMenu>),
    SwitchScreen(GuiScreen),
    AddModule(ep::Module),
    RemoveModule(Rcrc<ep::Module>),
    FocusTextField(Rcrc<TextField>),
    Elevate(InstanceRequest),
    OpenWebpage(String),
}

impl From<Tooltip> for GuiRequest {
    fn from(other: Tooltip) -> Self {
        Self::ShowTooltip(other)
    }
}

impl From<InstanceRequest> for GuiRequest {
    fn from(other: InstanceRequest) -> Self {
        Self::Elevate(other)
    }
}

pub trait MouseAction {
    fn allow_scaled(&self) -> bool {
        false
    }

    fn on_drag(&mut self, _delta: (f32, f32), _mods: &MouseMods) -> Vec<GuiRequest> {
        vec![]
    }

    fn on_drop(self: Box<Self>, _target: DropTarget) -> Vec<GuiRequest> {
        vec![]
    }

    fn on_click(self: Box<Self>) -> Vec<GuiRequest> {
        vec![]
    }

    fn on_double_click(self: Box<Self>) -> Vec<GuiRequest> {
        vec![]
    }
}

pub struct ScaledMouseAction {
    action: Box<dyn MouseAction>,
    scale: Rcrc<f32>,
}

impl ScaledMouseAction {
    /// This will return action if !action.allow_scaled()
    pub fn new(action: Box<dyn MouseAction>, scale: Rcrc<f32>) -> Box<dyn MouseAction> {
        if action.allow_scaled() {
            Box::new(ScaledMouseAction { action, scale })
        } else {
            action
        }
    }
}

impl MouseAction for ScaledMouseAction {
    fn on_drag(&mut self, delta: (f32, f32), mods: &MouseMods) -> Vec<GuiRequest> {
        let delta = delta.scale(1.0 / *self.scale.borrow());
        self.action.on_drag(delta, mods)
    }

    fn on_drop(self: Box<Self>, target: DropTarget) -> Vec<GuiRequest> {
        self.action.on_drop(target)
    }

    fn on_click(self: Box<Self>) -> Vec<GuiRequest> {
        self.action.on_click()
    }

    fn on_double_click(self: Box<Self>) -> Vec<GuiRequest> {
        self.action.on_double_click()
    }
}

pub struct ManipulateControl {
    current_value: f32,
    control: Rcrc<ep::Autocon>,
}

impl ManipulateControl {
    pub fn new(control: Rcrc<ep::Autocon>) -> Self {
        let current_value = control.borrow().value;
        Self {
            current_value,
            control,
        }
    }
}

impl MouseAction for ManipulateControl {
    fn on_drag(&mut self, delta: (f32, f32), mods: &MouseMods) -> Vec<GuiRequest> {
        let delta = range_drag_delta(delta, mods);
        let mut control_ref = self.control.borrow_mut();
        let range = control_ref.range;
        let delta = delta * (range.1 - range.0) as f32;
        self.current_value = (self.current_value + delta).clam(range.0, range.1);
        control_ref.value = maybe_snap_value(self.current_value, range, mods);
        for lane in &mut control_ref.automation {
            lane.range.0 = (lane.range.0 + delta).clam(range.0, range.1);
            lane.range.1 = (lane.range.1 + delta).clam(range.0, range.1);
        }
        vec![
            InstanceRequest::ReloadAutoconDynData.into(),
            range_value_tooltip(control_ref.value, &control_ref.suffix),
        ]
    }

    fn on_double_click(self: Box<Self>) -> Vec<GuiRequest> {
        let mut cref = self.control.borrow_mut();
        cref.value = cref.default;
        return vec![InstanceRequest::ReloadAutoconDynData.into()];
    }
}

#[make_constructor]
#[make_constructor(pub start_only)]
#[make_constructor(pub end_only)]
pub struct ManipulateLane {
    control: Rcrc<ep::Autocon>,
    lane: usize,
    #[value(true)]
    #[value(false for end_only)]
    start: bool,
    #[value(true)]
    #[value(false for start_only)]
    end: bool,
}

impl MouseAction for ManipulateLane {
    fn on_drag(&mut self, delta: (f32, f32), mods: &MouseMods) -> Vec<GuiRequest> {
        let delta = range_drag_delta(delta, mods);
        let mut control_ref = self.control.borrow_mut();
        let range = control_ref.range;
        let delta = delta * (range.1 - range.0) as f32;
        let lane = &mut control_ref.automation[self.lane];
        if self.start {
            lane.range.0 =
                maybe_snap_value((lane.range.0 + delta).clam(range.0, range.1), range, mods);
        }
        if self.end {
            lane.range.1 =
                maybe_snap_value((lane.range.1 + delta).clam(range.0, range.1), range, mods);
        }
        let tttext = format!(
            "{0}{2} to {1}{2}",
            format_decimal(lane.range.0, 4),
            format_decimal(lane.range.1, 4),
            control_ref.suffix,
        );
        vec![
            InstanceRequest::ReloadAutoconDynData.into(),
            Tooltip {
                text: tttext,
                interaction: InteractionHint::LeftClickAndDrag
                    | InteractionHint::PrecisionModifier
                    | InteractionHint::SnappingModifier,
            }
            .into(),
        ]
    }

    fn on_double_click(self: Box<Self>) -> Vec<GuiRequest> {
        let mut cref = self.control.borrow_mut();
        let range = cref.range;
        let lane = &mut cref.automation[self.lane];
        if self.start {
            lane.range.0 = range.0
        };
        if self.end {
            lane.range.1 = range.1
        };
        return vec![InstanceRequest::ReloadAutoconDynData.into()];
    }
}

#[make_constructor((.., current_value: i32))]
pub struct ManipulateIntBox {
    callback: Box<dyn FnMut(i32) -> staticons::StaticonUpdateRequest>,
    min: i32,
    max: i32,
    click_delta: i32,
    // The user needs to drag across multiple pixels to increse the value by one. This value
    // keeps track of what the value would be if it were a float and not an int.
    #[value(current_value as f32)]
    float_value: f32,
    #[value(false)]
    code_reload_requested: bool,
}

impl MouseAction for ManipulateIntBox {
    fn on_drag(&mut self, delta: (f32, f32), mods: &MouseMods) -> Vec<GuiRequest> {
        let delta = delta.0 - delta.1;
        let mut delta = delta as f32 / DISCRETE_STEP_PIXELS;
        if mods.precise {
            delta *= PRECISION_MULTIPLIER;
        }
        self.float_value += delta;
        self.float_value = self.float_value.clam(self.min as f32, self.max as f32);
        match (self.callback)(self.float_value as i32) {
            staticons::StaticonUpdateRequest::Nothing => vec![],
            staticons::StaticonUpdateRequest::UpdateDynData => {
                vec![InstanceRequest::ReloadStaticonDynData.into()]
            }
            staticons::StaticonUpdateRequest::UpdateCode => {
                self.code_reload_requested = true;
                vec![]
            }
        }
    }

    fn on_drop(mut self: Box<Self>, _target: DropTarget) -> Vec<GuiRequest> {
        let request = (self.callback)(self.float_value as i32);
        if self.code_reload_requested {
            vec![InstanceRequest::ReloadStructure.into()]
        } else {
            match request {
                staticons::StaticonUpdateRequest::Nothing => vec![],
                staticons::StaticonUpdateRequest::UpdateDynData => {
                    vec![InstanceRequest::ReloadStaticonDynData.into()]
                }
                staticons::StaticonUpdateRequest::UpdateCode => {
                    vec![InstanceRequest::ReloadStructure.into()]
                }
            }
        }
    }

    fn on_click(mut self: Box<Self>) -> Vec<GuiRequest> {
        self.float_value += self.click_delta as f32;
        self.float_value = self.float_value.min(self.max as f32);
        self.float_value = self.float_value.max(self.min as f32);
        let request = (self.callback)(self.float_value as i32);
        if self.code_reload_requested {
            vec![InstanceRequest::ReloadStructure.into()]
        } else {
            match request {
                staticons::StaticonUpdateRequest::Nothing => vec![],
                staticons::StaticonUpdateRequest::UpdateDynData => {
                    vec![InstanceRequest::ReloadStaticonDynData.into()]
                }
                staticons::StaticonUpdateRequest::UpdateCode => {
                    vec![InstanceRequest::ReloadStructure.into()]
                }
            }
        }
    }
}

#[make_constructor]
pub struct MutateStaticon {
    mutator: Box<dyn FnOnce() -> staticons::StaticonUpdateRequest>,
}

impl MutateStaticon {
    pub fn wrap<M>(mutator: M) -> Option<Box<dyn MouseAction>>
    where
        M: FnOnce() -> staticons::StaticonUpdateRequest + 'static,
    {
        Some(Box::new(Self::new(Box::new(mutator))))
    }
}

impl MouseAction for MutateStaticon {
    fn on_click(self: Box<Self>) -> Vec<GuiRequest> {
        let update = (self.mutator)();
        let mut ret = Vec::new();
        match update {
            staticons::StaticonUpdateRequest::Nothing => (),
            staticons::StaticonUpdateRequest::UpdateDynData => {
                ret.push(InstanceRequest::ReloadStaticonDynData.into());
            }
            staticons::StaticonUpdateRequest::UpdateCode => {
                ret.push(InstanceRequest::ReloadStructure.into());
            }
        }
        ret
    }
}

#[make_constructor]
pub struct ContinuouslyMutateStaticon {
    mutator:
        Box<dyn FnMut(f32, Option<f32>) -> (staticons::StaticonUpdateRequest, Option<Tooltip>)>,
    #[value(false)]
    code_reload_requested: bool,
}

impl ContinuouslyMutateStaticon {
    pub fn wrap<M>(mutator: M) -> Option<Box<dyn MouseAction>>
    where
        M: 'static + FnMut(f32, Option<f32>) -> (staticons::StaticonUpdateRequest, Option<Tooltip>),
    {
        Some(Box::new(Self::new(Box::new(mutator))))
    }
}

impl MouseAction for ContinuouslyMutateStaticon {
    fn on_drag(&mut self, delta: (f32, f32), mods: &MouseMods) -> Vec<GuiRequest> {
        let mut delta = delta.0 - delta.1;
        if mods.precise {
            delta *= PRECISION_MULTIPLIER;
        }
        let (update, tooltip) = (self.mutator)(delta, get_snap_steps(mods));
        let mut ret = Vec::new();
        if let Some(tooltip) = tooltip {
            ret.push(tooltip.into());
        }
        match update {
            staticons::StaticonUpdateRequest::Nothing => (),
            staticons::StaticonUpdateRequest::UpdateDynData => {
                ret.push(InstanceRequest::ReloadStaticonDynData.into());
            }
            staticons::StaticonUpdateRequest::UpdateCode => {
                self.code_reload_requested = true;
            }
        }
        ret
    }
}

pub struct DragModule {
    pos: (f32, f32),
    module: Rcrc<ep::Module>,
}

impl DragModule {
    pub fn new(module: Rcrc<ep::Module>) -> Self {
        let pos = module.borrow().pos;
        Self { pos, module }
    }
}

impl MouseAction for DragModule {
    fn on_drag(&mut self, delta: (f32, f32), mods: &MouseMods) -> Vec<GuiRequest> {
        self.pos = self.pos.add(delta);
        let mut module_ref = self.module.borrow_mut();
        if mods.snap {
            const SPACING: f32 = (grid(1) + GRID_P) as f32;
            module_ref.pos = self.pos.sub((self.pos.0 % SPACING, self.pos.1 % SPACING));
        } else {
            module_ref.pos = self.pos;
        }
        vec![Tooltip {
            text: "".to_owned(),
            interaction: InteractionHint::LeftClickAndDrag | InteractionHint::SnappingModifier,
        }
        .into()]
    }
}

#[make_constructor]
pub struct PanOffset {
    offset: Rcrc<(f32, f32)>,
}

impl MouseAction for PanOffset {
    fn on_drag(&mut self, delta: (f32, f32), _mods: &MouseMods) -> Vec<GuiRequest> {
        let mut offset_ref = self.offset.borrow_mut();
        *offset_ref = offset_ref.add(delta);
        vec![]
    }
}

#[make_constructor]
pub struct ConnectInput {
    module: Rcrc<ep::Module>,
    index: usize,
}

impl MouseAction for ConnectInput {
    fn on_drop(self: Box<Self>, target: DropTarget) -> Vec<GuiRequest> {
        let mut in_ref = self.module.borrow_mut();
        let template_ref = in_ref.template.borrow();
        let in_type = template_ref.inputs[self.index].get_type();
        drop(template_ref);
        if let DropTarget::Output(out_module, out_index) = target {
            let out_type = out_module.borrow().template.borrow().outputs[out_index].get_type();
            if in_type == out_type {
                in_ref.inputs[self.index] = ep::InputConnection::Wire(out_module, out_index);
            }
        // Only change to a default if it used to be connected.
        } else if let ep::InputConnection::Wire(..) = &in_ref.inputs[self.index] {
            let default = in_ref.template.borrow().default_inputs[self.index];
            in_ref.inputs[self.index] = ep::InputConnection::Default(default);
        }
        vec![InstanceRequest::ReloadStructure.into()]
    }
}

#[make_constructor]
pub struct ConnectOutput {
    module: Rcrc<ep::Module>,
    index: usize,
}

impl MouseAction for ConnectOutput {
    fn on_drop(self: Box<Self>, target: DropTarget) -> Vec<GuiRequest> {
        let out_type = self.module.borrow().template.borrow().outputs[self.index].get_type();
        if let DropTarget::Input(in_module, in_index) = target {
            let mut in_ref = in_module.borrow_mut();
            let in_type = in_ref.template.borrow().inputs[in_index].get_type();
            if in_type == out_type {
                in_ref.inputs[in_index] = ep::InputConnection::Wire(self.module, self.index);
            }
        } else if let DropTarget::Autocon(control) = target {
            if out_type == ep::JackType::Audio {
                let mut control_ref = control.borrow_mut();
                let range = control_ref.range;
                control_ref.automation.push(ep::AutomationLane {
                    connection: (self.module, self.index),
                    range,
                });
            }
        }
        vec![InstanceRequest::ReloadStructure.into()]
    }
}

#[make_constructor]
#[make_constructor(pub single(item: GuiRequest))]
pub struct SubmitRequestsOnClick {
    #[value(vec![item] for single)]
    requests: Vec<GuiRequest>,
}

impl SubmitRequestsOnClick {
    pub fn wrap(requests: Vec<GuiRequest>) -> Option<Box<dyn MouseAction>> {
        Some(Box::new(Self::new(requests)))
    }
}

impl MouseAction for SubmitRequestsOnClick {
    fn on_click(self: Box<Self>) -> Vec<GuiRequest> {
        self.requests
    }
}

impl From<GuiRequest> for SubmitRequestsOnClick {
    fn from(other: GuiRequest) -> Self {
        Self::new(vec![other])
    }
}

impl From<Vec<GuiRequest>> for SubmitRequestsOnClick {
    fn from(other: Vec<GuiRequest>) -> Self {
        Self::new(other)
    }
}

impl Into<Option<Box<dyn MouseAction>>> for GuiRequest {
    fn into(self: Self) -> Option<Box<dyn MouseAction>> {
        Some(Box::new(SubmitRequestsOnClick::from(self)))
    }
}

impl Into<Option<Box<dyn MouseAction>>> for InstanceRequest {
    fn into(self: Self) -> Option<Box<dyn MouseAction>> {
        GuiRequest::from(self).into()
    }
}

fn get_snap_steps(mods: &MouseMods) -> Option<f32> {
    if mods.snap {
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

fn range_drag_delta(delta: (f32, f32), mods: &MouseMods) -> f32 {
    let res = (delta.0 - delta.1) / RANGE_DRAG_PIXELS;
    if mods.precise {
        res * PRECISION_MULTIPLIER
    } else {
        res
    }
}

fn range_value_tooltip(value: f32, suffix: &str) -> GuiRequest {
    Tooltip {
        text: format!("{}{}", format_decimal(value, 4), suffix,),
        interaction: InteractionHint::LeftClickAndDrag
            | InteractionHint::PrecisionModifier
            | InteractionHint::SnappingModifier,
    }
    .into()
}

#[derive(Clone)]
pub enum DropTarget {
    None,
    Autocon(Rcrc<ep::Autocon>),
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
