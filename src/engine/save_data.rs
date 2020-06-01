use crate::engine::parts as ep;
use crate::engine::registry::Registry;
use crate::util::*;
use std::collections::{HashMap, HashSet};
use std::io::{self, Read, Write};
use std::path::PathBuf;

#[inline]
fn compose_u2(bit1: bool, bit0: bool) -> u8 {
    (if bit1 { 0b10 } else { 0b00 }) | (if bit0 { 0b01 } else { 0b00 })
}

#[inline]
fn decompose_u2(u2: u8) -> (bool, bool) {
    (u2 & 0b10 != 0, u2 & 0b01 != 0)
}

#[inline]
fn compose_u4(bit3: bool, bit2: bool, bit1: bool, bit0: bool) -> u8 {
    (if bit3 { 0b1000 } else { 0b0000 })
        | (if bit2 { 0b0100 } else { 0b0000 })
        | (if bit1 { 0b0010 } else { 0b0000 })
        | (if bit0 { 0b0001 } else { 0b0000 })
}

#[inline]
fn decompose_u4(u4: u8) -> (bool, bool, bool, bool) {
    (
        u4 & 0b1000 != 0,
        u4 & 0b0100 != 0,
        u4 & 0b0010 != 0,
        u4 & 0b0001 != 0,
    )
}

#[inline]
fn ser_str(buffer: &mut Vec<u8>, text: &str) {
    assert!(text.len() < std::u16::MAX as usize);
    ser_u16(buffer, text.len() as u16);
    buffer.reserve(text.len());
    for b in text.bytes() {
        buffer.push(b);
    }
}

#[inline]
fn ser_u2_slice(buffer: &mut Vec<u8>, value: &[u8]) {
    if value.len() == 0 {
        return;
    }
    let mut packed = vec![0; (value.len() + 3) / 4];
    for (index, value) in value.iter().cloned().enumerate() {
        debug_assert!(value <= 0b11);
        packed[index / 4] |= value << (index % 4 * 2);
    }
    buffer.append(&mut packed);
}

#[inline]
fn ser_u4_slice(buffer: &mut Vec<u8>, value: &[u8]) {
    if value.len() == 0 {
        return;
    }
    let mut packed = vec![0; (value.len() + 1) / 2];
    for (index, value) in value.iter().cloned().enumerate() {
        debug_assert!(value <= 0b1111);
        packed[index / 2] |= value << (index % 2 * 4);
    }
    buffer.append(&mut packed);
}

#[inline]
fn ser_u2_u6(buffer: &mut Vec<u8>, u2: u8, u6: u8) {
    debug_assert!(u2 <= 0b11);
    debug_assert!(u6 <= 0b111111);
    buffer.push((u2 << 6) | u6);
}

#[inline]
fn ser_u4_u12(buffer: &mut Vec<u8>, u4: u8, u12: u16) {
    debug_assert!(u4 <= 0xF);
    debug_assert!(u12 <= 0xFFF);
    ser_u16(buffer, ((u4 as u16) << 12) | u12);
}

#[inline]
fn ser_u8(buffer: &mut Vec<u8>, value: u8) {
    buffer.push(value);
}

#[inline]
fn ser_i16(buffer: &mut Vec<u8>, value: i16) {
    for b in &value.to_be_bytes() {
        buffer.push(*b);
    }
}

#[inline]
fn ser_u16(buffer: &mut Vec<u8>, value: u16) {
    for b in &value.to_be_bytes() {
        buffer.push(*b);
    }
}

#[inline]
fn ser_i32(buffer: &mut Vec<u8>, value: i32) {
    for b in &value.to_be_bytes() {
        buffer.push(*b);
    }
}

#[inline]
fn advance_des(slice: &mut &[u8], amount: usize) {
    *slice = &slice[amount..];
}

#[inline]
fn des_str(slice: &mut &[u8]) -> Result<String, ()> {
    let len = des_u16(slice)? as usize;
    if slice.len() < len {
        return Err(());
    }
    let buffer = Vec::from(&slice[..len]);
    advance_des(slice, len);
    let string = String::from_utf8(buffer).map_err(|_| ())?;
    Ok(string)
}

