//! C libraries the mod loads at run time (a mod crate can't build or link
//! C): libopus and rnnoise. The mod's own data folder is tried first
//! (`horizon-mods/.horizon-mod-data/simple_chat_rust/`, where Windows players put
//! the DLLs), then the system's names.
use libloading::Library;
use std::path::PathBuf;
use std::sync::OnceLock;

/// The mod's data folder, set by the game before the first load.
static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Where the game keeps this mod's files (`Game::native_data_dir`). Only the
/// first call counts; call it before voice chat starts.
pub fn set_data_dir(dir: PathBuf) {
    let _ = DATA_DIR.set(dir);
}

pub fn data_dir() -> Option<&'static PathBuf> {
    DATA_DIR.get()
}

/// The first of `local` (file names in the data folder) and then `system`
/// (names the system's loader finds) that loads; every failure otherwise.
pub fn load(local: &[&str], system: &[&str]) -> Result<Library, String> {
    let mut errors = Vec::new();
    let local = DATA_DIR
        .get()
        .into_iter()
        .flat_map(|dir| local.iter().map(move |name| dir.join(name)))
        .filter(|path| path.is_file());
    for path in local.chain(system.iter().map(PathBuf::from)) {
        // SAFETY: these libraries run no code of their own when they load.
        match unsafe { Library::new(&path) } {
            Ok(lib) => return Ok(lib),
            Err(e) => errors.push(format!("{}: {e}", path.display())),
        }
    }
    Err(errors.join("; "))
}

/// For tests: a folder holding built DLLs stands in for the data folder.
#[cfg(test)]
pub fn test_dir() {
    if let Some(dir) = std::env::var_os("VOICECHAT_TEST_OPUS_DIR") {
        set_data_dir(dir.into());
    }
}
