use super::save_data::Patch;
use super::yaml;
use crate::engine::parts::Module;
use crate::util::*;
use rand::RngCore;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};

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
}

impl Registry {
    fn load_module_resource(
        &mut self,
        name: &str,
        lib_name: String,
        module_id: String,
        buffer: Vec<u8>,
    ) -> Result<(), String> {
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
            module_id,
            &yaml,
        )?;
        let index = self.modules.len();
        let template_ref = module.template.borrow();
        let ser_id = (template_ref.lib_name.clone(), template_ref.template_id);
        drop(template_ref);
        self.modules.push(module);
        self.modules_by_resource_id.insert(resource_id, index);
        if self.modules_by_serialized_id.contains_key(&ser_id) {
            return Err(format!(
                "ERROR: Multiple modules have {} as their save id",
                ser_id.1
            ));
        }
        self.modules_by_serialized_id.insert(ser_id, index);
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
        let mut reader = std::io::Cursor::new(buffer);
        let patch = if let Some(full_path) = full_path {
            Patch::load_writable(full_path, &mut reader, &self)
        } else {
            Patch::load_readable(name.to_owned(), &mut reader, &self)
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
            let module_id = Self::strip_path_and_extension(file_name, ".module.yaml");
            self.load_module_resource(
                &full_name,
                lib_name.to_owned(),
                module_id.to_owned(),
                buffer,
            )?;
        } else if file_name.ends_with(".ns") {
            self.load_script_resource(&full_name, buffer)?;
        } else if file_name.ends_with(".abpatch") {
            self.unloaded_patches.push((full_name, full_path, buffer));
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
        let factory_library = std::include_bytes!(concat!(env!("OUT_DIR"), "/factory.ablib"));
        let reader = std::io::Cursor::new(factory_library as &[u8]);
        self.load_zipped_library("factory", reader)
            .map_err(|e| format!("ERROR: Failed to load factory library, caused by:\n{}", e))?;

        let user_library_path = self.library_path.join("user");
        fs::create_dir_all(&user_library_path).map_err(|err| {
            format!(
                "ERROR: Failed to create user library at {}, caused by:\n{}",
                user_library_path.to_string_lossy(),
                err
            )
        })?;
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
            if entry.path().is_dir() {
                #[rustfmt::skip]
                let name = entry.path().file_name().unwrap().to_string_lossy().into_owned();
                if loaded_libraries.contains(&name) {
                    return Err(format!(
                        "ERROR: You have installed multiple libraries named {}",
                        name
                    ));
                }
                loaded_libraries.insert(name);
                self.load_library_from_folder(&entry.path())?;
            } else if entry.path().extension() == Some(std::ffi::OsStr::new("ablib")) {
                #[rustfmt::skip]
                let name = entry.path().file_name().unwrap().to_string_lossy().into_owned();
                let name = String::from(&name[..name.len() - 6]);
                if loaded_libraries.contains(&name) {
                    return Err(format!(
                        "ERROR: You have installed multiple libraries named {}",
                        name
                    ));
                }
                loaded_libraries.insert(name);
                self.load_library_from_file(&entry.path())?;
            } else {
                return Err(format!(
                    "ERROR: The following library does not have the right extension (.ablib):\n{}",
                    entry.path().to_string_lossy()
                ));
            }
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
        };
        let result = registry.initialize();

        (registry, result)
    }

    pub fn borrow_modules(&self) -> &[Module] {
        &self.modules
    }

    pub fn borrow_module_by_resource_id(&self, id: &str) -> Option<&Module> {
        self.modules_by_resource_id
            .get(id)
            .map(|idx| &self.modules[*idx])
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
        let patch = Patch::writable(self.library_path.join("user").join(filename));
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
}
