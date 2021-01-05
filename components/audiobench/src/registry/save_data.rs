use super::mini_bin::*;
use super::module_template::ModuleTemplate;
use super::Registry;
use crate::config::*;
use crate::engine::controls::{AnyControl, Control};
use crate::engine::parts as ep;
use shared_util::prelude::*;
use std::collections::{HashMap, HashSet};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Clone)]
struct SavedControl {
    value: Vec<u8>,
}

impl SavedControl {
    fn save(control: &AnyControl) -> Self {
        let mut value = Vec::new();
        control.as_dyn_ptr().borrow().serialize(&mut value);
        Self { value }
    }

    fn restore(&self, on: &mut AnyControl) {
        let mut data_slice = &self.value[..];
        // Our own deserialize() method should have returned an error if the data's deserialization
        // method would cause an error.
        on.as_dyn_ptr()
            .borrow_mut()
            .deserialize(&mut data_slice)
            .unwrap();
    }

    fn serialize(&self, buffer: &mut Vec<u8>) {
        buffer.append(&mut self.value.clone());
    }

    /// prototypical_dummy_control is a sacrificial value that should be a clone of the actual
    /// control this SavedControl is storing data for. It will be used to determine how much data
    /// should be saved from the slice to deserialize in the *actual* control during `restore`.
    fn deserialize(slice: &mut &[u8], prototypical_dummy_control: AnyControl) -> Result<Self, ()> {
        let old_slice = *slice;
        // Basically have the dummy control deserialize the data and then just store the section
        // of data it deserialized.
        prototypical_dummy_control
            .as_dyn_ptr()
            .borrow_mut()
            .deserialize(slice)?;
        let data_len = old_slice.len() - slice.len();
        let value = Vec::from(&old_slice[..data_len]);
        Ok(Self { value })
    }
}

#[derive(Debug, Clone)]
struct SavedModule {
    lib_name: String,
    template_id: usize,
    controls: Vec<SavedControl>,
    pos: (i32, i32),
}

impl SavedModule {
    fn save(
        module: &ep::Module,
        module_indexes: &HashMap<*const RefCell<ep::Module>, usize>,
    ) -> Self {
        let template_ref = module.template.borrow();
        let lib_name = template_ref.lib_name.clone();
        let template_id = template_ref.template_id;
        let controls = module.controls.imc(SavedControl::save);
        let pos = module.pos;
        let pos = (pos.0 as i32, pos.1 as i32);
        drop(template_ref);
        Self {
            lib_name,
            template_id,
            controls,
            pos,
        }
    }

