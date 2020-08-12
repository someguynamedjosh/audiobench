use super::mini_bin::*;
use super::Registry;
use crate::engine::parts as ep;
use crate::engine::static_controls as staticons;
use crate::util::*;
use std::collections::{HashMap, HashSet};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Clone)]
struct SavedAutomationLane {
    module_index: usize,
    output_index: usize,
    range: (u16, u16),
}

impl SavedAutomationLane {
    fn save(
        lane: &ep::AutomationLane,
        control_range: (f32, f32),
        module_indexes: &HashMap<*const RefCell<ep::Module>, usize>,
    ) -> Self {
        let module_index = *module_indexes
            .get(&(&*lane.connection.0.as_ref() as *const _))
            .unwrap();
        let output_index = lane.connection.1;
        let range = (
            pack_value(lane.range.0, control_range),
            pack_value(lane.range.1, control_range),
        );
        Self {
            module_index,
            output_index,
            range,
        }
    }

    fn restore(
        &self,
        control_range: (f32, f32),
        modules: &[Rcrc<ep::Module>],
    ) -> ep::AutomationLane {
        let module = Rc::clone(&modules[self.module_index]);
        ep::AutomationLane {
            connection: (module, self.output_index),
            range: (
                unpack_value(self.range.0, control_range),
                unpack_value(self.range.1, control_range),
            ),
        }
    }

    fn get_ser_mode_u4(&self) -> u8 {
        let big_connection = self.module_index > 0b111111 || self.output_index > 0b11;
        let min_min = self.range.0 == 0x0;
        let max_max = self.range.1 == 0xFFFF;
        compose_u4(false, big_connection, min_min, max_max)
    }

    fn serialize(&self, buffer: &mut Vec<u8>, mode_u4: u8) {
        let (_, big_connection, min_min, max_max) = decompose_u4(mode_u4);
        if big_connection {
            ser_u4_u12(buffer, self.output_index as u8, self.module_index as u16);
        } else {
            ser_u2_u6(buffer, self.output_index as u8, self.module_index as u8);
        }
        if !min_min {
            ser_u16(buffer, self.range.0);
        }
        if !max_max {
            ser_u16(buffer, self.range.1);
        }
    }

    fn deserialize(slice: &mut &[u8], mode_u4: u8) -> Result<Self, ()> {
        let (_, big_connection, min_min, max_max) = decompose_u4(mode_u4);
        let (output_index, module_index) = if big_connection {
            let d = des_u4_u12(slice)?;
            (d.0 as usize, d.1 as usize)
        } else {
            let d = des_u2_u6(slice)?;
            (d.0 as usize, d.1 as usize)
        };
        let min = if min_min { 0 } else { des_u16(slice)? };
        let max = if max_max { 0xFFFF } else { des_u16(slice)? };
        Ok(Self {
            module_index,
            output_index,
            range: (min, max),
        })
    }
}

#[derive(Debug, Clone)]
struct SavedAutocon {
    // Use None to indicate the default value.
    value: Option<u16>,
    automation_lanes: Vec<SavedAutomationLane>,
}

impl SavedAutocon {
    fn save(
        autocon: &ep::Autocon,
        module_indexes: &HashMap<*const RefCell<ep::Module>, usize>,
    ) -> Self {
        let range = autocon.range;
        let value = pack_value(autocon.value, range);
        let value = if value == pack_value(autocon.default, range) || autocon.automation.len() > 0 {
            None
        } else {
            Some(value)
        };
        let automation_lanes = autocon
            .automation
            .iter()
            .map(|lane| SavedAutomationLane::save(lane, range, module_indexes))
            .collect();
        Self {
            value,
            automation_lanes,
        }
    }

    fn restore(&self, on: &mut ep::Autocon, modules: &[Rcrc<ep::Module>]) {
        if let Some(value) = self.value {
            let value = unpack_value(value, on.range);
            on.value = value;
        } else {
            on.value = on.default;
        }
        let range = on.range;
        on.automation = self
            .automation_lanes
            .iter()
            .map(|lane| lane.restore(range, modules))
            .collect();
    }

