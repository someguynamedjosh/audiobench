use crate::engine::yaml::{self, YamlNode};
use crate::engine::{Control, GuiOutline, Module, WidgetOutline};
use std::collections::{HashMap, HashSet};
use crate::util::*;

fn create_control_from_yaml(yaml: &YamlNode) -> Result<Rcrc<Control>, String> {
    let min = yaml.unique_child("min")?.f32()?;
    let max = yaml.unique_child("max")?.f32()?;
    let default = yaml.unique_child("default")?.f32()?;
    Ok(rcrc(Control::create(yaml.name.clone(), min, max, default)))
}

fn create_widget_outline_from_yaml(
    yaml: &YamlNode,
    controls: &Vec<Rcrc<Control>>,
) -> Result<WidgetOutline, String> {
    let x = yaml.unique_child("x")?.i32()?;
    let y = yaml.unique_child("y")?.i32()?;
    let grid_pos = (x, y);
    let find_control_index = |name: &str| {
        controls
            .iter()
            .position(|item| &item.borrow().code_name == name)
            .ok_or_else(|| {
                format!(
                    "ERROR: Invalid widget {}, caused by:\nERROR: No control named {}.",
                    &yaml.full_name, name
                )
            })
    };
    match &yaml.name[..] {
        "knob" => {
            let control_name = &yaml.unique_child("control")?.value;
            let control_index = find_control_index(control_name)?;
            let label = yaml.unique_child("label")?.value.clone();
            Ok(WidgetOutline::Knob {
                control_index,
                grid_pos,
                label,
            })
        }
        _ => Err(format!(
            "ERROR: Invalid widget {}, caused by:\nERROR: {} is not a valid widget type.",
            &yaml.full_name, &yaml.name
        )),
    }
}

fn create_module_prototype_from_yaml(yaml: &YamlNode) -> Result<Module, String> {
    let mut controls = Vec::new();
    let mut existing_controls = HashSet::new();
    for control_description in &yaml.unique_child("controls")?.children {
        if existing_controls.contains(&control_description.name) {
            return Err(format!(
                "ERROR: Duplicate entry for {}",
                control_description.full_name
            ));
        }
        existing_controls.insert(control_description.name.clone());
        controls.push(create_control_from_yaml(&control_description)?);
    }

    let gui_description = yaml.unique_child("gui")?;
    let widgets_description = gui_description.unique_child("widgets")?;
    let label = gui_description.unique_child("label")?.value.clone();
    let width = gui_description.unique_child("width")?.i32()?;
    let height = gui_description.unique_child("height")?.i32()?;
    let mut widgets = Vec::new();
    for widget_description in &widgets_description.children {
        widgets.push(create_widget_outline_from_yaml(
            widget_description,
            &controls,
        )?);
    }
    let gui = GuiOutline {
        label,
        size: (width, height),
        widget_outlines: widgets,
    };

    let num_inputs = yaml.unique_child("inputs")?.children.len();
    let num_outputs = yaml.unique_child("outputs")?.children.len();

    Ok(Module::create(rcrc(gui), controls, num_inputs, num_outputs))
}

pub struct Registry {}

impl Registry {
    pub fn new() -> Self {
        let file = std::include_str!("../../modules/oscillator.yaml");
        let parsed = yaml::parse_yaml(file, "embedded").unwrap();
        let module = create_module_prototype_from_yaml(&parsed);
        println!("{:#?}", module);
        Self {}
    }
}
