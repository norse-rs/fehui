use std::path::Path;
use std::fs;
use crate::Error;

#[cfg(windows)]
use libloading::os::windows::Symbol;
#[cfg(unix)]
use libloading::os::unix::Symbol;

#[cfg(target_os = "windows")]
pub fn dylib_name(name: &str) -> String {
    format!("{}.dll", name)
}
#[cfg(target_os = "macos")]
pub fn dylib_name(name: &str) -> String {
    format!("lib{}.dylib", name)
}
#[cfg(all(target_family = "unix", not(target_os = "macos")))]
pub fn dylib_name(name: &str) -> String {
    format!("lib{}.so", name)
}

pub struct Entry {
    _library: libloading::Library,
    api_load: Symbol<extern "C" fn(ui: *mut fehui::FehUI)>,
    _dir: tempfile::TempDir, // WARNING: this should be dropped last
}

impl Entry {
    pub fn new<P: AsRef<Path>>(lib: P) -> Result<Self, Error> {
        let dir = tempfile::tempdir()?;
        let lib_name = lib.as_ref().file_name().unwrap();
        let tmp_lib = dir.path().join(lib_name);
        fs::copy(lib.as_ref(), tmp_lib)?;
        let library = libloading::Library::new(dir.path().join(lib_name))?;
        let api_load = unsafe {
            library
                .get::<extern "C" fn(*mut fehui::FehUI)>(b"fehui_load")?
                .into_raw()
        };
        Ok(Entry {
            _library: library,
            api_load,
            _dir: dir,
        })
    }

    pub fn load(&mut self, ui: &mut fehui::FehUI) {
        (self.api_load)(ui as *mut _)
    }
}