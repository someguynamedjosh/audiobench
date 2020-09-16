use std::fs;
use std::io::{Read, Write};
use std::path::Path;

fn main() {
    // Can't use env! because it isn't defined when the build script is first compiled.
    let output_path = Path::new(&std::env::var("OUT_DIR").unwrap()).join("factory.ablib");
    let output_file = fs::File::create(output_path).unwrap();
    let mut zip_writer = zip::ZipWriter::new(output_file);
    let options =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let input_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("factory_library");
    println!("cargo:rerun-if-changed={:?}", input_path.as_os_str());
    // https://github.com/mvdnes/zip-rs/blob/master/examples/write_dir.rs
    for entry in walkdir::WalkDir::new(input_path.clone()).into_iter() {
        let entry = entry.unwrap();
        let path = entry.path();
        let zip_key = path.strip_prefix(input_path.clone()).unwrap();
        if path.is_file() {
            zip_writer
                .start_file_from_path(&zip_key, options.clone())
                .unwrap();
            let mut f = fs::File::open(path).unwrap();
            if zip_key == Path::new("library_info.yaml") {
                let engine_version: i32 = std::env::var("CARGO_PKG_VERSION_MINOR")
                    .unwrap()
                    .parse()
                    .unwrap();
                let mut file_contents = String::new();
                f.read_to_string(&mut file_contents).unwrap();
                file_contents =
                    file_contents.replace("$ENGINE_VERSION", &format!("{}", engine_version));
                zip_writer.write_all(file_contents.as_bytes()).unwrap();
            } else {
                std::io::copy(&mut f, &mut zip_writer).unwrap();
            }
        } else if zip_key.as_os_str().len() > 0 {
            zip_writer
                .add_directory_from_path(&zip_key, options.clone())
                .unwrap();
        }
    }
    zip_writer.finish().unwrap();
}
