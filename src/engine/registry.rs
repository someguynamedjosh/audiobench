use crate::engine::parts::{ComplexControl, Control, IOJack, JackType, Module, ModuleTemplate};
use crate::engine::save_data::Patch;
use crate::engine::yaml::{self, YamlNode};
use crate::gui::module_widgets::WidgetOutline;
use crate::util::*;
use rand::RngCore;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};

fn create_control_from_yaml(yaml: &YamlNode) -> Result<Rcrc<Control>, String> {
    let min = yaml.unique_child("min")?.f32()?;
    let max = yaml.unique_child("max")?.f32()?;
    let default = yaml.unique_child("default")?.f32()?;
    let suffix = if let Ok(node) = yaml.unique_child("suffix") {
        node.value.clone()
    } else {
        "".to_owned()
    };
    Ok(rcrc(Control::create(
        yaml.name.clone(),
        min,
        max,
        default,
        suffix,
    )))
}

fn create_widget_outline_from_yaml(
    yaml: &YamlNode,
    controls: &Vec<Rcrc<Control>>,
    complex_controls: &mut Vec<Rcrc<ComplexControl>>,
) -> Result<WidgetOutline, String> {
    let x = yaml.unique_child("x")?.i32()?;
    let y = yaml.unique_child("y")?.i32()?;
    let grid_pos = (x, y);
    let tooltip_node = yaml.unique_child("tooltip");
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
    let find_complex_control_index = |name: &str| {
        complex_controls
            .iter()
            .position(|item| &item.borrow().code_name == name)
            .ok_or_else(|| {
                format!(
                    "ERROR: Invalid widget {}, caused by:\nERROR: No complex control named {}.",
                    &yaml.full_name, name
                )
            })
    };
    let mut set_default = None;
    let outline = match &yaml.name[..] {
        "knob" => {
            let control_name = &yaml.unique_child("control")?.value;
            let control_index = find_control_index(control_name)?;
            let label = yaml.unique_child("label")?.value.clone();
            WidgetOutline::Knob {
                tooltip: tooltip_node?.value.clone(),
                control_index,
                grid_pos,
                label,
            }
        }
        "envelope_graph" => {
            let grid_size = (
                yaml.unique_child("w")?.i32()?,
                yaml.unique_child("h")?.i32()?,
            );
            let feedback_name = yaml.unique_child("feedback_name")?.value.clone();
            WidgetOutline::EnvelopeGraph {
                grid_pos,
                grid_size,
                feedback_name,
            }
        }
        "waveform_graph" => {
            let grid_size = (
                yaml.unique_child("w")?.i32()?,
                yaml.unique_child("h")?.i32()?,
            );
            let feedback_name = yaml.unique_child("feedback_name")?.value.clone();
            WidgetOutline::WaveformGraph {
                grid_pos,
                grid_size,
                feedback_name,
            }
        }
        "int_box" => {
            let ccontrol_name = &yaml.unique_child("control")?.value;
            let ccontrol_index = find_complex_control_index(ccontrol_name)?;
            let min = yaml.unique_child("min")?.i32()?;
            let max = yaml.unique_child("max")?.i32()?;
            let default = if let Ok(child) = yaml.unique_child("default") {
                child.i32()?
            } else {
                min
            };
            let label = yaml.unique_child("label")?.value.clone();
            set_default = Some((ccontrol_index, format!("{}", default)));
            WidgetOutline::IntBox {
                tooltip: tooltip_node?.value.clone(),
                ccontrol_index,
                grid_pos,
                range: (min, max),
                label,
            }
        }
        _ => {
            return Err(format!(
                "ERROR: Invalid widget {}, caused by:\nERROR: {} is not a valid widget type.",
                &yaml.full_name, &yaml.name
            ))
        }
    };
    if let Some((index, value)) = set_default {
        if complex_controls[index].borrow().value != "" {
            return Err(format!(
                "ERROR: Multiple widgets controlling the same complex control {}.",
                complex_controls[index].borrow().code_name
            ));
        }
        complex_controls[index].borrow_mut().default = value.clone();
        complex_controls[index].borrow_mut().value = value;
    }
    Ok(outline)
}

