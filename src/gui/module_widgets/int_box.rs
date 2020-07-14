use super::ModuleWidget;
use crate::engine::parts as ep;
use crate::gui::action::MouseAction;
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::{InteractionHint, MouseMods, Tooltip};
use crate::registry::Registry;
use crate::util::*;

#[derive(Clone)]
pub struct IntBoxBase {
    tooltip: String,
    pos: (f32, f32),
    range: (i32, i32),
    label: String,
    icons: (usize, usize),
}

impl IntBoxBase {
    pub const WIDTH: f32 = grid(2);
    pub const HEIGHT: f32 = grid(2) - FONT_SIZE - GRID_P / 2.0;
    pub fn create(
        tooltip: String,
        registry: &Registry,
        pos: (f32, f32),
        range: (i32, i32),
        label: String,
    ) -> IntBoxBase {
        IntBoxBase {
            tooltip,
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

pub trait IntBoxImpl {
    fn get_base(&self) -> &IntBoxBase;
    fn get_current_value(&self) -> i32;
    // The callback will be called whenever the user changes the value and that change should be
    // shown on screen. A recompile will be requested once the user stops changing the value
    // regardless of what happens in this callback.
    fn make_callback(&self) -> Box<dyn Fn(i32)>;
}

impl<T: IntBoxImpl> ModuleWidget for T {
    fn get_position(&self) -> (f32, f32) {
        self.get_base().pos
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
        let click_delta = if local_pos.1 > IntBoxBase::HEIGHT / 2.0 {
            -1
        } else {
            1
        };
        MouseAction::ManipulateIntBox {
            callback: self.make_callback(),
            min: self.get_base().range.0,
            max: self.get_base().range.1,
            click_delta,
            float_value: self.get_current_value() as f32,
        }
    }

    fn get_tooltip_at(&self, local_pos: (f32, f32)) -> Option<Tooltip> {
        Some(Tooltip {
            text: self.get_base().tooltip.clone(),
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
        let base = self.get_base();
        g.push_state();
        g.apply_offset(base.pos.0, base.pos.1);

        const W: f32 = IntBoxBase::WIDTH;
        const H: f32 = IntBoxBase::HEIGHT;
        const CS: f32 = CORNER_SIZE;
        g.set_color(&COLOR_BG);
        g.fill_rounded_rect(0.0, 0.0, W, H, CS);
        const IS: f32 = H / 2.0;
        g.draw_white_icon(base.icons.0, W - IS, 0.0, IS);
        g.draw_white_icon(base.icons.1, W - IS, IS, IS);
        {
            let val = format!("{}", self.get_current_value());
            const HA: HAlign = HAlign::Right;
            const VA: VAlign = VAlign::Center;
            g.set_color(&COLOR_TEXT);
            g.write_text(BIG_FONT_SIZE, 0.0, 0.0, W - IS - 4.0, H, HA, VA, 1, &val);
        }
        {
            let val = &base.label;
            const HA: HAlign = HAlign::Center;
            const VA: VAlign = VAlign::Bottom;
            g.set_color(&COLOR_TEXT);
            g.write_text(FONT_SIZE, 0.0, 0.0, W, grid(2), HA, VA, 1, val);
        }

        g.pop_state();
    }
}

#[derive(Clone)]
pub struct IntBox {
    base: IntBoxBase,
    ccontrol: Rcrc<ep::ComplexControl>,
}

impl IntBox {
    pub fn create(
        tooltip: String,
        registry: &Registry,
        ccontrol: Rcrc<ep::ComplexControl>,
        pos: (f32, f32),
        range: (i32, i32),
        label: String,
    ) -> IntBox {
        IntBox {
            base: IntBoxBase::create(tooltip, registry, pos, range, label),
            ccontrol,
        }
    }
}

impl IntBoxImpl for IntBox {
    fn get_base(&self) -> &IntBoxBase {
        &self.base
    }

    fn get_current_value(&self) -> i32 {
        self.ccontrol.borrow().value.parse().unwrap()
    }

    fn make_callback(&self) -> Box<dyn Fn(i32)> {
        let ccontrol = Rc::clone(&self.ccontrol);
        Box::new(move |new_value| {
            ccontrol.borrow_mut().value = format!("{}", new_value)
        })
    }
}