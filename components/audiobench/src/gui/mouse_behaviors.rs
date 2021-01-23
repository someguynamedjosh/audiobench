use crate::{
    engine::{
        controls::{Control, FloatInRangeControl, UpdateRequest},
        parts as ep, UiThreadEngine,
    },
    gui::{constants::*, InteractionHint, Tooltip},
    scui_config::{GuiState, MaybeMouseBehavior},
};
use scones::make_constructor;
use scui::{GuiInterface, GuiInterfaceProvider, MouseBehavior, MouseMods, Vec2D};
use shared_util::prelude::*;

#[derive(Clone, Debug)]
pub enum DropTarget {
    None,
    Control(Rcrc<dyn Control>),
    Output(Rcrc<ep::Module>, usize),
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

fn range_drag_delta(delta: Vec2D, mods: &MouseMods) -> f32 {
    let res = (delta.x - delta.y) / RANGE_DRAG_PIXELS;
    if mods.precise {
        res * PRECISION_MULTIPLIER
    } else {
        res
    }
}

fn discrete_drag_delta(delta: Vec2D, mods: &MouseMods) -> f32 {
    let res = (delta.x - delta.y) / DISCRETE_STEP_PIXELS;
    if mods.precise {
        res * PRECISION_MULTIPLIER
    } else {
        res
    }
}

fn range_value_tooltip(value: f32, suffix: &str) -> Tooltip {
    Tooltip {
        text: format!("{}{}", format_decimal(value, 4), suffix,),
        interaction: vec![
            InteractionHint::LeftClickAndDrag,
            InteractionHint::PrecisionModifier,
            InteractionHint::SnappingModifier,
        ],
    }
}

#[make_constructor((widget: &impl GuiInterfaceProvider<GuiState, DropTarget>, control: Rcrc<FloatInRangeControl>))]
pub struct ManipulateControl {
    #[value(Rc::clone(&widget.provide_gui_interface().state.borrow().engine))]
    engine: Rcrc<UiThreadEngine>,
    #[value(Rc::clone(&widget.provide_gui_interface()))]
    gui_interface: Rc<GuiInterface<GuiState, DropTarget>>,
    #[value(Rc::clone(&control))]
    control: Rcrc<FloatInRangeControl>,
    #[value(control.borrow().value)]
    current_value: f32,
}

impl MouseBehavior<DropTarget> for ManipulateControl {
    fn on_drag(&mut self, delta: Vec2D, mods: &MouseMods) {
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
        let tooltip = range_value_tooltip(control_ref.value, &control_ref.suffix);
        drop(control_ref);
        self.engine.borrow_mut().reload_dyn_data();
        self.gui_interface.state.borrow_mut().set_tooltip(tooltip);
    }

    fn on_double_click(self: Box<Self>) {
        let mut cref = self.control.borrow_mut();
        cref.value = cref.default;
        drop(cref);
        self.engine.borrow_mut().reload_dyn_data();
    }
}

#[make_constructor((widget: &impl GuiInterfaceProvider<GuiState, DropTarget>, ..))]
#[make_constructor(pub start_only(widget: &impl GuiInterfaceProvider<GuiState, DropTarget>, ..))]
#[make_constructor(pub end_only(widget: &impl GuiInterfaceProvider<GuiState, DropTarget>, ..))]
pub struct ManipulateLane {
    #[value(Rc::clone(&widget.provide_gui_interface().state.borrow().engine))]
    engine: Rcrc<UiThreadEngine>,
    #[value(Rc::clone(&widget.provide_gui_interface()))]
    gui_interface: Rc<GuiInterface<GuiState, DropTarget>>,
    control: Rcrc<FloatInRangeControl>,
    lane: usize,
    #[value(true)]
    #[value(false for end_only)]
    start: bool,
    #[value(true)]
    #[value(false for start_only)]
    end: bool,
}

impl MouseBehavior<DropTarget> for ManipulateLane {
    fn on_drag(&mut self, delta: Vec2D, mods: &MouseMods) {
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
        drop(control_ref);
        let tooltip = Tooltip {
            text: tttext,
            interaction: vec![
                InteractionHint::LeftClickAndDrag,
                InteractionHint::PrecisionModifier,
                InteractionHint::SnappingModifier,
            ],
        };
        self.gui_interface.state.borrow_mut().set_tooltip(tooltip);
        self.engine.borrow_mut().reload_dyn_data();
    }