    fn restore<'a>(&self, registry: &Registry) -> Result<ep::Module, ()> {
        let template = self.lookup_template(registry)?;
        let mut m = ep::Module::create(Rc::clone(template));
        m.pos = (self.pos.0 as f32, self.pos.1 as f32);
        for index in 0..self.controls.len() {
            self.controls[index].restore(&mut m.controls[index]);
        }
        Ok(m)
    }

    fn lookup_template<'a>(&self, registry: &'a Registry) -> Result<&'a Rcrc<ModuleTemplate>, ()> {
        registry
            .borrow_template_by_serialized_id(&(self.lib_name.clone(), self.template_id))
            .ok_or(())
    }

    /// Gets the number of controls this module should have when
    /// fully restored.
    fn get_num_controls(&self, registry: &Registry) -> Result<usize, ()> {
        let t = self.lookup_template(registry)?;
        Ok(t.borrow().default_controls.len())
    }

    fn get_prototypical_dummy_controls(&self, registry: &Registry) -> Result<Vec<AnyControl>, ()> {
        let t = self.lookup_template(registry)?;
        Ok(t.borrow()
            .default_controls
            .iter()
            .map(|(_, s)| s.deep_clone())
            .collect())
    }

    // The mode will indicate how data about this module is stored. Different modes take up
    // different amount of space, so this helps save space.
    fn get_ser_mode_u2(&self) -> u8 {
        let small_coords = self.pos.0.abs() < 0x7FFF && self.pos.1.abs() < 0x7FFF;
        let small_resource = self.lib_name == "Factory" && self.template_id < 0xFF;
        compose_u2(small_coords, small_resource)
    }

    fn serialize(&self, buffer: &mut Vec<u8>, mode_u2: u8, additional_libs: &[(String, u16)]) {
        let (small_coords, small_resource) = decompose_u2(mode_u2);
        if small_coords {
            ser_i16(buffer, self.pos.0 as i16);
            ser_i16(buffer, self.pos.1 as i16);
        } else {
            ser_i32(buffer, self.pos.0);
            ser_i32(buffer, self.pos.1);
        }
        if small_resource {
            ser_u8(buffer, self.template_id as u8);
        } else {
            let lib_id = if self.lib_name == "Factory" {
                0
            } else {
                additional_libs
                    .iter()
                    .position(|i| &i.0 == &self.lib_name)
                    .unwrap()
                    + 1
            };
            assert!(lib_id <= 0xFF);
            ser_u8(buffer, lib_id as u8);
            ser_u16(buffer, self.template_id as u16);
        }
    }

    fn deserialize(
        slice: &mut &[u8],
        mode_u2: u8,
        additional_libs: &[(String, u16)], // Name and version number.
    ) -> Result<Self, ()> {
        let (small_coords, small_resource) = decompose_u2(mode_u2);
        let (x, y) = if small_coords {
            (des_i16(slice)? as i32, des_i16(slice)? as i32)
        } else {
            (des_i32(slice)?, des_i32(slice)?)
        };
        let (lib_index, template_index) = if small_resource {
            (0, des_u8(slice)? as usize)
        } else {
            (des_u8(slice)? as usize, des_u16(slice)? as usize)
        };
        let lib_name = if lib_index == 0 {
            "Factory".to_owned()
        } else {
            additional_libs
                .get(lib_index as usize - 1)
                .ok_or(())?
                .0
                .clone()
        };
        Ok(Self {
            lib_name,
            template_id: template_index,
            controls: Vec::new(),
            pos: (x, y),
        })
    }
}

#[derive(Debug, Clone)]
struct SavedModuleGraph {
    modules: Vec<SavedModule>,
    factory_lib_version: u16,
    additional_libs: Vec<(String, u16)>,
}

impl SavedModuleGraph {
    fn blank() -> Self {
        Self {
            modules: Default::default(),
            // Engine version and factory lib version are always identical.
            factory_lib_version: ENGINE_VERSION,
            additional_libs: Vec::new(),
        }
    }

    fn save(graph: &ep::ModuleGraph, registry: &Registry) -> Self {
        let mut module_indexes: HashMap<*const RefCell<ep::Module>, usize> = HashMap::new();
        for (index, module) in graph.borrow_modules().iter().enumerate() {
            module_indexes.insert(&*module.as_ref(), index);
        }
        let module_indexes = module_indexes;
        let modules = graph
            .borrow_modules()
            .imc(|module| SavedModule::save(&*module.borrow(), &module_indexes));
        let mut library_names = HashSet::new();
        for module in &modules {
            library_names.insert(module.lib_name.clone());
        }
        // The factory library is stored more compactly as it is always assumed to be present.
        library_names.remove("Factory");
        let additional_libs = library_names.imc(|name| {
            let version = registry
                .borrow_library_info(name)
                .expect("Currently running patch uses an unloaded library?")
                .version;
            (name.to_owned(), version)
        });
        Self {
            modules,
            factory_lib_version: ENGINE_VERSION,
            additional_libs,
        }
    }

    fn restore(&self, graph: &mut ep::ModuleGraph, registry: &Registry) -> Result<(), String> {
        if self.factory_lib_version > ENGINE_VERSION {
            return Err(format!(
                "ERROR: this patch requires Audiobench v{} or newer, you are currently running v{}.",
                self.factory_lib_version, ENGINE_VERSION
            ));
        }
        for (lib_name, lib_version) in &self.additional_libs {
            if let Some(lib_info) = registry.borrow_library_info(lib_name) {
                if lib_info.version < *lib_version {
                    return Err(format!(
                        concat!(
                            "ERROR: this patch requires {} v{} or newer, you currently have ",
                            "installed v{}"
                        ),
                        lib_name, *lib_version, lib_info.version
                    ));
                }
            } else {
                return Err(format!(
                    "ERROR: this patch requires a library you do not have: {} v{} or newer",
                    lib_name, *lib_version
                ));
            }
        }
        let modules: Vec<_> = self
            .modules
            .iter()
            .map(|m| m.restore(registry).map(|m| rcrc(m)))
            .collect::<Result<_, _>>()
            .map_err(|_| format!("ERROR: Patch data is corrupt (failed to restore modules.)"))?;
        for index in 0..self.modules.len() {
            let module = Rc::clone(&modules[index]);
            // self.modules[index]
            //     .restore_connections(&mut *module.borrow_mut(), &modules[..])
            //     .map_err(|_| {
            //         format!("ERROR: Patch data is corrupt (failed to restore connections.)")
            //     })?;
        }
        graph.set_modules(modules);
        Ok(())
    }