#[inline]
fn des_u2_slice(slice: &mut &[u8], num_items: usize) -> Result<Vec<u8>, ()> {
    if slice.len() < (num_items + 3) / 4 {
        return Err(());
    }
    let mut res = Vec::with_capacity(num_items);
    if num_items == 0 {
        return Ok(res);
    }
    for index in 0..num_items {
        res.push((slice[index / 4] >> (index % 4 * 2)) & 0b11);
    }
    advance_des(slice, (num_items + 3) / 4);
    Ok(res)
}

#[inline]
fn des_u4_slice(slice: &mut &[u8], num_items: usize) -> Result<Vec<u8>, ()> {
    if slice.len() < (num_items + 1) / 2 {
        return Err(());
    }
    let mut res = Vec::with_capacity(num_items);
    if num_items == 0 {
        return Ok(res);
    }
    for index in 0..num_items {
        res.push((slice[index / 2] >> (index % 2 * 4)) & 0b1111);
    }
    advance_des(slice, (num_items + 1) / 2);
    Ok(res)
}

#[inline]
fn des_u2_u6(slice: &mut &[u8]) -> Result<(u8, u8), ()> {
    let packed = des_u8(slice)?;
    Ok((packed >> 6, packed & 0b111111))
}

#[inline]
fn des_u4_u12(slice: &mut &[u8]) -> Result<(u16, u16), ()> {
    let packed = des_u16(slice)?;
    Ok((packed >> 12, packed & 0xFFF))
}

#[inline]
fn des_u8(slice: &mut &[u8]) -> Result<u8, ()> {
    if slice.len() < 1 {
        return Err(());
    }
    let res = u8::from_be_bytes([slice[0]]);
    advance_des(slice, 1);
    Ok(res)
}

#[inline]
fn des_i16(slice: &mut &[u8]) -> Result<i16, ()> {
    if slice.len() < 2 {
        return Err(());
    }
    let res = i16::from_be_bytes([slice[0], slice[1]]);
    advance_des(slice, 2);
    Ok(res)
}

#[inline]
fn des_u16(slice: &mut &[u8]) -> Result<u16, ()> {
    if slice.len() < 2 {
        return Err(());
    }
    let res = u16::from_be_bytes([slice[0], slice[1]]);
    advance_des(slice, 2);
    Ok(res)
}

#[inline]
fn des_i32(slice: &mut &[u8]) -> Result<i32, ()> {
    if slice.len() < 4 {
        return Err(());
    }
    let res = i32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]);
    advance_des(slice, 4);
    Ok(res)
}

#[inline]
fn pack_value(value: f32, range: (f32, f32)) -> u16 {
    let value = value.from_range(range.0, range.1);
    // Value is now between 0 and 1.
    let value = value.to_range(0.0, 0x10000 as f32).min(0xFFFF as f32);
    // Value is now between 0 and 0xFFFF
    value as u16
}

#[inline]
fn unpack_value(value: u16, range: (f32, f32)) -> f32 {
    let value = value as f32;
    // Value is now between 0 and 0xFFFF
    let value = value.from_range(0.0, 0xFFFF as f32);
    // Value is now between 0 and 1.
    value.to_range(range.0, range.1)
}

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
struct SavedControl {
    // Use None to indicate the default value.
    value: Option<u16>,
    automation_lanes: Vec<SavedAutomationLane>,
}

impl SavedControl {
    fn save(
        control: &ep::Control,
        module_indexes: &HashMap<*const RefCell<ep::Module>, usize>,
    ) -> Self {
        let range = control.range;
        let value = pack_value(control.value, range);
        let value = if value == pack_value(control.default, range) || control.automation.len() > 0 {
            None
        } else {
            Some(value)
        };
        let automation_lanes = control
            .automation
            .iter()
            .map(|lane| SavedAutomationLane::save(lane, range, module_indexes))
            .collect();
        Self {
            value,
            automation_lanes,
        }
    }

