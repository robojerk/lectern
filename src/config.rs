//! Config directory for Lectern. On Linux uses XDG Base Directory:
//! `$XDG_CONFIG_HOME/lectern` or `~/.config/lectern`.

use std::path::PathBuf;

const APP_CONFIG_DIR: &str = "lectern";

/// Returns the Lectern config directory. On Linux this follows XDG:
/// `$XDG_CONFIG_HOME/lectern` or `~/.config/lectern`. Returns `None` if the
/// base config dir cannot be determined (e.g. no home dir).
pub fn config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join(APP_CONFIG_DIR))
}

/// Returns the path to a file inside the config directory. Creates the config
/// directory if it does not exist. Returns `None` if the config dir cannot be
/// determined.
pub fn config_file(name: &str) -> Option<PathBuf> {
    let path = config_dir()?.join(name);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    Some(path)
}
