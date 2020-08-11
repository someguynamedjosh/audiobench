use super::ModuleWidget;
use crate::engine::parts as ep;
use crate::engine::static_controls as staticons;
use crate::gui::action::MouseAction;
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::{InteractionHint, MouseMods, Tooltip};
use crate::registry::yaml::YamlNode;
use crate::registry::Registry;
use crate::util::*;

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: TimingSelector,
    constructor: create(
        registry: RegistryRef,
        pos: GridPos,
        control: ControlledTimingModeRef,
    ),
}

#[derive(Clone)]
pub struct TimingSelector {
    control: Rcrc<staticons::ControlledTimingMode>,
    pos: (f32, f32),
    note_icon: usize,
    song_icon: usize,
    time_icon: usize,
    beats_icon: usize,
}

impl TimingSelector {
    pub fn create(
        registry: &Registry,
        pos: (f32, f32),
        control: Rcrc<staticons::ControlledTimingMode>,
    ) -> Self {
        Self {
            control,
            pos,
            note_icon: registry.lookup_icon("factory:note").unwrap(),
            song_icon: registry.lookup_icon("factory:treble_clef").unwrap(),
            time_icon: registry.lookup_icon("factory:time").unwrap(),
            beats_icon: registry.lookup_icon("factory:metronome").unwrap(),
        }
    }

    fn source_value(&self) -> bool {
        self.control.borrow().uses_song_time()
    }

    fn type_value(&self) -> bool {
        self.control.borrow().uses_song_time()
    }
}

impl ModuleWidget for TimingSelector {
    fn get_position(&self) -> (f32, f32) {
        self.pos
    }

    fn get_bounds(&self) -> (f32, f32) {
        (grid(2), grid(2))
    }

    fn respond_to_mouse_press(
        &self,
        local_pos: (f32, f32),
        _mods: &MouseMods,
        _parent_pos: (f32, f32),
    ) -> MouseAction {
        let cref = Rc::clone(&self.control);
        if local_pos.0 < grid(2) / 2.0 {
            MouseAction::MutateStaticon(Box::new(move || cref.borrow_mut().toggle_source()))
        } else {
            MouseAction::MutateStaticon(Box::new(move || cref.borrow_mut().toggle_units()))
        }
    }

    fn get_tooltip_at(&self, local_pos: (f32, f32)) -> Option<Tooltip> {
        Some(Tooltip {
            text: if local_pos.0 < grid(2) / 2.0 {
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
        })
    }

    fn draw(
        &self,
        g: &mut GrahpicsWrapper,
        _highlight: bool,
        _parent_pos: (f32, f32),
        _feedback_data: &[f32],
    ) {
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1);

        const CS: f32 = CORNER_SIZE;
        const ICON_SIZE: f32 = (grid(2) - CS * 3.0) / 2.0;
        g.set_color(&COLOR_BG);
        g.fill_rounded_rect(0.0, 0.0, grid(2), CS * 2.0 + ICON_SIZE, CS);
        g.draw_white_icon(
            if self.source_value() {
                self.song_icon
            } else {
                self.note_icon
            },
            CS,
            CS,
            ICON_SIZE,
        );
        g.draw_white_icon(
            if self.type_value() {
                self.beats_icon
            } else {
                self.time_icon
            },
            CS + ICON_SIZE + CS,
            CS,
            ICON_SIZE,
        );
        g.set_color(&COLOR_TEXT);
        g.write_text(
            FONT_SIZE,
            0.0,
            0.0,
            grid(2),
            grid(2),
            HAlign::Center,
            VAlign::Bottom,
            1,
            "Timing",
        );

        g.pop_state();
    }
}