    fn on_double_click(self: Box<Self>) {
        let mut cref = self.control.borrow_mut();
        let range = cref.range;
        let lane = &mut cref.automation[self.lane];
        if self.start {
            lane.range.0 = range.0
        };
        if self.end {
            lane.range.1 = range.1
        };
        drop(cref);
        self.engine.borrow_mut().reload_dyn_data();
    }
}

#[make_constructor((widget: &impl GuiInterfaceProvider<GuiState, DropTarget>, .., current_value: i32))]
pub struct ManipulateIntBox {
    #[value(Rc::clone(&widget.provide_gui_interface().state.borrow().engine))]
    engine: Rcrc<UiThreadEngine>,
    callback: Box<dyn FnMut(i32) -> UpdateRequest>,
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

impl MouseBehavior<DropTarget> for ManipulateIntBox {
    fn on_drag(&mut self, delta: Vec2D, mods: &MouseMods) {
        let delta = discrete_drag_delta(delta, mods);
        self.float_value += delta;
        self.float_value = self.float_value.clam(self.min as f32, self.max as f32);
        match (self.callback)(self.float_value as i32) {
            UpdateRequest::Nothing => (),
            UpdateRequest::UpdateDynData => {
                self.engine.borrow_mut().reload_dyn_data();
            }
            UpdateRequest::UpdateCode => {
                self.code_reload_requested = true;
            }
        }
    }

    fn on_drop(mut self: Box<Self>, _target: Option<DropTarget>) {
        let request = (self.callback)(self.float_value as i32);
        if self.code_reload_requested {
            self.engine.borrow_mut().regenerate_code()
        } else {
            match request {
                UpdateRequest::Nothing => (),
                UpdateRequest::UpdateDynData => {
                    self.engine.borrow_mut().reload_dyn_data();
                }
                UpdateRequest::UpdateCode => self.engine.borrow_mut().regenerate_code(),
            }
        }
    }

    fn on_click(mut self: Box<Self>) {
        self.float_value += self.click_delta as f32;
        self.float_value = self.float_value.min(self.max as f32);
        self.float_value = self.float_value.max(self.min as f32);
        let request = (self.callback)(self.float_value as i32);
        if self.code_reload_requested {
            self.engine.borrow_mut().regenerate_code()
        } else {
            match request {
                UpdateRequest::Nothing => (),
                UpdateRequest::UpdateDynData => {
                    self.engine.borrow_mut().reload_dyn_data();
                }
                UpdateRequest::UpdateCode => self.engine.borrow_mut().regenerate_code(),
            }
        }
    }
}

#[make_constructor(new)]
pub struct MutateControl {
    engine: Rcrc<UiThreadEngine>,
    mutator: Box<dyn FnOnce() -> UpdateRequest>,
}

impl MutateControl {
    pub fn wrap<W, M>(widget: &W, mutator: M) -> MaybeMouseBehavior
    where
        W: GuiInterfaceProvider<GuiState, DropTarget>,
        M: FnOnce() -> UpdateRequest + 'static,
    {
        let int = widget.provide_gui_interface();
        let engine = Rc::clone(&int.state.borrow().engine);
        Some(Box::new(Self::new(engine, Box::new(mutator))))
    }
}

impl MouseBehavior<DropTarget> for MutateControl {
    fn on_click(self: Box<Self>) {
        let update = (self.mutator)();
        match update {
            UpdateRequest::Nothing => (),
            UpdateRequest::UpdateDynData => {
                self.engine.borrow_mut().reload_dyn_data();
            }
            UpdateRequest::UpdateCode => self.engine.borrow_mut().regenerate_code(),
        }
    }
}

#[make_constructor(new)]
pub struct ContinuouslyMutateControl {
    engine: Rcrc<UiThreadEngine>,
    gui_interface: Rc<GuiInterface<GuiState, DropTarget>>,
    mutator: Box<dyn FnMut(f32, Option<f32>) -> (UpdateRequest, Option<Tooltip>)>,
    #[value(false)]
    code_reload_requested: bool,
}

impl ContinuouslyMutateControl {
    pub fn wrap<W, M>(widget: &W, mutator: M) -> MaybeMouseBehavior
    where
        W: GuiInterfaceProvider<GuiState, DropTarget>,
        M: FnMut(f32, Option<f32>) -> (UpdateRequest, Option<Tooltip>) + 'static,
    {
        let int = widget.provide_gui_interface();
        let engine = Rc::clone(&int.state.borrow().engine);
        Some(Box::new(Self::new(engine, int, Box::new(mutator))))
    }
}

impl MouseBehavior<DropTarget> for ContinuouslyMutateControl {
    fn on_drag(&mut self, delta: Vec2D, mods: &MouseMods) {
        let mut delta = delta.x - delta.y;
        if mods.precise {
            delta *= PRECISION_MULTIPLIER;
        }
        let (update, tooltip) = (self.mutator)(delta, get_snap_steps(mods));
        if let Some(tooltip) = tooltip {
            self.gui_interface.state.borrow_mut().set_tooltip(tooltip);
        }
        match update {
            UpdateRequest::Nothing => (),
            UpdateRequest::UpdateDynData => {
                self.engine.borrow_mut().reload_dyn_data();
            }
            UpdateRequest::UpdateCode => {
                self.code_reload_requested = true;
            }
        }
    }

    fn on_drop(self: Box<Self>, _target: Option<DropTarget>) {
        if self.code_reload_requested {
            self.engine.borrow_mut().regenerate_code()
        }
    }
}
