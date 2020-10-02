use super::yaml::YamlNode;
use crate::engine::parts as ep;
use crate::engine::static_controls as staticons;
use crate::gui::module_widgets::WidgetOutline;
use shared_util::prelude::*;
use std::collections::{HashMap, HashSet};

fn create_control_from_yaml(yaml: &YamlNode) -> Result<Rcrc<ep::Autocon>, String> {
    let min = yaml.unique_child("min")?.parse()?;
    let max = yaml.unique_child("max")?.parse_ranged(Some(min), None)?;
    let default = yaml
        .unique_child("default")?
        .parse_ranged(Some(min), Some(max))?;
    let suffix = if let Ok(node) = yaml.unique_child("suffix") {
        node.value.clone()
    } else {
        "".to_owned()
    };
    Ok(rcrc(ep::Autocon::create(
        yaml.name.clone(),
        min,
        max,
        default,
        suffix,
    )))
}

pub(super) fn create_module_prototype_from_yaml(
    icon_indexes: &HashMap<String, usize>,
    lib_name: String,
    resource_name: String,
    yaml: &YamlNode,
) -> Result<ep::Module, String> {
    let mut controls = Vec::new();
    let mut existing_controls = HashSet::new();
    for control_description in &yaml.unique_child("autocons")?.children {
        if existing_controls.contains(&control_description.name) {
            return Err(format!(
                "ERROR: Duplicate entry for {}",
                control_description.full_name
            ));
        }
        existing_controls.insert(control_description.name.clone());
        controls.push(create_control_from_yaml(&control_description)?);
    }

    let mut staticons = Vec::new();
    if let Ok(child) = &yaml.unique_child("staticons") {
        for description in &child.children {
            // TODO: Error for duplicate control
            staticons.push(rcrc(staticons::from_yaml(description)?));
        }
    }

    let save_id = yaml
        .unique_child("save_id")?
        .parse_ranged(Some(0), Some(0xFFFF))?;

    let gui_description = yaml.unique_child("gui")?;
    let widgets_description = gui_description.unique_child("widgets")?;
    let label = gui_description.unique_child("label")?.value.clone();
    let category = gui_description.unique_child("category")?.value.clone();
    let tooltip = gui_description.unique_child("tooltip")?.value.clone();
    let width = gui_description
        .unique_child("width")?
        .parse_ranged(Some(0), None)?;
    let height = gui_description
        .unique_child("height")?
        .parse_ranged(Some(0), None)?;
    let mut widgets = Vec::new();
    for widget_description in &widgets_description.children {
        widgets.push(WidgetOutline::from_yaml(
            widget_description,
            &controls,
            &mut staticons,
        )?);
    }

    let mut inputs = Vec::new();
    let mut default_inputs = Vec::new();
    for input_description in &yaml.unique_child("inputs")?.children {
        let type_name = &input_description.unique_child("type")?.value;
        let typ = ep::JackType::from_str(type_name)
            .map_err(|_| format!("ERROR: {} is not a valid input type.", type_name))?;
        // The factory library should always come with these icons.
        let icon = *icon_indexes.get(typ.icon_name()).unwrap();
        let custom_icon = if let Ok(node) = input_description.unique_child("icon") {
            Some(
                *icon_indexes
                    .get(&node.value)
                    .ok_or_else(|| format!("ERROR: {} is not a valid icon name.", &node.value))?,
            )
        } else {
            None
        };
        let label = input_description.unique_child("label")?.value.clone();
        let tooltip = input_description.unique_child("tooltip")?.value.clone();
        default_inputs.push(
            if let Ok(node) = input_description.unique_child("default") {
                let index = node.parse()?;
                if index >= typ.get_num_defaults() {
                    0
                } else {
                    index
                }
            } else {
                0
            },
        );
        inputs.push(ep::IOJack::create(
            icon_indexes,
            typ,
            icon,
            custom_icon,
            input_description.name.clone(),
            label,
            tooltip,
        ));
    }
    let mut outputs = Vec::new();
    for output_description in &yaml.unique_child("outputs")?.children {
        let type_name = &output_description.unique_child("type")?.value;
        let typ = ep::JackType::from_str(type_name)
            .map_err(|_| format!("ERROR: {} is not a valid output type.", type_name))?;
        // The factory library should always come with these icons.
        let icon = *icon_indexes.get(typ.icon_name()).unwrap();
        let custom_icon = if let Ok(node) = output_description.unique_child("icon") {
            Some(
                *icon_indexes
                    .get(&node.value)
                    .ok_or_else(|| format!("ERROR: {} is not a valid icon name.", &node.value))?,
            )
        } else {
            None
        };
        let label = output_description.unique_child("label")?.value.clone();
        let tooltip = output_description.unique_child("tooltip")?.value.clone();
        outputs.push(ep::IOJack::create(
            icon_indexes,
            typ,
            icon,
            custom_icon,
            output_description.name.clone(),
            label,
            tooltip,
        ));
    }

    let feedback_data_len = widgets.iter().fold(0, |counter, item| {
        counter + item.get_feedback_data_requirement().size()
    });

    let template = ModuleTemplate {
        lib_name,
        resource_name,
        code_resource: yaml.name.replace(".module.yaml", ".module.ns"),
        template_id: save_id,

        label,
        category,
        tooltip,
        size: (width, height),
        widget_outlines: widgets,
        feedback_data_len,

        inputs,
        default_inputs: default_inputs.clone(),
        outputs,
    };

    Ok(ep::Module::create(
        rcrc(template),
        controls,
        staticons,
        default_inputs,
    ))
}

#[derive(Debug)]
pub struct ModuleTemplate {
    pub lib_name: String,
    pub resource_name: String,
    pub code_resource: String,
    pub template_id: usize,

    pub label: String,
    pub category: String,
    pub tooltip: String,
    pub size: (i32, i32),
    pub widget_outlines: Vec<WidgetOutline>,
    pub feedback_data_len: usize,

    pub inputs: Vec<ep::IOJack>,
    pub default_inputs: Vec<usize>,
    pub outputs: Vec<ep::IOJack>,
}