    fn restore(&self, on: &mut ep::Control, modules: &[Rcrc<ep::Module>]) {
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
struct SavedComplexControl {
    value: String,
}

impl SavedComplexControl {
    fn save(ccontrol: &ep::ComplexControl) -> Self {
        Self {
            value: ccontrol.value.clone(),
        }
    }

    fn restore(&self, on: &mut ep::ComplexControl) {
        on.value = self.value.clone();
    }

    // No fancy modes. Complex controls are just always inefficient, not much we can do about them.
    fn serialize(&self, buffer: &mut Vec<u8>) {
        ser_str(buffer, &self.value);
    }

    fn deserialize(slice: &mut &[u8]) -> Result<Self, ()> {
        Ok(Self {
            value: des_str(slice)?,
        })
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
    controls: Vec<SavedControl>,
    complex_controls: Vec<SavedComplexControl>,
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
        let controls = module
            .controls
            .iter()
            .map(|c| SavedControl::save(&*c.borrow(), module_indexes))
            .collect();
        let complex_controls = module
            .complex_controls
            .iter()
            .map(|cc| SavedComplexControl::save(&*cc.borrow()))
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
            controls,
            complex_controls,
            input_connections,
            pos,
        }
    }

    fn restore(&self, registry: &Registry) -> Result<ep::Module, ()> {
        let mut m = self.lookup_prototype(registry)?.clone();
        m.pos = (self.pos.0 as f32, self.pos.1 as f32);
        for index in 0..self.complex_controls.len() {
            self.complex_controls[index].restore(&mut *m.complex_controls[index].borrow_mut());
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
        for index in 0..self.controls.len() {
            let mut control_ref = on.controls[index].borrow_mut();
            self.controls[index].restore(&mut *control_ref, modules);
        }
        Ok(())
    }

    fn lookup_prototype<'a>(&self, registry: &'a Registry) -> Result<&'a ep::Module, ()> {
        registry
            .borrow_module_by_serialized_id(&(self.lib_name.clone(), self.template_id))
            .ok_or(())
    }

    /// Gets the number of inputs, controls, and complex controls this module should have when
    /// fully restored.
    fn get_requirements(&self, registry: &Registry) -> Result<(usize, usize, usize), ()> {
        let p = self.lookup_prototype(registry)?;
        let num_inputs = p.inputs.len();
        let num_controls = p.controls.len();
        let num_ccontrols = p.complex_controls.len();
        Ok((num_inputs, num_controls, num_ccontrols))
    }

    // The mode will indicate how data about this module is stored. Different modes take up
    // different amount of space, so this helps save space.
    fn get_ser_mode_u2(&self) -> u8 {
        let small_coords = self.pos.0.abs() < 0x7FFF && self.pos.1.abs() < 0x7FFF;
        let small_resource = self.lib_name == "base" && self.template_id < 0xFF;
        compose_u2(small_coords, small_resource)
    }

    fn serialize(&self, buffer: &mut Vec<u8>, mode_u2: u8, ordered_lib_names: &[String]) {
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
            let lib_id = if self.lib_name == "base" {
                0
            } else {
                ordered_lib_names
                    .iter()
                    .position(|i| i == &self.lib_name)
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
        ordered_lib_names: &[String],
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
            "base".to_owned()
        } else {
            ordered_lib_names[lib_index as usize - 1].clone()
        };
        Ok(Self {
            lib_name,
            template_id: template_index,
            controls: Vec::new(),
            complex_controls: Vec::new(),
            input_connections: Vec::new(),
            pos: (x, y),
        })
    }
}

#[derive(Debug, Clone)]
struct SavedModuleGraph {
    modules: Vec<SavedModule>,
}

impl SavedModuleGraph {
    fn blank() -> Self {
        Self {
            modules: Default::default(),
        }
    }

    fn save(graph: &ep::ModuleGraph) -> Self {
        let mut module_indexes: HashMap<*const RefCell<ep::Module>, usize> = HashMap::new();
        for (index, module) in graph.borrow_modules().iter().enumerate() {
            module_indexes.insert(&*module.as_ref(), index);
        }
        let module_indexes = module_indexes;
        let modules = graph
            .borrow_modules()
            .imc(|module| SavedModule::save(&*module.borrow(), &module_indexes));
        Self { modules }
    }

    fn restore(&self, graph: &mut ep::ModuleGraph, registry: &Registry) -> Result<(), ()> {
        let modules: Vec<_> = self
            .modules
            .iter()
            .map(|m| m.restore(registry).map(|m| rcrc(m)))
            .collect::<Result<_, _>>()?;
        for index in 0..self.modules.len() {
            let module = Rc::clone(&modules[index]);
            self.modules[index].restore_connections(&mut *module.borrow_mut(), &modules[..])?;
        }
        graph.set_modules(modules);
        Ok(())
    }

