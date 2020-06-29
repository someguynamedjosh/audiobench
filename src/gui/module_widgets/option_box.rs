use super::ModuleWidget;
use crate::engine::parts as ep;
use crate::gui::action::{MouseAction};
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::{InteractionHint, MouseMods, Tooltip};
use crate::util::*;

#[derive(Clone)]
pub struct OptionBox {
    tooltip: String,
    ccontrol: Rcrc<ep::ComplexControl>,
    pos: (f32, f32),
    size: (f32, f32),
    options: Vec<String>,
    label: String,
}

impl OptionBox {
    pub(super) fn create(
        tooltip: String,
        ccontrol: Rcrc<ep::ComplexControl>,
        pos: (f32, f32),
        size: (f32, f32),
        options: Vec<String>,
        label: String,
    ) -> OptionBox {
        OptionBox {
            tooltip,
            ccontrol,
            pos,
            size,
            options,
            label,
        }
    }
}

impl ModuleWidget for OptionBox {
    fn get_position(&self) -> (f32, f32) {
        self.pos
    }

    fn get_bounds(&self) -> (f32, f32) {
        self.size
    }

    fn respond_to_mouse_press(
        &self,
        local_pos: (f32, f32),
        mods: &MouseMods,
        parent_pos: (f32, f32),
    ) -> MouseAction {
        let height_per_option =
            (self.size.1 - FONT_SIZE - GRID_P / 2.0) / self.options.len() as f32;
        let option = (local_pos.1 / height_per_option) as usize;
        if option < self.options.len() {
            MouseAction::SetComplexControl(Rc::clone(&self.ccontrol), format!("{}", option))
        } else {
            // Still return a set control thing so that if we double-click, we still know to reset
            // the control and not just do nothing.
            let value = self.ccontrol.borrow().value.clone();
            MouseAction::SetComplexControl(Rc::clone(&self.ccontrol), format!("{}", value))
        }
    }

    fn get_tooltip_at(&self, local_pos: (f32, f32)) -> Option<Tooltip> {
        Some(Tooltip {
            text: self.tooltip.clone(),
            interaction: InteractionHint::LeftClick | InteractionHint::DoubleClick,
        })
    }

    fn draw(
        &self,
        g: &mut GrahpicsWrapper,
        highlight: bool,
        parent_pos: (f32, f32),
        feedback_data: &[f32],
    ) {
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1);

        const CS: f32 = CORNER_SIZE;
        g.set_color(&COLOR_BG);
        // Don't ask why GP / 2 and not just GP, it just looks better and I don't know why.
        let height_per_option =
            (self.size.1 - FONT_SIZE - GRID_P / 2.0) / self.options.len() as f32;
        let h = height_per_option * self.options.len() as f32;
        g.fill_rounded_rect(0.0, 0.0, self.size.0, h, CS);
        let current_option: usize = self.ccontrol.borrow().value.parse().unwrap_or(0);
        for (index, option) in self.options.iter().enumerate() {
            let y = index as f32 * height_per_option;
            if index == current_option {
                g.set_color(&COLOR_IO_AREA);
                g.fill_rounded_rect(0.0, y, self.size.0, height_per_option, CORNER_SIZE);
            }
            g.set_color(&COLOR_TEXT);
            g.write_text(
                FONT_SIZE,
                0.0,
                y,
                self.size.0,
                height_per_option,
                HAlign::Center,
                VAlign::Center,
                1,
                option,
            );
        }
        g.write_text(
            FONT_SIZE,
            0.0,
            0.0,
            self.size.0,
            self.size.1,
            HAlign::Center,
            VAlign::Bottom,
            1,
            &self.label,
        );

        g.pop_state();
    }
}
