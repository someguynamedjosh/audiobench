use super::ModuleWidgetImpl;
use crate::engine::controls::{TimingModeControl, };
use crate::gui::constants::*;
use crate::gui::mouse_behaviors::MutateControl;
use crate::gui::{InteractionHint, Tooltip};
use crate::scui_config::{DropTarget, MaybeMouseBehavior, Renderer};
use scui::{MouseMods, Vec2D, WidgetImpl};
use shared_util::prelude::*;

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: TimingSelector,
    constructor: new(
        parent: ParentRef,
        pos: GridPos,
        control: TimingModeControlRef,
    ),
}

scui::widget! {
    pub TimingSelector
    State {
        pos: Vec2D,
        control: Rcrc<TimingModeControl>,
        note_icon: usize,
        song_icon: usize,
        time_icon: usize,
        beats_icon: usize,
    }
}

impl TimingSelector {
    pub fn new(
        parent: &impl TimingSelectorParent,
        pos: Vec2D,
        control: Rcrc<TimingModeControl>,
    ) -> Rc<Self> {
        let int = parent.provide_gui_interface();
        let gui_state = int.state.borrow();
        let registry = gui_state.registry.borrow();
        let state = TimingSelectorState {
            pos,
            control,
            note_icon: registry.lookup_icon("Factory:note").unwrap(),
            song_icon: registry.lookup_icon("Factory:treble_clef").unwrap(),
            time_icon: registry.lookup_icon("Factory:time").unwrap(),
            beats_icon: registry.lookup_icon("Factory:metronome").unwrap(),
        };
        Rc::new(Self::create(parent, state))
    }

    fn source_value(&self) -> bool {
        self.state.borrow().control.borrow().uses_elapsed_time()
    }

    fn type_value(&self) -> bool {
        self.state.borrow().control.borrow().is_beat_synchronized()
    }
}

impl WidgetImpl<Renderer, DropTarget> for TimingSelector {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().pos.into()
    }

    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        (grid(2), grid(2)).into()
    }

    fn get_mouse_behavior_impl(
        self: &Rc<Self>,
        pos: Vec2D,
        _mods: &MouseMods,
    ) -> MaybeMouseBehavior {
        let cref = Rc::clone(&self.state.borrow().control);
        if pos.x < grid(2) / 2.0 {
            MutateControl::wrap(self, move || cref.borrow_mut().toggle_source())
        } else {
            MutateControl::wrap(self, move || cref.borrow_mut().toggle_units())
        }
    }

    fn on_hover_impl(self: &Rc<Self>, pos: Vec2D) -> Option<()> {
        let tooltip = Tooltip {
            text: if pos.x < grid(2) / 2.0 {
                format!(
                    "Change timing source, current value is \"{}\"",
                    if self.source_value() { "song" } else { "note" }
                )
            } else {
                format!(
                    "Change timing type, current value is \"{}\"",
                    if self.type_value() {
                        "beats"
                    } else {
                        "seconds"
                    }
                )
            },
            interaction: InteractionHint::LeftClick.into(),
        };
        self.with_gui_state_mut(|state| {
            state.set_tooltip(tooltip);
        });
        Some(())
    }

    fn draw_impl(self: &Rc<Self>, g: &mut Renderer) {
        let state = self.state.borrow();

        const CS: f32 = CORNER_SIZE;
        const ICON_SIZE: f32 = (grid(2) - CS * 3.0) / 2.0;
        g.set_color(&COLOR_BG0);
        g.draw_rounded_rect(0.0, (grid(2), CS * 2.0 + ICON_SIZE), CS);
        g.draw_white_icon(
            if self.source_value() {
                state.song_icon
            } else {
                state.note_icon
            },
            CS,
            ICON_SIZE,
        );
        g.draw_white_icon(
            if self.type_value() {
                state.beats_icon
            } else {
                state.time_icon
            },
            (CS + ICON_SIZE + CS, CS),
            ICON_SIZE,
        );
        g.set_color(&COLOR_FG1);
        g.draw_text(FONT_SIZE, 0.0, grid(2), (0, 1), 1, "Timing");
    }
}

impl ModuleWidgetImpl for TimingSelector {}
