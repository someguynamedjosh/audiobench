use super::module_template::ModuleTemplate;
use super::Registry;
use crate::config::*;
use crate::engine::controls::{AnyControl, Control};
use crate::engine::parts as ep;
use shared_util::{
    mini_serde::{MiniDes, MiniSer},
    prelude::*,
};
use std::io::{self, Write};
use std::path::PathBuf;
use std::{
    collections::{HashMap, HashSet},
    unimplemented,
};

#[derive(Debug, Clone)]
struct SavedModuleGraph;

impl SavedModuleGraph {
    pub fn blank() -> Self {
        Self
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
        // self.note_graph = SavedModuleGraph::save(graph, registry);
        unimplemented!()
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
        // Format version number.
        ser.u8(Self::FORMAT_VERSION);
        ser.str(&self.name);
        unimplemented!();
        // self.note_graph.serialize(&mut buffer);
        base64::encode_config(&ser.finish(), base64::URL_SAFE_NO_PAD)
    }

    pub fn load_from_serialized_data(
        &mut self,
        data: &[u8],
        registry: &Registry,
    ) -> Result<(), String> {
        let everything = base64::decode_config(data, base64::URL_SAFE_NO_PAD)
            .map_err(|_| "ERROR: Patch data is corrupt (invalid base64 data.)")?;
        let mut des = MiniDes::start(everything);
        let format_version = des
            .u8()
            .map_err(|_| "ERROR: Patch data is corrupt (does not contain format version.)")?;
        if format_version > Self::FORMAT_VERSION {
            return Err("ERROR: patch was created in a newer version of Audiobench".to_owned());
        }
        self.name = des
            .str()
            .map_err(|_| "ERROR: Patch data is corrupt (does not contain patch name.)")?;
        unimplemented!();
        // self.note_graph = SavedModuleGraph::deserialize(&mut ptr, format_version, registry)
        //     .map_err(|_| "ERROR: Patch data is corrupt".to_owned())?;
        Ok(())
    }
}