    fn get_ser_mode_u2(&self) -> u8 {
        if self.value.is_none() && self.automation_lanes.len() == 0 {
            0
        } else if self.automation_lanes.len() == 0 {
            1
        } else if self.automation_lanes.len() == 1 {
            2
        } else {
            3
        }
    }

    fn serialize_mode1(&self, buffer: &mut Vec<u8>) {
        ser_u16(buffer, self.value.unwrap());
    }

    fn serialize_mode3_u4(&self) -> u8 {
        debug_assert!(
            self.automation_lanes.len() >= 2 && self.automation_lanes.len() - 2 <= 0b1111
        );
        (self.automation_lanes.len() - 2) as u8
    }

    fn deserialize_mode012(slice: &mut &[u8], mode: u8) -> Result<(Self, usize), ()> {
        Ok(if mode == 0 {
            (
                Self {
                    value: None,
                    automation_lanes: Vec::new(),
                },
                0,
            )
        } else if mode == 1 {
            (
                Self {
                    value: Some(des_u16(slice)?),
                    automation_lanes: Vec::new(),
                },
                0,
            )
        } else if mode == 2 {
            (
                Self {
                    value: None,
                    automation_lanes: Vec::new(),
                },
                1,
            )
        } else {
            unreachable!()
        })
    }

    fn deserialize_mode3_u4(u4: u8) -> (Self, usize) {
        (
            Self {
                value: None,
                automation_lanes: Vec::new(),
            },
            u4 as usize + 2,
        )
    }
}

#[derive(Debug, Clone)]
struct SavedStaticon {
    value: Vec<u8>,
}

impl SavedStaticon {
    fn save(staticon: &staticons::Staticon) -> Self {
        let mut value = Vec::new();
        staticon.borrow_data().serialize(&mut value);
        Self { value }
    }

    fn restore(&self, on: &mut staticons::Staticon) {
        let mut data_slice = &self.value[..];
        // Our own deserialize() method should have returned an error if the data's deserialization
        // method would cause an error.
        on.borrow_data_mut().deserialize(&mut data_slice).unwrap();
    }

    fn serialize(&self, buffer: &mut Vec<u8>) {
        buffer.append(&mut self.value.clone());
    }

    /// prototypical_dummy_control is a sacrificial value that should be a clone of the actual
    /// control this SavedStaticon is storing data for. It will be used to determine how much data
    /// should be saved from the slice to deserialize in the *actual* control during `restore`.
    fn deserialize(
        slice: &mut &[u8],
        prototypical_dummy_control: staticons::Staticon,
    ) -> Result<Self, ()> {
        let old_slice = *slice;
        // Basically have the dummy control deserialize the data and then just store the section
        // of data it deserialized.
        prototypical_dummy_control
            .borrow_data_mut()
            .deserialize(slice)?;
        let data_len = old_slice.len() - slice.len();
        let value = Vec::from(&old_slice[..data_len]);
        Ok(Self { value })
    }
}

#[derive(Debug, Clone)]
enum SavedInputConnection {
    DefaultDefault,
    Default(usize),
    Output {
        module_index: usize,
        output_index: usize,
    },
}

impl SavedInputConnection {
    fn save(
        connection: &ep::InputConnection,
        default: usize,
        module_indexes: &HashMap<*const RefCell<ep::Module>, usize>,
    ) -> Self {
        match connection {
            ep::InputConnection::Default(index) => {
                if *index == default {
                    Self::DefaultDefault
                } else {
                    Self::Default(*index)
                }
            }
            ep::InputConnection::Wire(module, output_index) => {
                let module_index = *module_indexes
                    .get(&(&*module.as_ref() as *const _))
                    .unwrap();
                SavedInputConnection::Output {
                    module_index,
                    output_index: *output_index,
                }
            }
        }
    }

