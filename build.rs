use std::fs;
use std::path;
use std::process::Command;
use std::io::{Read, Write};

fn expect_success(command: &mut Command, error_message: &'static str) {
    let status = command.status().expect(error_message);
    if !status.success() {
        panic!(error_message);
    }
}

fn build_projucer_linux() {
    // Check that essential commands exist.
    expect_success(
        Command::new("git").arg("--version"),
        "Git does not appear to be installed.",
    );
    expect_success(
        Command::new("make").arg("--version"),
        "Make does not appear to be installed.",
    );

    // Clone the source repo.
    if path::Path::new("juce").exists() {
        fs::remove_dir_all("juce").expect("Failed to cleanup existing JUCE source code.");
    }
    expect_success(
        Command::new("git").args(&["clone", "https://github.com/juce-framework/JUCE", "juce"]),
        "Failed to clone https://github.com/juce-framework/JUCE.",
    );

    // Activate GPL mode.
    let config_path = "juce/extras/Projucer/JuceLibraryCode/AppConfig.h";
    let mut file =
        fs::File::open(config_path).expect(&format!("Failed to open {} for reading", config_path));
    // This isn't very efficient but it doesn't need to be.
    let mut content = "".to_owned();
    file.read_to_string(&mut content)
        .expect(&format!("Failed to read from {}", config_path));
    // If this errors, the format of AppConfig.h has probably changed and this build script should
    // be changed accordingly.
    content
        .find("JUCER_ENABLE_GPL_MODE 0")
        .expect("Could not find GPL flag in the Jucer config file.");
    let content = content.replace("JUCER_ENABLE_GPL_MODE 0", "JUCER_ENABLE_GPL_MODE 1");
    let mut file = fs::File::create(config_path)
        .expect(&format!("Failed to open {} for writing", config_path));
    write!(file, "{}", content).expect("Failed to write to the config file.");
    drop(file);

    // Build projucer with make
    let build_dir = "juce/extras/Projucer/Builds/LinuxMakefile/";
    // If this fails, try running it manually and see if you are missing any packages.
    expect_success(
        Command::new("make").current_dir(build_dir), 
        "Failed to run 'make' in the folder juce/extras/Projucer/Builds/LinuxMakefile/"
    );
        
    let success_indicator = fs::File::create("juce/success_indicator.txt").expect("Failed to create juce/success_indicator.txt");
}

fn setup_projucer() {
    if path::Path::new("juce/success_indicator.txt").exists() {
        // Projucer is already built.
        return;
    }
    if cfg!(target_os = "linux") {
        build_projucer_linux();
    } else {
        panic!("Compilation is not currently supported for this platform.")
    }
}

fn main() {
}