    fn serialize(&self, buffer: &mut Vec<u8>) {
        let mut lib_names: HashSet<_> = self
            .modules
            .iter()
            .map(|module| module.lib_name.clone())
            .collect();
        lib_names.remove("base"); // Base is always lib #0.
        let ordered_lib_names: Vec<_> = lib_names.into_iter().collect();
        // FE not FF because it doesn't include base.
        assert!(ordered_lib_names.len() < 0xFE);
        ser_u8(buffer, ordered_lib_names.len() as u8);
        for name in &ordered_lib_names {
            ser_str(buffer, name);
        }
        // The biggest indexes (for connections) are 12 bits long.
        assert!(self.modules.len() <= 0xFFF);
        ser_u16(buffer, self.modules.len() as u16);
        let mod_modes: Vec<_> = self.modules.imc(|module| module.get_ser_mode_u2());
        ser_u2_slice(buffer, &mod_modes[..]);
        for index in 0..self.modules.len() {
            self.modules[index].serialize(buffer, mod_modes[index], &ordered_lib_names);
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
            for control in &module.controls {
                control_modes.push(control.get_ser_mode_u2());
            }
        }
        ser_u2_slice(buffer, &control_modes[..]);
        for module in &self.modules {
            for control in &module.controls {
                let mode = control.get_ser_mode_u2();
                if mode == 1 {
                    control.serialize_mode1(buffer);
                }
            }
        }
        let mut control_mode3_u4 = Vec::new();
        for module in &self.modules {
            for control in &module.controls {
                let mode = control.get_ser_mode_u2();
                if mode == 3 {
                    control_mode3_u4.push(control.serialize_mode3_u4());
                }
            }
        }
        ser_u4_slice(buffer, &control_mode3_u4[..]);

        for module in &self.modules {
            for ccontrol in &module.complex_controls {
                ccontrol.serialize(buffer);
            }
        }

        let mut lane_modes = Vec::new();
        for module in &self.modules {
            for control in &module.controls {
                for lane in &control.automation_lanes {
                    lane_modes.push(lane.get_ser_mode_u4());
                }
            }
        }
        ser_u4_slice(buffer, &lane_modes[..]);
        for module in &self.modules {
            for control in &module.controls {
                for lane in &control.automation_lanes {
                    let mode = lane.get_ser_mode_u4();
                    lane.serialize(buffer, mode);
                }
            }
        }
    }