    fn restore(&self, default: usize, modules: &[Rcrc<ep::Module>]) -> ep::InputConnection {
        match self {
            Self::DefaultDefault => ep::InputConnection::Default(default),
            Self::Default(index) => ep::InputConnection::Default(*index),
            Self::Output {
                module_index,
                output_index,
            } => ep::InputConnection::Wire(Rc::clone(&modules[*module_index]), *output_index),
        }
    }

    fn get_ser_mode_u2(&self) -> u8 {
        match self {
            Self::DefaultDefault => 0,
            Self::Default(..) => 1,
            Self::Output {
                module_index,
                output_index,
            } => {
                if *module_index <= 0b111111 && *output_index <= 0b11 {
                    2
                } else {
                    debug_assert!(*module_index <= 0xFFF && *output_index < 0xF);
                    3
                }
            }
        }
    }

    fn serialize_mode1_u4(&self) -> u8 {
        if let Self::Default(idx) = self {
            debug_assert!(*idx <= 0b1111);
            *idx as u8
        } else {
            unreachable!()
        }
    }

    fn serialize_mode23(&self, buffer: &mut Vec<u8>, mode: u8) {
        if let Self::Output {
            module_index,
            output_index,
        } = self
        {
            if mode == 2 {
                ser_u2_u6(buffer, *output_index as u8, *module_index as u8);
            } else if mode == 3 {
                ser_u4_u12(buffer, *output_index as u8, *module_index as u16);
            } else {
                unreachable!()
            }
        } else {
            unreachable!()
        }
    }

    fn deserialize_mode1_u4(u4: u8) -> Self {
        Self::Default(u4 as usize)
    }

    fn deserialize_mode023(slice: &mut &[u8], mode: u8) -> Result<Self, ()> {
        if mode == 0 {
            return Ok(Self::DefaultDefault);
        }
        let (output_index, module_index) = if mode == 2 {
            let d = des_u2_u6(slice)?;
            (d.0 as usize, d.1 as usize)
        } else if mode == 3 {
            let d = des_u4_u12(slice)?;
            (d.0 as usize, d.1 as usize)
        } else {
            unreachable!()
        };
        Ok(Self::Output {
            module_index,
            output_index,
        })
    }
}

