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

pub struct LibraryInfo {
    pretty_name: String,
    description: String,
    version: u16,
}

struct PreloadedLibrary {
    internal_name: String,
    content: Box<dyn LibraryContentProvider>,
    info: LibraryInfo,
}

trait LibraryContentProvider {
    fn get_num_files(&self) -> usize;
    fn get_file_name(&mut self, index: usize) -> String;
    fn get_full_path(&mut self, index: usize) -> Option<PathBuf>;
    fn read_file_contents(&mut self, index: usize) -> Result<Vec<u8>, String>;
}

// Allows loading a library from a plain directory.
struct DirectoryLibraryContentProvider {
    root_path: PathBuf,
    file_paths: Vec<PathBuf>,
}

impl DirectoryLibraryContentProvider {
    fn new(root_path: PathBuf) -> Result<Self, String> {
        let mut file_paths = Vec::new();
        let mut unvisited_paths = vec![PathBuf::new()];
        while let Some(visiting) = unvisited_paths.pop() {
            let reader_path = root_path.join(&visiting);
            let reader = fs::read_dir(&reader_path).map_err(|err| {
                format!(
                    "ERROR: Failed to list files in {}, caused by:\nERROR: {}",
                    reader_path.to_string_lossy(),
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
        Ok(Self {
            root_path,
            file_paths,
        })
    }
}

impl LibraryContentProvider for DirectoryLibraryContentProvider {
    fn get_num_files(&self) -> usize {
        self.file_paths.len()
    }

    fn get_file_name(&mut self, index: usize) -> String {
        self.file_paths[index].to_string_lossy().into()
    }

    fn get_full_path(&mut self, index: usize) -> Option<PathBuf> {
        Some(self.root_path.join(&self.file_paths[index]))
    }

    fn read_file_contents(&mut self, index: usize) -> Result<Vec<u8>, String> {
        fs::read(self.root_path.join(&self.file_paths[index]))
            .map_err(|err| format!("ERROR: {}", err))
    }
}

// Allows loading a library from a zip file.
struct ZippedLibraryContentProvider<R: Read + Seek> {
    archive: zip::ZipArchive<R>,
    non_directory_files: Vec<usize>,
}

impl<R: Read + Seek> ZippedLibraryContentProvider<R> {
    fn new(reader: R) -> Result<Self, String> {
        let mut archive = zip::ZipArchive::new(reader).map_err(|e| format!("ERROR: {}", e))?;
        let non_directory_files = (0..archive.len())
            .filter(|element| !archive.by_index(*element).unwrap().name().ends_with("/"))
            .collect();
        Ok(Self {
            archive,
            non_directory_files,
        })
    }
}

impl<R: Read + Seek> LibraryContentProvider for ZippedLibraryContentProvider<R> {
    fn get_num_files(&self) -> usize {
        self.non_directory_files.len()
    }

    fn get_file_name(&mut self, index: usize) -> String {
        self.archive
            .by_index(self.non_directory_files[index])
            .unwrap()
            .name()
            .to_owned()
    }

    fn get_full_path(&mut self, _index: usize) -> Option<PathBuf> {
        None
    }

    fn read_file_contents(&mut self, index: usize) -> Result<Vec<u8>, String> {
        let mut file = self
            .archive
            .by_index(self.non_directory_files[index])
            .unwrap();
        let mut buffer = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut buffer).map_err(|err| {
            format!(
                "ERROR: Failed to read zipped file {}, caused by:\nERROR: {}",
                file.name(),
                err
            )
        })?;
        Ok(buffer)
    }
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
        if self.modules_by_serialized_id.contains_key(&ser_id) {
            let mut save_ids = HashSet::new();
            for (this_lib_name, save_id) in self.modules_by_serialized_id.keys() {
                if this_lib_name == &lib_name {
                    save_ids.insert(*save_id);
                }
            }
            let mut next_available_id = 0;
            while save_ids.contains(&next_available_id) {
                next_available_id += 1;
            }
            return Err(format!(
                "ERROR: Multiple modules have {} as their save id. The lowest available ID is {}.",
                ser_id.1, next_available_id
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
        } else if file_name == "library_info.yaml" {
            // Handled in library preload phase.
        } else {
            return Err(format!(
                "ERROR: Not sure what to do with the file {}.",
                full_name
            ));
        }
        Ok(())
    }

    fn load_library(
        &mut self,
        lib_name: &str,
        file_provider: &mut impl LibraryContentProvider,
    ) -> Result<(), String> {
        // Load icons before other data.
        for index in 0..file_provider.get_num_files() {
            let file_name = file_provider.get_file_name(index);
            if file_name.ends_with(".icon.svg") {
                let full_path = file_provider.get_full_path(index);
                let contents = file_provider.read_file_contents(index)?;
                self.load_resource(lib_name, &file_name, full_path, contents)?;
            }
        }
        for index in 0..file_provider.get_num_files() {
            let file_name = file_provider.get_file_name(index);
            if !file_name.ends_with(".icon.svg") {
                let full_path = file_provider.get_full_path(index);
                let contents = file_provider.read_file_contents(index)?;
                self.load_resource(lib_name, &file_name, full_path, contents)?;
            }
        }
        Ok(())
    }

    fn load_zipped_library(
        &mut self,
        lib_name: &str,
        lib_reader: impl Read + Seek,
    ) -> Result<(), String> {
        let mut provider = ZippedLibraryContentProvider::new(lib_reader)?;
        self.load_library(lib_name, &mut provider)
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
        let mut provider = DirectoryLibraryContentProvider::new(path.into())?;
        let lib_name: String = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into();
        self.load_library(&lib_name, &mut provider)
    }

    fn load_library_new(&mut self, mut library: PreloadedLibrary) -> Result<(), String> {
        // Load icons before other data.
        for index in 0..library.content.get_num_files() {
            let file_name = library.content.get_file_name(index);
            if file_name.ends_with(".icon.svg") {
                let full_path = library.content.get_full_path(index);
                let contents = library.content.read_file_contents(index)?;
                self.load_resource(&library.internal_name, &file_name, full_path, contents)?;
            }
        }
        for index in 0..library.content.get_num_files() {
            let file_name = library.content.get_file_name(index);
            if !file_name.ends_with(".icon.svg") {
                let full_path = library.content.get_full_path(index);
                let contents = library.content.read_file_contents(index)?;
                self.load_resource(&library.internal_name, &file_name, full_path, contents)?;
            }
        }
        Ok(())
    }

    fn parse_library_info(name: &str, buffer: Vec<u8>) -> Result<LibraryInfo, String> {
        let buffer_as_text = String::from_utf8(buffer).map_err(|e| {
            format!(
                "ERROR: Not a valid UTF-8 text document, caused by:\nERROR: {}",
                e
            )
        })?;
        let yaml = yaml::parse_yaml(&buffer_as_text, name)?;
        let pretty_name = yaml.unique_child("pretty_name")?.value.clone();
        let description = yaml.unique_child("description")?.value.clone();
        let version = yaml.unique_child("version")?.i32()?;
        if version < 0 || version > 0xFFFF {
            return Err(format!(
                "ERROR: The version number {} is invalid, it must be between 0 and {}.",
                version, 0xFFFF
            ));
        }
        let version = version as u16;
        Ok(LibraryInfo {
            pretty_name,
            description,
            version,
        })
    }

    fn preload_library(
        lib_name: String,
        mut content: Box<dyn LibraryContentProvider>,
    ) -> Result<PreloadedLibrary, String> {
        for index in 0..content.get_num_files() {
            if &content.get_file_name(index) == "library_info.yaml" {
                let lib_info_name = format!("{}:{}", lib_name, "library_info.yaml");
                let buffer = content.read_file_contents(index).map_err(|err| {
                    format!(
                        "ERROR: Failed to read file {}, caused by:\n{}",
                        &lib_info_name, err
                    )
                })?;
                let lib_info = Self::parse_library_info(&lib_info_name, buffer).map_err(|err| {
                    format!(
                        "ERROR: Failed to parse {}, caused by:\n{}",
                        &lib_info_name, err
                    )
                })?;
                return Ok(PreloadedLibrary {
                    internal_name: lib_name,
                    info: lib_info,
                    content,
                });
            }
        }
        Err(format!(
            "ERROR: Library does not have a library_info.yaml file"
        ))
    }

    fn initialize(&mut self) -> Result<(), String> {
        let factory_library = {
            let raw = std::include_bytes!(concat!(env!("OUT_DIR"), "/factory.ablib"));
            let reader = std::io::Cursor::new(raw as &[u8]);
            let content = ZippedLibraryContentProvider::new(reader).map_err(|err| {
                format!(
                    "ERROR: Failed to open factory library, caused by:\n{}",
                    err
                )
            })?;
            Self::preload_library("factory".to_owned(), Box::new(content)).map_err(|err| {
                format!(
                    "ERROR: Failed to preload factory library, caused by:\n{}",
                    err
                )
            })?
        };
        self.load_library_new(factory_library)
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
