use crate::engine::parts::{Control, GuiOutline, IOTab, Module, TabType, WidgetOutline};
use crate::engine::yaml::{self, YamlNode};
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

fn create_module_prototype_from_yaml(
    icon_indexes: &HashMap<String, usize>,
    yaml: &YamlNode,
    module_id: &str,
) -> Result<Module, String> {
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

    let mut inputs = Vec::new();
    for input_description in &yaml.unique_child("inputs")?.children {
        let type_name = &input_description.unique_child("type")?.value;
        let typ = TabType::from_str(type_name)
            .map_err(|_| format!("ERROR: {} is not a valid input type.", type_name))?;
        // The base library should always come with these icons.
        let icon = *icon_indexes.get(typ.icon_name()).unwrap();
        inputs.push(IOTab::create(typ, icon, input_description.name.clone()));
    }
    let mut outputs = Vec::new();
    for output_description in &yaml.unique_child("outputs")?.children {
        let type_name = &output_description.unique_child("type")?.value;
        let typ = TabType::from_str(type_name)
            .map_err(|_| format!("ERROR: {} is not a valid output type.", type_name))?;
        // The base library should always come with these icons.
        let icon = *icon_indexes.get(typ.icon_name()).unwrap();
        outputs.push(IOTab::create(typ, icon, output_description.name.clone()));
    }

    Ok(Module::create(
        rcrc(gui),
        controls,
        inputs,
        outputs,
        module_id.to_owned(),
        yaml.name.replace(".module.yaml", ".module.ns"),
    ))
}

pub struct Registry {
    modules: HashMap<String, Module>,
    scripts: HashMap<String, String>,
    icon_indexes: HashMap<String, usize>,
    icons: Vec<Vec<u8>>,
}

impl Registry {
    fn load_module_resource(
        &mut self,
        name: &str,
        module_id: &str,
        buffer: Vec<u8>,
    ) -> Result<(), String> {
        let buffer_as_text = String::from_utf8(buffer).map_err(|e| {
            format!(
                "ERROR: The file {} is not a valid UTF-8 text document, caused by:\nERROR: {}",
                name, e
            )
        })?;
        let yaml = yaml::parse_yaml(&buffer_as_text, name)?;
        let module = create_module_prototype_from_yaml(&self.icon_indexes, &yaml, &module_id)?;
        self.modules.insert(module.internal_id.clone(), module);
        Ok(())
    }

    fn load_script_resource(&mut self, name: &str, buffer: Vec<u8>) -> Result<(), String> {
        let buffer_as_text = String::from_utf8(buffer).map_err(|e| {
            format!(
                "ERROR: The file {} is not a valid UTF-8 text document, caused by:\nERROR: {}",
                name, e
            )
        })?;
        self.scripts.insert(name.to_owned(), buffer_as_text);
        Ok(())
    }

    fn strip_path_and_extension<'a>(full_path: &'a str, extension: &str) -> &'a str {
        let last_slash = full_path.rfind("/").unwrap_or(0);
        let extension_start = full_path.rfind(extension).unwrap_or(full_path.len());
        &full_path[last_slash + 1..extension_start]
    }

    fn load_library_impl(
        &mut self,
        lib_name: &str,
        lib_reader: impl Read + Seek,
    ) -> Result<(), String> {
        let mut reader = zip::ZipArchive::new(lib_reader).map_err(|e| format!("ERROR: {}", e))?;
        // Modules can refer to icons, so load all the icons before all the modules.
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
            if name.ends_with(".icon.svg") {
                let file_name = Self::strip_path_and_extension(file.name(), ".icon.svg");
                let icon_id = format!("{}:{}", lib_name, file_name);
                self.icon_indexes.insert(icon_id, self.icons.len());
                self.icons.push(buffer);
            } else {
                // Don't error here, we'll wait to the second loop to check if a file really is
                // unrecognized. That way we only have to maintain one set of conditions.
            }
        }
        // Now load the modules and other files.
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
                let file_name = Self::strip_path_and_extension(file.name(), ".module.yaml");
                let module_id = format!("{}:{}", lib_name, file_name);
                self.load_module_resource(&name, &module_id, buffer)?;
            } else if name.ends_with(".ns") {
                self.load_script_resource(&name, buffer)?;
            } else if name.ends_with(".md") {
                // Ignore, probably just readme / license type stuff.
            } else if name.ends_with(".icon.svg") {
                // Already loaded earlier.
            } else {
                return Err(format!(
                    "ERROR: Not sure what to do with the file {}.",
                    name
                ));
            }
        }
        Ok(())
    }

    fn load_library_from_file(&mut self, path: &Path) -> Result<(), String> {
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
            scripts: HashMap::new(),
            icon_indexes: HashMap::new(),
            icons: Vec::new(),
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

    pub fn borrow_scripts(&self) -> &HashMap<String, String> {
        &self.scripts
    }

    pub fn lookup_icon(&self, name: &str) -> Option<usize> {
        self.icon_indexes.get(name).cloned()
    }

    pub fn get_num_icons(&self) -> usize {
        self.icons.len()
    }

    pub fn borrow_icon_data(&self, index: usize) -> &[u8] {
        &self.icons[index][..]
    }
}
