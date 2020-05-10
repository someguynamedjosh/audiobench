use crate::engine::yaml::{self, YamlNode};
use crate::engine::{Control, GuiOutline, Module, WidgetOutline};
use crate::util::*;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;

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

fn create_module_prototype_from_yaml(yaml: &YamlNode, module_id: &str) -> Result<Module, String> {
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

    Ok(Module::create(
        rcrc(gui),
        controls,
        num_inputs,
        num_outputs,
        module_id.to_owned(),
        yaml.name.replace(".module.yaml", ".module.ns"),
    ))
}

pub struct Registry {
    modules: HashMap<String, Module>,
}

impl Registry {
    fn load_module_resource(&mut self, name: &str, module_id: &str, buffer: Vec<u8>) -> Result<(), String> {
        let buffer_as_text = String::from_utf8(buffer).map_err(|e| {
            format!(
                "ERROR: The file {} is not a valid UTF-8 text document, caused by:\nERROR: {}",
                name, e
            )
        })?;
        let yaml = yaml::parse_yaml(&buffer_as_text, name)?;
        let module = create_module_prototype_from_yaml(&yaml, &module_id)?;
        self.modules.insert(module.internal_id.clone(), module);
        Ok(())
    }

    fn load_library_impl(
        &mut self,
        lib_name: &str,
        lib_reader: impl Read + Seek,
    ) -> Result<(), String> {
        let mut reader = zip::ZipArchive::new(lib_reader).map_err(|e| format!("ERROR: {}", e))?;
        for index in 0..reader.len() {
            let mut file = reader.by_index(index).unwrap();
            let name = format!("{}:{}", lib_name, file.name());
            if name.ends_with("/") {
                // We don't do anything special with directories.
                continue;
            }
            let mut buffer = Vec::with_capacity(file.size() as usize);
            file.read_to_end(&mut buffer).map_err(|e| {
                format!(
                    "ERROR: Failed to read resource {}, caused by:\nERROR: {}",
                    file.name(),
                    e
                )
            })?;
            if name.ends_with(".module.yaml") {
                let file_name = file.name();
                let last_slash = file_name.rfind('/').unwrap_or(0);
                let extension_start = file_name.rfind(".module.yaml").unwrap_or(file_name.len());
                let file_name = &file_name[last_slash + 1..extension_start];
                let module_id = format!("{}:{}", lib_name, file_name);
                self.load_module_resource(&name, &module_id, buffer)?;
            } else {
                return Err(format!(
                    "ERROR: Not sure what to do with the file {}.",
                    name
                ));
            }
        }
        Ok(())
    }

    pub fn load_library_from_file(&mut self, path: &Path) -> Result<(), String> {
        let lib_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_owned();
        let extension_index = lib_name.rfind(".").unwrap_or(lib_name.len());
        let lib_name = (&lib_name[..extension_index]).to_owned();
        let file = File::open(path).map_err(|e| {
            format!(
                "ERROR: Failed to load library from {}, caused by:\nERROR: {}",
                path.to_string_lossy(),
                e
            )
        })?;
        self.load_library_impl(&lib_name, file).map_err(|e| {
            format!(
                "ERROR: Failed to load library from {}, caused by:\n{}",
                path.to_string_lossy(),
                e
            )
        })
    }

    pub fn new() -> (Self, Result<(), String>) {
        let mut registry = Self {
            modules: HashMap::new(),
        };

        let base_library = std::include_bytes!(concat!(env!("OUT_DIR"), "/base.ablib"));
        let reader = std::io::Cursor::new(base_library as &[u8]);
        let result = registry
            .load_library_impl("base", reader)
            .map_err(|e| format!("ERROR: Failed to load base library, caused by:\n{}", e));

        (registry, result)
    }

    pub fn borrow_module(&self, id: &str) -> Option<&Module> {
        self.modules.get(id)
    }
}
