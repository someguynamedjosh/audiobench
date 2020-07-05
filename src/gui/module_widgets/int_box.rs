use super::ModuleWidget;
use crate::engine::parts as ep;
use crate::registry::Registry;
use crate::gui::action::MouseAction;
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::{InteractionHint, MouseMods, Tooltip};
use crate::util::*;

#[derive(Clone)]
pub struct IntBox {
    tooltip: String,
    ccontrol: Rcrc<ep::ComplexControl>,
    pos: (f32, f32),
    range: (i32, i32),
    label: String,
    icons: (usize, usize),
}

impl IntBox {
    const WIDTH: f32 = grid(2);
    const HEIGHT: f32 = grid(2) - FONT_SIZE - GRID_P / 2.0;
    pub fn create(
        tooltip: String,
        registry: &Registry,
        ccontrol: Rcrc<ep::ComplexControl>,
        pos: (f32, f32),
        range: (i32, i32),
        label: String,
    ) -> IntBox {
        IntBox {
            tooltip,
            ccontrol,
            pos,
            range,
            label,
            // Factory library is guaranteed to have these icons.
            icons: (
                registry.lookup_icon("factory:increase").unwrap(),
                registry.lookup_icon("factory:decrease").unwrap(),
            ),
        }
    }
}

impl ModuleWidget for IntBox {
    fn get_position(&self) -> (f32, f32) {
        self.pos
    }
    fn get_bounds(&self) -> (f32, f32) {
        (grid(2), grid(2))
    }
    fn respond_to_mouse_press(
        &self,
        local_pos: (f32, f32),
        mods: &MouseMods,
        parent_pos: (f32, f32),
    ) -> MouseAction {
        let click_delta = if local_pos.1 > IntBox::HEIGHT / 2.0 {
            -1
        } else {
            1
        };
        MouseAction::ManipulateIntControl {
            cref: Rc::clone(&self.ccontrol),
            min: self.range.0,
            max: self.range.1,
            click_delta,
            float_value: self.ccontrol.borrow().value.parse().unwrap(),
        }
    }

    fn get_tooltip_at(&self, local_pos: (f32, f32)) -> Option<Tooltip> {
        Some(Tooltip {
            text: self.tooltip.clone(),
            interaction: InteractionHint::LeftClick
                | InteractionHint::LeftClickAndDrag
                | InteractionHint::DoubleClick,
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

        const W: f32 = IntBox::WIDTH;
        const H: f32 = IntBox::HEIGHT;
        const CS: f32 = CORNER_SIZE;
        g.set_color(&COLOR_BG);
        g.fill_rounded_rect(0.0, 0.0, W, H, CS);
        const IS: f32 = H / 2.0;
        g.draw_white_icon(self.icons.0, W - IS, 0.0, IS);
        g.draw_white_icon(self.icons.1, W - IS, IS, IS);
        {
            let val = &self.ccontrol.borrow().value;
            const HA: HAlign = HAlign::Right;
            const VA: VAlign = VAlign::Center;
            g.set_color(&COLOR_TEXT);
            g.write_text(BIG_FONT_SIZE, 0.0, 0.0, W - IS - 4.0, H, HA, VA, 1, val);
        }
        {
            let val = &self.label;
            const HA: HAlign = HAlign::Center;
            const VA: VAlign = VAlign::Bottom;
            g.set_color(&COLOR_TEXT);
            g.write_text(FONT_SIZE, 0.0, 0.0, W, grid(2), HA, VA, 1, val);
        }

        g.pop_state();
    }
}
