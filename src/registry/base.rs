use super::library_preload::{self, PreloadedLibrary, ZippedLibraryContentProvider};
use super::save_data::Patch;
use super::update_check::{self, UpdateInfo};
use super::yaml;
use crate::engine::parts::Module;
use crate::util::*;
use rand::RngCore;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};

pub use super::library_preload::LibraryInfo;

pub struct Registry {
    modules: Vec<Module>,
    modules_by_resource_id: HashMap<String, usize>,
    modules_by_serialized_id: HashMap<(String, usize), usize>,

    scripts: HashMap<String, String>,

    icon_indexes: HashMap<String, usize>,
    icons: Vec<Vec<u8>>,

    unloaded_patches: Vec<(String, Option<PathBuf>, Vec<u8>)>,
    patches: Vec<Rcrc<Patch>>,
    patch_paths: HashMap<String, usize>,

    library_path: PathBuf,
    library_info: HashMap<String, LibraryInfo>,
    checked_updates: HashMap<String, Option<UpdateInfo>>,
    update_check_stream: Receiver<(String, Option<UpdateInfo>)>,
}

enum DelayedError {
    DuplicateSaveId(usize),
}

impl Registry {
    fn load_module_resource(
        &mut self,
        name: &str,
        lib_name: String,
        module_id: String,
        buffer: Vec<u8>,
    ) -> Result<Option<DelayedError>, String> {
        let buffer_as_text = String::from_utf8(buffer).map_err(|e| {
            format!(
                "ERROR: The file {} is not a valid UTF-8 text document, caused by:\nERROR: {}",
                name, e
            )
        })?;
        let yaml = yaml::parse_yaml(&buffer_as_text, name)?;
        let resource_id = format!("{}:{}", lib_name, module_id);
        let module = super::module_template::create_module_prototype_from_yaml(
            &self.icon_indexes,
            lib_name.clone(),
            module_id.clone(),
            &yaml,
        )
        .map_err(|err| {
            format!(
                "ERROR: Failed to load module {}, caused by:\n{}",
                module_id, err
            )
        })?;
        let index = self.modules.len();
        let template_ref = module.template.borrow();
        let ser_id = (template_ref.lib_name.clone(), template_ref.template_id);
        drop(template_ref);
        self.modules.push(module);
        self.modules_by_resource_id.insert(resource_id, index);
        let delayed_error = if self.modules_by_serialized_id.contains_key(&ser_id) {
            Some(DelayedError::DuplicateSaveId(ser_id.1))
        } else {
            None
        };
        self.modules_by_serialized_id.insert(ser_id, index);
        Ok(delayed_error)
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
        let patch = if let Some(full_path) = full_path {
            Patch::load_writable(full_path, &buffer[..], &self)
        } else {
            Patch::load_readable(name.to_owned(), &buffer[..], &self)
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
    ) -> Result<Option<DelayedError>, String> {
        let full_name = format!("{}:{}", lib_name, file_name);
        if file_name.ends_with(".icon.svg") {
            let file_name = Self::strip_path_and_extension(file_name, ".icon.svg");
            let icon_id = format!("{}:{}", lib_name, file_name);
            self.icon_indexes.insert(icon_id, self.icons.len());
            self.icons.push(buffer);
        } else if file_name.ends_with(".module.yaml") {
            let module_id = Self::strip_path_and_extension(file_name, ".module.yaml");
            return self.load_module_resource(
                &full_name,
                lib_name.to_owned(),
                module_id.to_owned(),
                buffer,
            );
        } else if file_name.ends_with(".ns") {
            self.load_script_resource(&full_name, buffer)?;
        } else if file_name.ends_with(".abpatch") {
            self.unloaded_patches.push((full_name, full_path, buffer));
        } else if file_name.ends_with(".md") {
            // Ignore, probably just readme / license type stuff.
        } else if file_name == "library_info.yaml" {
            // Handled in library preload phase.
        } else {
            return Err(format!(
                "ERROR: Not sure what to do with the file {}.",
                full_name
            ));
        }
        Ok(None)
    }

    fn load_library(&mut self, mut library: PreloadedLibrary) -> Result<LibraryInfo, String> {
        // Load icons before other data.
        for index in 0..library.content.get_num_files() {
            let file_name = library.content.get_file_name(index);
            if file_name.ends_with(".icon.svg") {
                let full_path = library.content.get_full_path(index);
                let contents = library.content.read_file_contents(index)?;
                let delayed_error =
                    self.load_resource(&library.internal_name, &file_name, full_path, contents)?;
                assert!(
                    delayed_error.is_none(),
                    "Icons should not cause delayed errors."
                );
            }
        }
        let mut delayed_error = None;
        for index in 0..library.content.get_num_files() {
            let file_name = library.content.get_file_name(index);
            if !file_name.ends_with(".icon.svg") {
                let full_path = library.content.get_full_path(index);
                let contents = library.content.read_file_contents(index)?;
                delayed_error = delayed_error.or(self.load_resource(
                    &library.internal_name,
                    &file_name,
                    full_path,
                    contents,
                )?);
            }
        }
        if let Some(DelayedError::DuplicateSaveId(dupl_id)) = delayed_error {
            let mut save_ids = HashSet::new();
            for (this_lib_name, save_id) in self.modules_by_serialized_id.keys() {
                if this_lib_name == &library.internal_name {
                    save_ids.insert(*save_id);
                }
            }
            let mut next_available_id = 0;
            while save_ids.contains(&next_available_id) {
                next_available_id += 1;
            }
            return Err(format!(
                "ERROR: Multiple modules have {} as their save id. The lowest available ID is {}.",
                dupl_id, next_available_id
            ));
        }
        Ok(library.info)
    }

    fn create_and_update_user_library(&self) -> Result<(), String> {
        let user_library_path = self.library_path.join("user");
        fs::create_dir_all(&user_library_path).map_err(|err| {
            format!(
                "ERROR: Failed to create user library at {}, caused by:\n{}",
                user_library_path.to_string_lossy(),
                err
            )
        })?;
        let library_info = include_str!("user_library_info.yaml");
        fs::write(
            user_library_path.join("library_info.yaml"),
            &library_info.replace("$ENGINE_VERSION", &format!("{}", ENGINE_VERSION)),
        )
        .map_err(|err| {
            format!(
                "ERROR: Failed to create library_info.yaml for user library, caused by:\nERROR:{}",
                err
            )
        })?;
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), String> {
        let factory_library = {
            let raw = std::include_bytes!(concat!(env!("OUT_DIR"), "/factory.ablib"));
            let reader = std::io::Cursor::new(raw as &[u8]);
            let content = ZippedLibraryContentProvider::new(reader).map_err(|err| {
                format!("ERROR: Failed to open factory library, caused by:\n{}", err)
            })?;
            library_preload::preload_library("factory".to_owned(), Box::new(content)).map_err(
                |err| {
                    format!(
                        "ERROR: Failed to preload factory library, caused by:\n{}",
                        err
                    )
                },
            )?
        };
        let factory_lib_info = self
            .load_library(factory_library)
            .map_err(|e| format!("ERROR: Failed to load factory library, caused by:\n{}", e))?;
        self.library_info
            .insert("factory".to_owned(), factory_lib_info);

        self.create_and_update_user_library()?;

        let mut loaded_libraries = HashSet::new();
        loaded_libraries.insert("factory".to_owned());
        for entry in fs::read_dir(&self.library_path).map_err(|err| {
            format!(
                "ERROR: Failed to read libraries from {}, caused by:\n{}",
                self.library_path.to_string_lossy(),
                err
            )
        })? {
            let entry = if let Ok(entry) = entry {
                entry
            } else {
                continue;
            };
            let library =
                library_preload::preload_library_from_path(&entry.path()).map_err(|err| {
                    format!(
                        "ERROR: Failed to preload library from {}, caused by:\n{}",
                        entry.path().to_string_lossy(),
                        err
                    )
                })?;
            let internal_name = library.internal_name.clone();
            let info = self.load_library(library).map_err(|err| {
                format!(
                    "ERROR: Failed to load library {}, caused by:\n{}",
                    internal_name, err
                )
            })?;
            self.library_info.insert(internal_name, info);
        }

        // We wait to load patches in case patches depend on libraries that aren't loaded yet when
        // the library they are a part of is being loaded.
        let unloaded_patches = std::mem::take(&mut self.unloaded_patches);
        for (name, path, data) in unloaded_patches.into_iter() {
            self.load_patch(&name, path, data)?;
        }

        Ok(())
    }

    pub fn new() -> (Self, Result<(), String>) {
        let library_path = {
            let user_dirs = directories::UserDirs::new().unwrap();
            let document_dir = user_dirs.document_dir().unwrap();
            document_dir.join("Audiobench")
        };

        let (sender, receiver) = mpsc::channel();
        let update_urls = vec![ENGINE_UPDATE_URL.to_owned()];
        update_check::spawn_update_checker(update_urls, sender);

        let mut registry = Self {
            modules: Vec::new(),
            modules_by_resource_id: HashMap::new(),
            modules_by_serialized_id: HashMap::new(),
            scripts: HashMap::new(),
            icon_indexes: HashMap::new(),
            icons: Vec::new(),
            unloaded_patches: Vec::new(),
            patches: Vec::new(),
            patch_paths: HashMap::new(),
            library_path,
            library_info: HashMap::new(),
            checked_updates: HashMap::new(),
            update_check_stream: receiver,
        };
        let result = registry.initialize();

        (registry, result)
    }

    pub fn borrow_modules(&self) -> &[Module] {
        &self.modules
    }

    pub fn borrow_module_by_serialized_id(&self, id: &(String, usize)) -> Option<&Module> {
        self.modules_by_serialized_id
            .get(id)
            .map(|idx| &self.modules[*idx])
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
        self.patch_paths
            .insert(format!("user:{}", filename), self.patches.len());
        let patch = Patch::new(self.library_path.join("user").join(filename));
        let prc = rcrc(patch);
        self.patches.push(prc);
        self.patches.last().unwrap()
    }

    pub fn get_patch_by_name(&self, name: &str) -> Option<&Rcrc<Patch>> {
        self.patch_paths.get(name).map(|i| &self.patches[*i])
    }

    pub fn borrow_patches(&self) -> &Vec<Rcrc<Patch>> {
        &self.patches
    }

    pub fn borrow_library_info(&self, name: &str) -> Option<&LibraryInfo> {
        self.library_info.get(name)
    }

    pub fn borrow_library_infos(&self) -> impl Iterator<Item = &LibraryInfo> {
        self.library_info.values()
    }

    // Returns true if the update checker is still running.
    pub fn poll_update_checker(&mut self) -> bool {
        loop {
            let res = self.update_check_stream.try_recv();
            match res {
                Ok((url, result)) => {
                    self.checked_updates.insert(url, result);
                }
                Err(TryRecvError::Disconnected) => return false,
                Err(TryRecvError::Empty) => return true,
            }
        }
    }

    pub fn borrow_update_info(&self, url: &str) -> Option<&Option<UpdateInfo>> {
        self.checked_updates.get(url)
    }

    pub fn any_updates_available(&self) -> bool {
        if let Some(Some(info)) = self.checked_updates.get(ENGINE_UPDATE_URL) {
            if info.version > ENGINE_VERSION {
                return true;
            }
        }
        false
    }
}
