//! The outside world nitidus hands off to: the user's editor, the system's opener, and the
//! places files go.

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

/// The opener command on this platform.
const OPENER: &str = if cfg!(target_os = "macos") {
    "open"
} else {
    "xdg-open"
};

/// Opens `path` in `$VISUAL`, then `$EDITOR`, then `vi`, and waits for it to exit.
///
/// The editor value is a shell word list (`vim -Nu NONE`), so it runs through `sh` with the path
/// passed as an argument, never spliced into the command.
pub fn edit(path: &Path) -> Result<()> {
    let editor = env::var("VISUAL")
        .or_else(|_| env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".into());
    let status = Command::new("sh")
        .args(["-c", &format!("{editor} \"$1\""), "nitidus-editor"])
        .arg(path)
        .status()
        .with_context(|| format!("run editor {editor:?}"))?;
    if !status.success() {
        bail!("editor exited with {status}");
    }
    Ok(())
}

/// Opens a file or URL with the system's opener, without waiting.
pub fn open(target: &str) -> Result<()> {
    let opener = Command::new(OPENER)
        .arg(target)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    opener.with_context(|| format!("run {OPENER}"))?;
    Ok(())
}

/// `path` with a leading `~/` replaced by the home directory.
pub fn expand_home(path: &str) -> PathBuf {
    match (path.strip_prefix("~/"), env::var_os("HOME")) {
        (Some(rest), Some(home)) => Path::new(&home).join(rest),
        _ => PathBuf::from(path),
    }
}

/// Where saved attachments go: `$NITIDUS_DOWNLOADS`, else `~/Downloads`.
pub fn downloads_dir() -> PathBuf {
    let configured = env::var_os("NITIDUS_DOWNLOADS").map(PathBuf::from);
    let home = || env::var_os("HOME").map(|home| PathBuf::from(home).join("Downloads"));
    configured.or_else(home).unwrap_or_else(env::temp_dir)
}

/// A private scratch directory for files opened from message `id`.
pub fn scratch_dir(id: &str) -> Result<PathBuf> {
    let dir = env::temp_dir().join("nitidus").join(id);
    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    Ok(dir)
}
