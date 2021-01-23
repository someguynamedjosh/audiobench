use const_env::from_env;

#[from_env("CARGO_PKG_VERSION_MINOR")]
pub const ENGINE_VERSION: u16 = 0xFFFF;
pub const ENGINE_INFO: &'static str = concat!(
    "Audiobench is free and open source software. You are free to do anything you want with it, ",
    "including selling any audio, patches, or modules you make with or for it. If you make ",
    "modifications to the source code you must make those changes freely available under the GNU ",
    "General Public License, Version 3. Source code is available at ",
    "https://gitlab.com/Code_Cube/audio-bench."
);
#[cfg(debug_assertions)]
pub const ENGINE_UPDATE_URL: &'static str = "http://localhost:8000/latest.json";
#[cfg(not(debug_assertions))]
pub const ENGINE_UPDATE_URL: &'static str = "https://bit.ly/adb_update_check";
