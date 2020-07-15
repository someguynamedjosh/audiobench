use super::ModuleWidget;
use crate::engine::parts as ep;
use crate::gui::action::MouseAction;
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::{InteractionHint, MouseMods, Tooltip};
use crate::registry::yaml::YamlNode;
use crate::registry::Registry;
use crate::util::*;

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: HertzBox,
    constructor: create(
        registry: RegistryRef,
        pos: GridPos,
        control: ComplexControlRef,
        range: FloatRange,
        label: String,
        tooltip: String,
    ),
    complex_control_default_provider: get_defaults,
}

#[derive(Clone)]
pub struct HertzBox {
    tooltip: String,
    ccontrol: Rcrc<ep::ComplexControl>,
    pos: (f32, f32),
    range: (f32, f32),
    label: String,
}

impl HertzBox {
    const WIDTH: f32 = grid(3);
    const HEIGHT: f32 = grid(2) - FONT_SIZE - GRID_P / 2.0;
    pub fn create(
        registry: &Registry,
        pos: (f32, f32),
        ccontrol: Rcrc<ep::ComplexControl>,
        range: (f32, f32),
        label: String,
        tooltip: String,
    ) -> HertzBox {
        HertzBox {
            tooltip,
            ccontrol,
            pos,
            range,
            label,
        }
    }

    fn get_defaults(
        outline: &GeneratedHertzBoxOutline,
        yaml: &YamlNode,
    ) -> Result<Vec<(usize, String)>, String> {
        Ok(vec![(
            outline.control_index,
            if let Ok(child) = yaml.unique_child("default") {
                format!("{:.1}", child.f32()?)
            } else {
                format!("{:.1}", outline.range.0)
            },
        )])
    }
}

impl ModuleWidget for HertzBox {
    fn get_position(&self) -> (f32, f32) {
        self.pos
    }
    fn get_bounds(&self) -> (f32, f32) {
        (grid(3), grid(2))
    }
    fn respond_to_mouse_press(
        &self,
        local_pos: (f32, f32),
        mods: &MouseMods,
        parent_pos: (f32, f32),
    ) -> MouseAction {
        let click_delta = if local_pos.1 > HertzBox::HEIGHT / 2.0 {
            -1
        } else {
            1
        };
        MouseAction::ManipulateHertzControl {
            cref: Rc::clone(&self.ccontrol),
            min: self.range.0,
            max: self.range.1,
            precise_value: self.ccontrol.borrow().value.parse().unwrap(),
        }
    }

    fn get_tooltip_at(&self, local_pos: (f32, f32)) -> Option<Tooltip> {
        Some(Tooltip {
            text: self.tooltip.clone(),
            interaction: InteractionHint::LeftClickAndDrag | InteractionHint::DoubleClick,
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

        const W: f32 = HertzBox::WIDTH;
        const H: f32 = HertzBox::HEIGHT;
        const CS: f32 = CORNER_SIZE;
        g.set_color(&COLOR_BG);
        g.fill_rounded_rect(0.0, 0.0, W, H, CS);
        {
            let val = format!("{}hz", self.ccontrol.borrow().value);
            const HA: HAlign = HAlign::Right;
            const VA: VAlign = VAlign::Center;
            g.set_color(&COLOR_TEXT);
            g.write_text(
                BIG_FONT_SIZE,
                GRID_P,
                0.0,
                W - GRID_P * 2.0,
                H,
                HA,
                VA,
                1,
                &val,
            );
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