#[derive(Debug, Clone)]
struct SavedModule {
    lib_name: String,
    template_id: usize,
    autocons: Vec<SavedAutocon>,
    staticons: Vec<SavedStaticon>,
    input_connections: Vec<SavedInputConnection>,
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
        let autocons = module
            .autocons
            .iter()
            .map(|c| SavedAutocon::save(&*c.borrow(), module_indexes))
            .collect();
        let staticons = module
            .staticons
            .iter()
            .map(|cc| SavedStaticon::save(&*cc.borrow()))
            .collect();
        let mut input_connections = Vec::new();
        for (index, connection) in module.inputs.iter().enumerate() {
            let default = template_ref.default_inputs[index];
            input_connections.push(SavedInputConnection::save(
                connection,
                default,
                module_indexes,
            ));
        }
        let pos = module.pos;
        let pos = (pos.0 as i32, pos.1 as i32);
        drop(template_ref);
        Self {
            lib_name,
            template_id,
            autocons,
            staticons,
            input_connections,
            pos,
        }
    }

    fn restore(&self, registry: &Registry) -> Result<ep::Module, ()> {
        let mut m = self.lookup_prototype(registry)?.clone();
        m.pos = (self.pos.0 as f32, self.pos.1 as f32);
        for index in 0..self.staticons.len() {
            self.staticons[index].restore(&mut *m.staticons[index].borrow_mut());
        }
        Ok(m)
    }

    fn restore_connections(
        &self,
        on: &mut ep::Module,
        modules: &[Rcrc<ep::Module>],
    ) -> Result<(), ()> {
        let template_ref = on.template.borrow();
        for index in 0..self.input_connections.len() {
            let default = template_ref.default_inputs[index];
            on.inputs[index] = self.input_connections[index].restore(default, modules);
        }
        for index in 0..self.autocons.len() {
            let mut control_ref = on.autocons[index].borrow_mut();
            self.autocons[index].restore(&mut *control_ref, modules);
        }
        Ok(())
    }

    fn lookup_prototype<'a>(&self, registry: &'a Registry) -> Result<&'a ep::Module, ()> {
        registry
            .borrow_module_by_serialized_id(&(self.lib_name.clone(), self.template_id))
            .ok_or(())
    }

    /// Gets the number of inputs, autocons, and staticons this module should have when
    /// fully restored.
    fn get_requirements(&self, registry: &Registry) -> Result<(usize, usize, usize), ()> {
        let p = self.lookup_prototype(registry)?;
        let num_inputs = p.inputs.len();
        let num_autocons = p.autocons.len();
        let num_staticons = p.staticons.len();
        Ok((num_inputs, num_autocons, num_staticons))
    }

    fn get_prototypical_dummy_staticons(
        &self,
        registry: &Registry,
    ) -> Result<Vec<staticons::Staticon>, ()> {
        let p = self.lookup_prototype(registry)?;
        Ok(p.staticons.iter().map(|s| s.borrow().clone()).collect())
    }

    // The mode will indicate how data about this module is stored. Different modes take up
    // different amount of space, so this helps save space.
    fn get_ser_mode_u2(&self) -> u8 {
        let small_coords = self.pos.0.abs() < 0x7FFF && self.pos.1.abs() < 0x7FFF;
        let small_resource = self.lib_name == "factory" && self.template_id < 0xFF;
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
            let lib_id = if self.lib_name == "factory" {
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
            "factory".to_owned()
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
            autocons: Vec::new(),
            staticons: Vec::new(),
            input_connections: Vec::new(),
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
        library_names.remove("factory");
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
            self.modules[index]
                .restore_connections(&mut *module.borrow_mut(), &modules[..])
                .map_err(|_| {
                    format!("ERROR: Patch data is corrupt (failed to restore connections.)")
                })?;
        }
        graph.set_modules(modules);
        Ok(())
    }

    fn serialize(&self, buffer: &mut Vec<u8>) {
        // FE not FF because it doesn't include factory.
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

        let mut input_modes = Vec::new();
        for module in &self.modules {
            for input in &module.input_connections {
                input_modes.push(input.get_ser_mode_u2());
            }
        }
        ser_u2_slice(buffer, &input_modes[..]);
        for module in &self.modules {
            for input in &module.input_connections {
                let mode = input.get_ser_mode_u2();
                if mode == 2 || mode == 3 {
                    input.serialize_mode23(buffer, mode);
                }
            }
        }
        let mut ser_u4 = Vec::new();
        for module in &self.modules {
            for input in &module.input_connections {
                if input.get_ser_mode_u2() == 1 {
                    ser_u4.push(input.serialize_mode1_u4());
                }
            }
        }
        ser_u4_slice(buffer, &ser_u4[..]);

        let mut control_modes = Vec::new();
        for module in &self.modules {
            for control in &module.autocons {
                control_modes.push(control.get_ser_mode_u2());
            }
        }
        ser_u2_slice(buffer, &control_modes[..]);
        for module in &self.modules {
            for control in &module.autocons {
                let mode = control.get_ser_mode_u2();
                if mode == 1 {
                    control.serialize_mode1(buffer);
                }
            }
        }
        let mut control_mode3_u4 = Vec::new();
        for module in &self.modules {
            for control in &module.autocons {
                let mode = control.get_ser_mode_u2();
                if mode == 3 {
                    control_mode3_u4.push(control.serialize_mode3_u4());
                }
            }
        }
        ser_u4_slice(buffer, &control_mode3_u4[..]);

        for module in &self.modules {
            for staticon in &module.staticons {
                staticon.serialize(buffer);
            }
        }

        let mut lane_modes = Vec::new();
        for module in &self.modules {
            for control in &module.autocons {
                for lane in &control.automation_lanes {
                    lane_modes.push(lane.get_ser_mode_u4());
                }
            }
        }
        ser_u4_slice(buffer, &lane_modes[..]);
        for module in &self.modules {
            for control in &module.autocons {
                for lane in &control.automation_lanes {
                    let mode = lane.get_ser_mode_u4();
                    lane.serialize(buffer, mode);
                }
            }
        }
    }

    fn deserialize(slice: &mut &[u8], format_version: u8, registry: &Registry) -> Result<Self, ()> {
        let num_libs = des_u8(slice)?;
        // After version 1, the version number of the factory library is included in the patch.
        let factory_lib_version = if format_version >= 1 {
            des_u16(slice)?
        } else {
            // Before format version 1, the factory library was always at version 0.
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

        let (mut num_inputs, mut num_autocons) = (0, 0);
        let mut prototypical_dummy_staticons = Vec::new();
        for module in &modules {
            let (i, a, _) = module.get_requirements(registry)?;
            num_inputs += i;
            num_autocons += a;
            let mut pdss = module.get_prototypical_dummy_staticons(registry)?;
            prototypical_dummy_staticons.append(&mut pdss);
        }

        let input_modes = des_u2_slice(slice, num_inputs)?;
        let mut inputs = vec![None; num_inputs];
        let mut num_mode1_inputs = 0;
        for (index, mode) in input_modes.iter().cloned().enumerate() {
            if mode == 0 || mode == 2 || mode == 3 {
                inputs[index] = Some(SavedInputConnection::deserialize_mode023(slice, mode)?);
            } else {
                num_mode1_inputs += 1;
            }
        }
        let u4_data = des_u4_slice(slice, num_mode1_inputs)?;
        let mut data_index = 0;
        for (index, mode) in input_modes.iter().cloned().enumerate() {
            if mode == 1 {
                inputs[index] = Some(SavedInputConnection::deserialize_mode1_u4(
                    u4_data[data_index],
                ));
                data_index += 1;
            }
        }

        let control_modes = des_u2_slice(slice, num_autocons)?;
        let mut autocons = vec![None; num_autocons];
        let mut num_mode3_autocons = 0;
        let mut num_lanes = 0;
        for (index, mode) in control_modes.iter().cloned().enumerate() {
            if mode == 3 {
                num_mode3_autocons += 1;
            } else {
                let (control, lanes) = SavedAutocon::deserialize_mode012(slice, mode)?;
                num_lanes += lanes;
                autocons[index] = Some((control, lanes));
            }
        }
        let u4_data = des_u4_slice(slice, num_mode3_autocons)?;
        let mut data_index = 0;
        for (index, mode) in control_modes.iter().cloned().enumerate() {
            if mode == 3 {
                let (control, lanes) =
                    SavedAutocon::deserialize_mode3_u4(*u4_data.get(data_index).ok_or(())?);
                num_lanes += lanes;
                autocons[index] = Some((control, lanes));
                data_index += 1;
            }
        }

        let mut staticons = Vec::new();
        for pdc in prototypical_dummy_staticons.into_iter() {
            staticons.push(SavedStaticon::deserialize(slice, pdc)?);
        }
        let lane_modes = des_u4_slice(slice, num_lanes)?;
        let mut lanes = Vec::new();
        for mode in lane_modes.into_iter() {
            lanes.push(SavedAutomationLane::deserialize(slice, mode)?);
        }

        if slice.len() > 0 {
            return Err(());
        }

        let (mut ip, mut cp, mut ccp, mut lp) = (0, 0, 0, 0);
        for module in &mut modules {
            let (i, c, cc) = module.get_requirements(registry)?;
            for _ in 0..i {
                module
                    .input_connections
                    .push(inputs.get(ip).ok_or(())?.clone().unwrap());
                ip += 1;
            }
            for _ in 0..c {
                let (mut autocon, num_lanes) = autocons.get(cp).ok_or(())?.clone().unwrap();
                cp += 1;
                for _ in 0..num_lanes {
                    autocon
                        .automation_lanes
                        .push(lanes.get(lp).ok_or(())?.clone());
                    lp += 1;
                }
                module.autocons.push(autocon);
            }
            for _ in 0..cc {
                module.staticons.push(staticons.get(ccp).ok_or(())?.clone());
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
