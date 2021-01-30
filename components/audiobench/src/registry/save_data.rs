use crate::{
    engine::{controls::Control, parts as ep},
    registry::Registry,
};
use shared_util::{
    mini_serde::{MiniDes, MiniSer},
    prelude::*,
};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Clone)]
enum PatchSource {
    Writable(PathBuf),
    Readable(String),
}

#[derive(Debug, Clone)]
pub struct Patch {
    source: PatchSource,
    name: String,
    exists_on_disk: bool,
    data: Vec<u8>,
}

impl Patch {
    const FORMAT_VERSION: u8 = 2;

    pub fn new(save_path: PathBuf) -> Self {
        Self {
            name: "Unnamed".to_owned(),
            source: PatchSource::Writable(save_path),
            exists_on_disk: false,
            data: Vec::new(),
        }
    }

    fn load(source: PatchSource, data: &[u8], registry: &Registry) -> Result<Self, String> {
        let mut patch = Self {
            name: Default::default(),
            source,
            exists_on_disk: true,
            data: Vec::new(),
        };
        patch.deserialize(data)?;
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
        let mut ser = MiniSer::new();
        let mut ordered_lib_names = Vec::new();
        let lib_data: Vec<_> = registry.borrow_library_infos().collect();
        assert!(lib_data.len() < 0x100);
        ser.u8((lib_data.len() - 1) as _);
        for (lib_name, lib_info) in lib_data {
            if lib_name == "User" {
                continue;
            }
            ordered_lib_names.push(lib_name.clone());
            ser.str(lib_name);
            ser.version(lib_info.version);
        }
        let lib_index = |name: &String| {
            ordered_lib_names
                .iter()
                .position(|other| other == name)
                .unwrap() as u8
        };

        let ordered_modules = Vec::from(graph.borrow_modules());
        assert!(ordered_modules.len() < 0x100);
        let mod_index = |rc: &Rc<_>| {
            ordered_modules
                .iter()
                .position(|other| Rc::ptr_eq(rc, other))
                .unwrap() as u8
        };
        for module in graph.borrow_modules() {
            let module = module.borrow();
            let template = module.template.borrow();
            ser.u8(lib_index(&template.lib_name));
            ser.u8(template.save_id as _);
            ser.i32(module.pos.0 as _);
            ser.i32(module.pos.1 as _);
            for control in &module.controls {
                let control_ptr = control.as_dyn_ptr();
                let control = control_ptr.borrow();
                for source in control.get_connected_automation() {
                    ser.bool(true);
                    ser.u8(mod_index(&source.module));
                    ser.u4(source.output_index as _);
                }
                ser.bool(false);
                control.serialize(&mut ser);
            }
        }
        println!("{}", ser.debug_content);
        self.data = ser.finish();
    }

    pub fn restore_note_graph(
        &self,
        graph: &mut ep::ModuleGraph,
        registry: &Registry,
    ) -> Result<(), String> {
        unimplemented!()
        // self.note_graph.restore(graph, registry)
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
        let mut ser = MiniSer::new();
        ser.u8(Self::FORMAT_VERSION);
        ser.str(&self.name);
        ser.blob(&self.data[..]);
        println!("{}", ser.debug_content);
        let data = ser.finish();
        println!("{:?}", data);
        base64::encode_config(&data, base64::URL_SAFE_NO_PAD)
    }

    pub fn deserialize(&mut self, data: &[u8]) -> Result<(), String> {
        let data = base64::decode_config(data, base64::URL_SAFE_NO_PAD)
            .map_err(|_| "ERROR: Patch data is corrupt (invalid base64 data.)")?;
        let mut des = MiniDes::start(data);
        let format_version = des
            .u8()
            .map_err(|_| "ERROR: Patch data is corrupt (does not contain format version.)")?;
        if format_version > Self::FORMAT_VERSION {
            return Err("ERROR: Patch was created in a newer version of Audiobench".to_owned());
        } else if format_version == 1 {
            return Err(concat!(
                "ERROR: Patch was created in an older version of Audiobench ",
                "which is no longer supported"
            )
            .to_owned());
        }
        self.name = des
            .str()
            .map_err(|_| "ERROR: Patch data is corrupt (does not contain patch name.)")?;
        println!("{}", self.name);
        self.data = des.end();
        Ok(())
    }
}