    fn serialize(&self, buffer: &mut Vec<u8>) {
        // FE not FF because it doesn't include Factory.
        assert!(self.additional_libs.len() < 0xFE);
        ser_u8(buffer, self.additional_libs.len() as u8);
        ser_u16(buffer, ENGINE_VERSION);
        for (lib_name, lib_version) in &self.additional_libs {
            ser_str(buffer, lib_name);
            ser_u16(buffer, *lib_version);
        }
        // The biggest indexes (for connections) are 12 bits long.
        assert!(self.modules.len() <= 0xFFF);
        ser_u16(buffer, self.modules.len() as u16);
        let mod_modes: Vec<_> = self.modules.imc(|module| module.get_ser_mode_u2());
        ser_u2_slice(buffer, &mod_modes[..]);
        for index in 0..self.modules.len() {
            self.modules[index].serialize(buffer, mod_modes[index], &self.additional_libs[..]);
        }

        for module in &self.modules {
            for control in &module.controls {
                control.serialize(buffer);
            }
        }
    }

    fn deserialize(slice: &mut &[u8], format_version: u8, registry: &Registry) -> Result<Self, ()> {
        let num_libs = des_u8(slice)?;
        // After version 1, the version number of the Factory library is included in the patch.
        let factory_lib_version = if format_version >= 1 {
            des_u16(slice)?
        } else {
            // Before format version 1, the Factory library was always at version 0.
            0
        };
        // If dependencies_ok is false, we will skip loading the rest of the data in the patch. When
        // restore() is called, a friendly error will be returned with the cause of the problem.
        let mut dependencies_ok = factory_lib_version <= ENGINE_VERSION;
        let mut additional_libs = Vec::new();
        for _ in 0..num_libs {
            // No presets exist before version 1 that used additional libraries.
            assert!(format_version >= 1);
            let lib_requirement = (des_str(slice)?, des_u16(slice)?);
            if let Some(lib_info) = registry.borrow_library_info(&lib_requirement.0) {
                if lib_requirement.1 > lib_info.version {
                    dependencies_ok = false;
                }
            } else {
                dependencies_ok = false;
            }
            additional_libs.push(lib_requirement);
        }
        if !dependencies_ok {
            // This patch will refuse to restore due to missing dependencies. We return Ok() because
            // this method should only return Err() if the patch data is *corrupt*. A dependency
            // error should be reported in a more friendly manner (during restore().)
            return Ok(Self {
                modules: Vec::new(),
                factory_lib_version,
                additional_libs,
            });
        }
        let num_modules = des_u16(slice)? as usize;
        let mod_modes = des_u2_slice(slice, num_modules)?;
        let mut modules: Vec<_> = mod_modes
            .iter()
            .map(|mode| SavedModule::deserialize(slice, *mode, &additional_libs))
            .collect::<Result<_, _>>()?;

        let mut prototypical_dummy_controls = Vec::new();
        for module in &modules {
            let mut pdss = module.get_prototypical_dummy_controls(registry)?;
            prototypical_dummy_controls.append(&mut pdss);
        }

        let mut controls = Vec::new();
        for pdc in prototypical_dummy_controls.into_iter() {
            controls.push(SavedControl::deserialize(slice, pdc)?);
        }

        if slice.len() > 0 {
            return Err(());
        }

        let mut ccp = 0;
        for module in &mut modules {
            let cc = module.get_num_controls(registry)?;
            for _ in 0..cc {
                module.controls.push(controls.get(ccp).ok_or(())?.clone());
                ccp += 1;
            }
        }

        Ok(Self {
            modules,
            factory_lib_version,
            additional_libs,
        })
    }
}