    fn deserialize(slice: &mut &[u8], registry: &Registry) -> Result<Self, ()> {
        let num_libs = des_u8(slice)?;
        let mut ordered_lib_names = Vec::new();
        for _ in 0..num_libs {
            ordered_lib_names.push(des_str(slice)?);
        }
        let num_modules = des_u16(slice)? as usize;
        let mod_modes = des_u2_slice(slice, num_modules)?;
        let mut modules: Vec<_> = mod_modes
            .iter()
            .map(|mode| SavedModule::deserialize(slice, *mode, &ordered_lib_names))
            .collect::<Result<_, _>>()?;

        let (mut num_inputs, mut num_controls, mut num_ccontrols) = (0, 0, 0);
        for module in &modules {
            let (i, c, cc) = module.get_requirements(registry)?;
            num_inputs += i;
            num_controls += c;
            num_ccontrols += cc;
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

        let control_modes = des_u2_slice(slice, num_controls)?;
        let mut controls = vec![None; num_controls];
        let mut num_mode3_controls = 0;
        let mut num_lanes = 0;
        for (index, mode) in control_modes.iter().cloned().enumerate() {
            if mode == 3 {
                num_mode3_controls += 1;
            } else {
                let (control, lanes) = SavedControl::deserialize_mode012(slice, mode)?;
                num_lanes += lanes;
                controls[index] = Some((control, lanes));
            }
        }
        let u4_data = des_u4_slice(slice, num_mode3_controls)?;
        let mut data_index = 0;
        for (index, mode) in control_modes.iter().cloned().enumerate() {
            if mode == 3 {
                let (control, lanes) = SavedControl::deserialize_mode3_u4(u4_data[data_index]);
                num_lanes += lanes;
                controls[index] = Some((control, lanes));
                data_index += 1;
            }
        }

        let mut complex_controls = Vec::new();
        for _ in 0..num_ccontrols {
            complex_controls.push(SavedComplexControl::deserialize(slice)?);
        }
        let lane_modes = des_u4_slice(slice, num_lanes)?;
        let mut lanes = Vec::new();
        for mode in lane_modes.into_iter() {
            lanes.push(SavedAutomationLane::deserialize(slice, mode)?);
        }

        let (mut ip, mut cp, mut ccp, mut lp) = (0, 0, 0, 0);
        for module in &mut modules {
            let (i, c, cc) = module.get_requirements(registry)?;
            for _ in 0..i {
                module.input_connections.push(inputs[ip].clone().unwrap());
                ip += 1;
            }
            for _ in 0..c {
                let (mut control, num_lanes) = controls[cp].clone().unwrap();
                cp += 1;
                for _ in 0..num_lanes {
                    control.automation_lanes.push(lanes[lp].clone());
                    lp += 1;
                }
                module.controls.push(control);
            }
            for _ in 0..cc {
                module.complex_controls.push(complex_controls[ccp].clone());
                ccp += 1;
            }
        }

        Ok(Self { modules })
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
}

impl Patch {
    pub fn writable(save_path: PathBuf) -> Self {
        Self {
            name: "Unnamed".to_owned(),
            note_graph: SavedModuleGraph::blank(),
            source: PatchSource::Writable(save_path),
        }
    }

    pub fn load_readable(
        source: String,
        reader: &mut impl Read,
        registry: &Registry,
    ) -> Result<Self, String> {
        Self::deserialize(PatchSource::Readable(source), reader, registry)
    }

    pub fn load_writable(
        source: PathBuf,
        reader: &mut impl Read,
        registry: &Registry,
    ) -> Result<Self, String> {
        Self::deserialize(PatchSource::Writable(source), reader, registry)
    }

    pub fn is_writable(&self) -> bool {
        if let PatchSource::Writable(..) = &self.source {
            true
        } else {
            false
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn borrow_name(&self) -> &str {
        &self.name
    }

    pub fn save_note_graph(&mut self, graph: &ep::ModuleGraph) {
        self.note_graph = SavedModuleGraph::save(graph);
    }

    pub fn restore_note_graph(
        &self,
        graph: &mut ep::ModuleGraph,
        registry: &Registry,
    ) -> Result<(), String> {
        self.note_graph
            .restore(graph, registry)
            .map_err(|_| "ERROR: Patch data is corrupt".to_owned())
    }

    pub fn write(&self) -> io::Result<()> {
        let path = if let PatchSource::Writable(path) = &self.source {
            path
        } else {
            panic!("Cannot write a non-writable patch!");
        };
        let file = std::fs::File::create(path)?;
        let mut writer = std::io::BufWriter::new(file);
        self.serialize(&mut writer)?;
        Ok(())
    }

    fn serialize(&self, writer: &mut impl Write) -> io::Result<()> {
        let mut buffer = Vec::new();
        // Format version number.
        ser_u8(&mut buffer, 0);
        ser_str(&mut buffer, &self.name);
        self.note_graph.serialize(&mut buffer);
        let encoded = base64::encode_config(&buffer, base64::URL_SAFE_NO_PAD);
        write!(writer, "{}", encoded)?;
        Ok(())
    }

    fn deserialize(
        source: PatchSource,
        reader: &mut impl Read,
        registry: &Registry,
    ) -> Result<Self, String> {
        let mut everything = Vec::new();
        reader
            .read_to_end(&mut everything)
            .map_err(|_| "ERROR: Failed to read patch data".to_owned())?;

        let err_map = |_| "ERROR: Patch data is corrupt".to_owned();
        let everything =
            base64::decode_config(&everything, base64::URL_SAFE_NO_PAD).map_err(err_map)?;

        let err_map = |_| "ERROR: Patch data is corrupt".to_owned();
        let mut ptr = &everything[..];
        if des_u8(&mut ptr).map_err(err_map)? > 0 {
            return Err("ERROR: patch was created in a newer version of Audiobench".to_owned());
        }

        let err_map = |_| "ERROR: Patch data is corrupt".to_owned();
        let name = des_str(&mut ptr).map_err(err_map)?;

        let note_graph = SavedModuleGraph::deserialize(&mut ptr, registry)
            .map_err(|_| "ERROR: Patch data is corrupt".to_owned())?;
        Ok(Self {
            source,
            name,
            note_graph,
        })
    }
}
