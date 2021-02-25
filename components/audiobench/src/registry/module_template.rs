use crate::{
    engine::{
        controls::{self, AnyControl},
        parts as ep,
    },
    gui::module_widgets::WidgetOutline,
    registry::yaml::YamlNode,
};
use std::collections::HashMap;

pub(super) fn create_module_template_from_yaml(
    icon_indexes: &HashMap<String, usize>,
    lib_name: String,
    resource_name: String,
    yaml: &YamlNode,
) -> Result<ModuleTemplate, String> {
    let mut controls = Vec::new();
    if let Ok(child) = &yaml.unique_child("controls") {
        for description in &child.children {
            // TODO: Error for duplicate control
            controls.push(controls::from_yaml(description)?);
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
            icon_indexes,
            &mut controls,
        )?);
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

    let name_start = yaml
        .name
        .rfind('/')
        .or_else(|| yaml.name.rfind(':'))
        .expect("Illegal file name")
        + 1;
    let name_end = yaml.name.rfind(".module.yaml").expect("Illegal file name");
    let name = String::from(&yaml.name[name_start..name_end]);

    Ok(ModuleTemplate {
        lib_name,
        module_name: name,
        save_id,

        label,
        category,
        tooltip,
        size: (width, height),
        widget_outlines: widgets,

        default_controls: controls,
        outputs,
    })
}

#[derive(Debug)]
pub struct ModuleTemplate {
    pub lib_name: String,
    pub module_name: String,
    pub save_id: usize,

    pub label: String,
    pub category: String,
    pub tooltip: String,
    pub size: (i32, i32),
    pub widget_outlines: Vec<WidgetOutline>,

    /// First field is code name, second field is control.
    pub default_controls: Vec<(String, AnyControl)>,
    pub outputs: Vec<ep::IOJack>,
}
