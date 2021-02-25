use const_env::from_env;
use shared_util::Version;

#[from_env("CARGO_PKG_VERSION_MAJOR")]
const ENGINE_VERSION_MAJ: u8 = 0;
#[from_env("CARGO_PKG_VERSION_MINOR")]
const ENGINE_VERSION_MIN: u8 = 0;
#[from_env("CARGO_PKG_VERSION_PATCH")]
const ENGINE_VERSION_PATCH: u8 = 0;
pub const ENGINE_VERSION: Version =
    unsafe { Version::new_unchecked(ENGINE_VERSION_MAJ, ENGINE_VERSION_MIN, ENGINE_VERSION_PATCH) };

pub const ENGINE_INFO: &'static str = concat!(
    "Audiobench is free and open source software. You are free to do anything you want with it, ",
    "including selling any audio, patches, or modules you make with or for it. If you make ",
    "modifications to the source code you must make those changes freely available under the GNU ",
    "General Public License, Version 3. Source code is available at ",
    "https://github.com/joshua-maros/audiobench."
);
#[cfg(debug_assertions)]
pub const ENGINE_UPDATE_URL: &'static str = "http://localhost:8000/latest.json";
#[cfg(not(debug_assertions))]
pub const ENGINE_UPDATE_URL: &'static str = "https://bit.ly/adb_update_check";
