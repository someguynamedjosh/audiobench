use super::yaml;
use crate::config::*;
use std::fs::{self, File};
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};

pub struct LibraryInfo {
    pub pretty_name: String,
    pub description: String,
    pub version: u16,
    // min_engine_version check is handled by parse_library_info.
}

pub(super) struct PreloadedLibrary {
    pub internal_name: String,
    pub content: Box<dyn LibraryContentProvider>,
    pub info: LibraryInfo,
}

pub(super) trait LibraryContentProvider {
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
pub(super) struct ZippedLibraryContentProvider<R: Read + Seek> {
    archive: zip::ZipArchive<R>,
    non_directory_files: Vec<usize>,
}

impl<R: Read + Seek> ZippedLibraryContentProvider<R> {
    pub(super) fn new(reader: R) -> Result<Self, String> {
        let mut archive = zip::ZipArchive::new(reader).map_err(|e| {
            format!(
                "ERROR: File is not a valid ZIP archive, caused by:\nERROR: {}",
                e
            )
        })?;
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

fn parse_library_info(name: &str, buffer: Vec<u8>) -> Result<LibraryInfo, String> {
    assert!(
        ENGINE_VERSION < 0xFFFF,
        "ERROR: Engine version not provided during compilation."
    );
    let buffer_as_text = String::from_utf8(buffer).map_err(|e| {
        format!(
            "ERROR: Not a valid UTF-8 text document, caused by:\nERROR: {}",
            e
        )
    })?;
    let yaml = yaml::parse_yaml(&buffer_as_text, name)?;
    let pretty_name = yaml.unique_child("pretty_name")?.value.clone();
    let description = yaml.unique_child("description")?.value.clone();
    let version = yaml
        .unique_child("version")?
        .parse_ranged(Some(0), Some(0xFFFF))?;
    let min_engine_version = yaml
        .unique_child("min_engine_version")?
        .parse_ranged(Some(0), Some(0xFFFF))?;
    if min_engine_version > ENGINE_VERSION as i32 {
        return Err(format!(
            concat!(
                "ERROR: This library requires at least version {} of Audiobench.\n",
                "You are currently running version {}.",
            ),
            min_engine_version, ENGINE_VERSION
        ));
    }
    Ok(LibraryInfo {
        pretty_name,
        description,
        version,
    })
}

pub(super) fn preload_library(
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
            let lib_info = parse_library_info(&lib_info_name, buffer).map_err(|err| {
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

pub(super) fn preload_library_from_path(path: &Path) -> Result<PreloadedLibrary, String> {
    let lib_name: String = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into();
    if path.is_dir() {
        let content = DirectoryLibraryContentProvider::new(path.to_owned())?;
        preload_library(lib_name, Box::new(content))
    } else {
        let extension_index = lib_name.rfind(".").unwrap_or(lib_name.len());
        if &lib_name[extension_index..] != ".ablib" {
            return Err(format!(
                concat!("ERROR: The file has an invalid extension \"{}\" (should be .ablib)"),
                &lib_name[extension_index..]
            ));
        }
        let lib_name = (&lib_name[..extension_index]).to_owned();
        let file = File::open(path)
            .map_err(|e| format!("ERROR: Failed to open file, caused by:\nERROR: {}", e))?;
        let content = ZippedLibraryContentProvider::new(file)?;
        preload_library(lib_name, Box::new(content))
    }
}
