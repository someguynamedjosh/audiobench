use super::ModuleWidget;
use crate::engine::static_controls as staticons;
use crate::gui::action::MouseAction;
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::{InteractionHint, MouseMods, Tooltip};
use crate::util::*;

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: OptionBox,
    constructor: create(
        pos: GridPos,
        size: GridSize,
        control: ControlledOptionChoiceRef,
        label: String,
        tooltip: String,
    ),
}

#[derive(Clone)]
pub struct OptionBox {
    pos: (f32, f32),
    size: (f32, f32),
    control: Rcrc<staticons::ControlledOptionChoice>,
    label: String,
    tooltip: String,
}

impl OptionBox {
    pub fn create(
        pos: (f32, f32),
        size: (f32, f32),
        control: Rcrc<staticons::ControlledOptionChoice>,
        label: String,
        tooltip: String,
    ) -> OptionBox {
        OptionBox {
            tooltip,
            control,
            pos,
            size,
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
        _mods: &MouseMods,
        _parent_pos: (f32, f32),
    ) -> MouseAction {
        let num_options = self.control.borrow().get_options().len();
        let height_per_option = (self.size.1 - FONT_SIZE - GRID_P / 2.0) / num_options as f32;
        let option = (local_pos.1 / height_per_option) as usize;
        if option < num_options {
            let cref = Rc::clone(&self.control);
            MouseAction::MutateStaticon(Box::new(move || {
                cref.borrow_mut().set_selected_option(option)
            }))
        } else {
            MouseAction::None
        }
    }

    fn get_tooltip_at(&self, _local_pos: (f32, f32)) -> Option<Tooltip> {
        Some(Tooltip {
            text: self.tooltip.clone(),
            interaction: InteractionHint::LeftClick | InteractionHint::DoubleClick,
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
        g.set_color(&COLOR_BG);
        let num_options = self.control.borrow().get_options().len();
        // Don't ask why GP / 2 and not just GP, it just looks better and I don't know why.
        let height_per_option = (self.size.1 - FONT_SIZE - GRID_P / 2.0) / num_options as f32;
        let h = height_per_option * num_options as f32;
        g.fill_rounded_rect(0.0, 0.0, self.size.0, h, CS);
        let current_option = self.control.borrow().get_selected_option();
        for (index, option) in self.control.borrow().get_options().iter().enumerate() {
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
