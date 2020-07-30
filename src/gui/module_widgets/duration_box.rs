use super::ModuleWidget;
use crate::engine::parts as ep;
use crate::gui::action::MouseAction;
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::{InteractionHint, MouseMods, Tooltip};
use crate::registry::yaml::YamlNode;
use crate::util::*;

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: DurationBox,
    constructor: create(
        pos: GridPos,
        control: ComplexControlRef,
        type_control: ComplexControlRef,
        label: String,
        tooltip: String,
    ),
    complex_control_default_provider: get_defaults,
}

#[derive(Clone)]
pub struct DurationBox {
    tooltip: String,
    ccontrol: Rcrc<ep::ComplexControl>,
    type_control: Rcrc<ep::ComplexControl>,
    pos: (f32, f32),
    label: String,
}

impl DurationBox {
    const WIDTH: f32 = grid(2);
    const HEIGHT: f32 = grid(2) - FONT_SIZE - GRID_P / 2.0;
    pub fn create(
        pos: (f32, f32),
        ccontrol: Rcrc<ep::ComplexControl>,
        type_control: Rcrc<ep::ComplexControl>,
        label: String,
        tooltip: String,
    ) -> DurationBox {
        DurationBox {
            tooltip,
            ccontrol,
            type_control,
            pos,
            label,
        }
    }

    fn get_defaults(
        outline: &GeneratedDurationBoxOutline,
        yaml: &YamlNode,
    ) -> Result<Vec<(usize, String)>, String> {
        Ok(vec![(
            outline.control_index,
            if let Ok(child) = yaml.unique_child("default") {
                child.value.trim().to_owned()
            } else {
                "1.00".to_owned()
            },
        )])
    }
}

impl ModuleWidget for DurationBox {
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
        _parent_pos: (f32, f32),
    ) -> MouseAction {
        let val = &self.ccontrol.borrow().value;
        if val.contains('/') {
            let slash_pos = val.find('/').unwrap();
            let num = (val[..slash_pos]).parse::<f32>().unwrap() as i32;
            let den = (val[slash_pos + 1..]).parse::<f32>().unwrap() as i32;
            let use_denominator = local_pos.0 >= Self::WIDTH / 2.0;
            if mods.right_click {
                // Toggle mode
                let value = num as f32 / den as f32;
                let decimals = if value < 0.999 {
                    3
                } else if value < 9.99 {
                    2
                } else if value < 99.9 {
                    1
                } else {
                    0
                };
                let str_value = format!("{:.1$}", value, decimals);
                MouseAction::SetComplexControl(Rc::clone(&self.ccontrol), str_value)
            } else {
                MouseAction::ManipulateDurationControl {
                    cref: Rc::clone(&self.ccontrol),
                    precise_value: if use_denominator { den } else { num } as f32,
                    denominator: use_denominator,
                }
            }
        } else {
            if mods.right_click {
                // Toggle mode
                let str_value = "1.0/1.0".to_owned();
                MouseAction::SetComplexControl(Rc::clone(&self.ccontrol), str_value)
            } else {
                MouseAction::ManipulateDurationControl {
                    cref: Rc::clone(&self.ccontrol),
                    precise_value: self.ccontrol.borrow().value.parse().unwrap(),
                    denominator: false,
                }
            }
        }
    }

    fn get_tooltip_at(&self, _local_pos: (f32, f32)) -> Option<Tooltip> {
        Some(Tooltip {
            text: self.tooltip.clone(),
            interaction: InteractionHint::LeftClickAndDrag | InteractionHint::RightClick,
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

        const W: f32 = DurationBox::WIDTH;
        const H: f32 = DurationBox::HEIGHT;
        const CS: f32 = CORNER_SIZE;
        g.set_color(&COLOR_BG);
        g.fill_rounded_rect(0.0, 0.0, W, H, CS);
        {
            let is_beats = self.type_control.borrow().value == "TRUE";
            let val = &self.ccontrol.borrow().value;
            let val = if val.contains('/') {
                let slash_pos = val.find('/').unwrap();
                let num = (val[..slash_pos]).parse::<f32>().unwrap() as i32;
                let den = (val[slash_pos + 1..]).parse::<f32>().unwrap() as i32;
                format!("{}/{}{}", num, den, if is_beats { "b" } else { "s" })
            } else {
                format!("{}{}", val, if is_beats { "b" } else { "s" })
            };
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