#[derive(Debug, Clone)]
enum PatchSource {
    Writable(PathBuf),
    Readable(String),
}

#[derive(Debug, Clone)]
pub struct Patch {
    source: PatchSource,
    name: String,
    note_graph: SavedModuleGraph,
    exists_on_disk: bool,
}

impl Patch {
    const FORMAT_VERSION: u8 = 1;

    pub fn new(save_path: PathBuf) -> Self {
        Self {
            name: "Unnamed".to_owned(),
            note_graph: SavedModuleGraph::blank(),
            source: PatchSource::Writable(save_path),
            exists_on_disk: false,
        }
    }

    fn load(source: PatchSource, data: &[u8], registry: &Registry) -> Result<Self, String> {
        let mut patch = Self {
            name: Default::default(),
            note_graph: SavedModuleGraph::blank(),
            source,
            exists_on_disk: true,
        };
        patch.load_from_serialized_data(data, registry)?;
        Ok(patch)
    }

    pub fn load_readable(source: String, data: &[u8], registry: &Registry) -> Result<Self, String> {
        Self::load(PatchSource::Readable(source), data, registry)
    }

    pub fn load_writable(
        source: PathBuf,
        data: &[u8],
        registry: &Registry,
    ) -> Result<Self, String> {
        Self::load(PatchSource::Writable(source), data, registry)
    }

    pub fn is_writable(&self) -> bool {
        if let PatchSource::Writable(..) = &self.source {
            true
        } else {
            false
        }
    }

    /// Returns true if the patch has been saved at all. In other words, returns true if the synth
    /// can be closed and reopened without losing the patch.
    pub fn exists_on_disk(&self) -> bool {
        self.exists_on_disk
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn borrow_name(&self) -> &str {
        &self.name
    }

    pub fn save_note_graph(&mut self, graph: &ep::ModuleGraph, registry: &Registry) {
        self.note_graph = SavedModuleGraph::save(graph, registry);
    }

    pub fn restore_note_graph(
        &self,
        graph: &mut ep::ModuleGraph,
        registry: &Registry,
    ) -> Result<(), String> {
        self.note_graph.restore(graph, registry)
    }

    pub fn write(&mut self) -> io::Result<()> {
        let path = if let PatchSource::Writable(path) = &self.source {
            path
        } else {
            panic!("Cannot write a non-writable patch!");
        };
        let file = std::fs::File::create(path)?;
        self.exists_on_disk = true;
        let mut writer = std::io::BufWriter::new(file);
        let contents = self.serialize();
        writer.write_all(contents.as_bytes())?;
        Ok(())
    }

    pub fn delete_from_disk(&mut self) -> io::Result<()> {
        let path = if let PatchSource::Writable(path) = &self.source {
            path
        } else {
            panic!("Cannot delete a non-writable patch!");
        };
        if self.exists_on_disk {
            std::fs::remove_file(path)?;
            self.exists_on_disk = false;
        }
        Ok(())
    }

    pub fn serialize(&self) -> String {
        let mut buffer = Vec::new();
        // Format version number.
        ser_u8(&mut buffer, Self::FORMAT_VERSION);
        ser_str(&mut buffer, &self.name);
        self.note_graph.serialize(&mut buffer);
        base64::encode_config(&buffer, base64::URL_SAFE_NO_PAD)
    }

    pub fn load_from_serialized_data(
        &mut self,
        data: &[u8],
        registry: &Registry,
    ) -> Result<(), String> {
        let everything = base64::decode_config(data, base64::URL_SAFE_NO_PAD)
            .map_err(|_| "ERROR: Patch data is corrupt (invalid base64 data.)")?;
        let mut ptr = &everything[..];
        let format_version = des_u8(&mut ptr)
            .map_err(|_| "ERROR: Patch data is corrupt (does not contain format version.)")?;
        if format_version > Self::FORMAT_VERSION {
            return Err("ERROR: patch was created in a newer version of Audiobench".to_owned());
        }
        self.name = des_str(&mut ptr)
            .map_err(|_| "ERROR: Patch data is corrupt (does not contain patch name.)")?;
        self.note_graph = SavedModuleGraph::deserialize(&mut ptr, format_version, registry)
            .map_err(|_| "ERROR: Patch data is corrupt".to_owned())?;
        Ok(())
    }
}
