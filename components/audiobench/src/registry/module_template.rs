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
    mut yaml: YamlNode,
) -> Result<ModuleTemplate, String> {
    let mut controls = Vec::new();
    if let Ok(mut child) = yaml.map_entry("controls") {
        for (key, description) in child.map_entries()? {
            // TODO: Error for duplicate control
            controls.push(controls::from_yaml(key, description)?);
        }
    }

    let save_id = yaml
        .map_entry("save_id")?
        .parse_ranged(Some(0), Some(0xFFFF))?;

    let mut gui_description = yaml.map_entry("gui")?;
    let mut widgets_description = gui_description.map_entry("widgets")?;
    let label = gui_description.map_entry("label")?.value()?.to_owned();
    let category = gui_description.map_entry("category")?.value()?.to_owned();
    let tooltip = gui_description.map_entry("tooltip")?.value()?.to_owned();
    let width = gui_description
        .map_entry("width")?
        .parse_ranged(Some(0), None)?;
    let height = gui_description
        .map_entry("height")?
        .parse_ranged(Some(0), None)?;
    let mut widgets = Vec::new();
    for description in widgets_description.list_entries()? {
        widgets.push(WidgetOutline::from_yaml(
            description,
            icon_indexes,
            &mut controls,
        )?);
    }

    let mut outputs = Vec::new();
    for (key, mut output_description) in yaml.map_entry("outputs")?.map_entries()? {
        let type_name_entry = output_description.map_entry("datatype")?;
        let type_name = type_name_entry.value()?;
        let typ = ep::JackType::from_str(type_name)
            .map_err(|_| format!("ERROR: {} is not a valid output type.", type_name))?;
        // The factory library should always come with these icons.
        let icon = *icon_indexes.get(typ.icon_name()).unwrap();
        let custom_icon = if let Ok(node) = output_description.map_entry("icon") {
            let value = node.value()?;
            Some(
                *icon_indexes
                    .get(value)
                    .ok_or_else(|| format!("ERROR: {} is not a valid icon name.", value))?,
            )
        } else {
            None
        };
        let label = output_description.map_entry("label")?.value()?.to_owned();
        let tooltip = output_description.map_entry("tooltip")?.value()?.to_owned();
        outputs.push(ep::IOJack::create(
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