fn create_module_prototype_from_yaml(
    icon_indexes: &HashMap<String, usize>,
    resource_name: String,
    yaml: &YamlNode,
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

    let mut complex_controls = Vec::new();
    if let Ok(child) = &yaml.unique_child("complex_controls") {
        for description in &child.children {
            // TODO: Error for duplicate control
            complex_controls.push(rcrc(ComplexControl {
                code_name: description.name.clone(),
                value: "".to_owned(),
                default: "".to_owned(),
            }));
        }
    }

    let gui_description = yaml.unique_child("gui")?;
    let widgets_description = gui_description.unique_child("widgets")?;
    let label = gui_description.unique_child("label")?.value.clone();
    let category = gui_description.unique_child("category")?.value.clone();
    let tooltip = gui_description.unique_child("tooltip")?.value.clone();
    let width = gui_description.unique_child("width")?.i32()?;
    let height = gui_description.unique_child("height")?.i32()?;
    let mut widgets = Vec::new();
    for widget_description in &widgets_description.children {
        widgets.push(create_widget_outline_from_yaml(
            widget_description,
            &controls,
            &mut complex_controls,
        )?);
    }

    for control in &complex_controls {
        if control.borrow().value == "" {
            return Err(format!(
                "ERROR: No widget was created for the complex control {}",
                control.borrow().code_name
            ));
        }
    }

    let mut inputs = Vec::new();
    let mut default_inputs = Vec::new();
    for input_description in &yaml.unique_child("inputs")?.children {
        let type_name = &input_description.unique_child("type")?.value;
        let typ = JackType::from_str(type_name)
            .map_err(|_| format!("ERROR: {} is not a valid input type.", type_name))?;
        // The base library should always come with these icons.
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
                let index = node.i32()? as usize;
                if index >= typ.get_num_defaults() {
                    0
                } else {
                    index
                }
            } else {
                0
            },
        );
        inputs.push(IOJack::create(
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
        let typ = JackType::from_str(type_name)
            .map_err(|_| format!("ERROR: {} is not a valid output type.", type_name))?;
        // The base library should always come with these icons.
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
        outputs.push(IOJack::create(
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
        resource_name,
        label,
        category,
        tooltip,
        code_resource: yaml.name.replace(".module.yaml", ".module.ns"),
        size: (width, height),
        widget_outlines: widgets,
        inputs,
        default_inputs: default_inputs.clone(),
        outputs,
        feedback_data_len,
    };

    Ok(Module::create(
        rcrc(template),
        controls,
        complex_controls,
        default_inputs,
    ))
}

pub struct Registry {
    modules: HashMap<String, Module>,
    scripts: HashMap<String, String>,
    icon_indexes: HashMap<String, usize>,
    icons: Vec<Vec<u8>>,
    patches: Vec<Rcrc<Patch>>,
    patch_paths: HashMap<String, usize>,
    user_library_path: PathBuf,
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
        let module =
            create_module_prototype_from_yaml(&self.icon_indexes, module_id.to_owned(), &yaml)?;
        self.modules.insert(module_id.to_owned(), module);
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

    fn load_patch(
        &mut self,
        name: &str,
        full_path: Option<PathBuf>,
        buffer: Vec<u8>,
    ) -> Result<(), String> {
        let mut reader = std::io::BufReader::new(std::io::Cursor::new(buffer));
        let patch = if let Some(full_path) = full_path {
            crate::engine::save_data::Patch::load_writable(full_path, &mut reader)
        } else {
            crate::engine::save_data::Patch::load_readable(name.to_owned(), &mut reader)
        }
        .map_err(|err| {
            format!(
                "ERROR: The file {} could not be loaded, caused by:\n{}",
                name, err
            )
        })?;
        self.patch_paths.insert(name.to_owned(), self.patches.len());
        self.patches.push(rcrc(patch));
        Ok(())
    }

    fn strip_path_and_extension<'a>(full_path: &'a str, extension: &str) -> &'a str {
        let last_slash = full_path.rfind("/").unwrap_or(0);
        let extension_start = full_path.rfind(extension).unwrap_or(full_path.len());
        &full_path[last_slash + 1..extension_start]
    }

    fn load_resource(
        &mut self,
        lib_name: &str,
        file_name: &str,
        full_path: Option<PathBuf>,
        buffer: Vec<u8>,
    ) -> Result<(), String> {
        let full_name = format!("{}:{}", lib_name, file_name);
        if file_name.ends_with(".icon.svg") {
            let file_name = Self::strip_path_and_extension(file_name, ".icon.svg");
            let icon_id = format!("{}:{}", lib_name, file_name);
            self.icon_indexes.insert(icon_id, self.icons.len());
            self.icons.push(buffer);
        } else if file_name.ends_with(".module.yaml") {
            let file_name = Self::strip_path_and_extension(file_name, ".module.yaml");
            let module_id = format!("{}:{}", lib_name, file_name);
            self.load_module_resource(&full_name, &module_id, buffer)?;
        } else if file_name.ends_with(".ns") {
            self.load_script_resource(&full_name, buffer)?;
        } else if file_name.ends_with(".abpatch") {
            self.load_patch(&full_name, full_path, buffer)?;
        } else if file_name.ends_with(".md") {
            // Ignore, probably just readme / license type stuff.
        } else {
            return Err(format!(
                "ERROR: Not sure what to do with the file {}.",
                full_name
            ));
        }
        Ok(())
    }

    fn load_zipped_library(
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
                    name, e
                )
            })?;
            // Only load icons right now, wait until later to load anything else.
            if name.ends_with(".icon.svg") {
                self.load_resource(lib_name, file.name(), None, buffer)?;
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
                    name, e
                )
            })?;
            if !name.ends_with(".icon.svg") {
                self.load_resource(lib_name, file.name(), None, buffer)?;
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
        self.load_zipped_library(&lib_name, file).map_err(|e| {
            format!(
                "ERROR: Failed to load library from {}, caused by:\n{}",
                path.to_string_lossy(),
                e
            )
        })
    }

    fn load_library_from_folder(&mut self, path: &Path) -> Result<(), String> {
        assert!(path.is_dir());
        let mut file_paths = Vec::new();
        let mut unvisited_paths = vec![PathBuf::new()];
        while let Some(visiting) = unvisited_paths.pop() {
            let reader = fs::read_dir(path.join(&visiting)).map_err(|err| {
                format!(
                    "ERROR: Failed to load a library from {}, caused by:\n{}",
                    path.to_string_lossy(),
                    err
                )
            })?;
            for entry in reader {
                let entry = if let Ok(entry) = entry {
                    entry
                } else {
                    continue;
                };
                let path = entry.path();
                let local_path = visiting.join(entry.file_name());
                if path.is_dir() {
                    unvisited_paths.push(local_path);
                } else {
                    file_paths.push(local_path);
                }
            }
        }
        let lib_name = path.file_name().unwrap_or_default().to_string_lossy();
        fn read_file(lib_name: &str, full_path: &Path) -> Result<Vec<u8>, String> {
            fs::read(&full_path).map_err(|err| {
                format!(
                    concat!(
                        "ERROR: Failed to load library {}, caused by:\n",
                        "ERROR: Failed to read from file {}, caused by:\n",
                        "{}"
                    ),
                    lib_name,
                    full_path.to_string_lossy(),
                    err
                )
            })
        }
        // Load only icons.
        for file_path in &file_paths {
            if file_path.ends_with(".icon.svg") {
                let full_path = path.join(&file_path);
                let buffer = read_file(&lib_name, &full_path)?;
                self.load_resource(
                    &lib_name,
                    &file_path.to_string_lossy(),
                    Some(full_path),
                    buffer,
                )?;
            }
        }
        // Load other files.
        for file_path in file_paths {
            if !file_path.ends_with(".icon.svg") {
                let full_path = path.join(&file_path);
                let buffer = read_file(&lib_name, &full_path)?;
                self.load_resource(
                    &lib_name,
                    &file_path.to_string_lossy(),
                    Some(full_path),
                    buffer,
                )?;
            }
        }
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), String> {
        let base_library = std::include_bytes!(concat!(env!("OUT_DIR"), "/base.ablib"));
        let reader = std::io::Cursor::new(base_library as &[u8]);
        self.load_zipped_library("base", reader)
            .map_err(|e| format!("ERROR: Failed to load base library, caused by:\n{}", e))?;

        fs::create_dir_all(&self.user_library_path).map_err(|err| {
            format!(
                "ERROR: Failed to create user library at {}, caused by:\n{}",
                self.user_library_path.to_string_lossy(),
                err
            )
        })?;
        let ulp = self.user_library_path.clone();
        self.load_library_from_folder(&ulp)?;

        Ok(())
    }

    pub fn new() -> (Self, Result<(), String>) {
        let user_library_path = {
            let user_dirs = directories::UserDirs::new().unwrap();
            let document_dir = user_dirs.document_dir().unwrap();
            document_dir.join("Audiobench").join("user")
        };

        let mut registry = Self {
            modules: HashMap::new(),
            scripts: HashMap::new(),
            icon_indexes: HashMap::new(),
            icons: Vec::new(),
            patches: Vec::new(),
            patch_paths: HashMap::new(),
            user_library_path,
        };
        let result = registry.initialize();

        (registry, result)
    }

    pub fn borrow_module(&self, id: &str) -> Option<&Module> {
        self.modules.get(id)
    }

    pub fn iterate_over_modules(&self) -> impl Iterator<Item = &Module> {
        self.modules.values()
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

    pub fn create_new_user_patch(&mut self) -> &Rcrc<Patch> {
        let filename = format!("{:016X}.abpatch", rand::thread_rng().next_u64());
        self.patch_paths.insert(format!("user:{}", filename), self.patches.len());
        let patch = Patch::writable(self.user_library_path.join(filename));
        let prc = rcrc(patch);
        self.patches.push(prc);
        self.patches.last().unwrap()
    }

    pub fn get_patch_by_name(&self, name: &str) -> Option<&Rcrc<Patch>> {
        self.patch_paths.get(name).map(|i| &self.patches[*i])
    }
}
