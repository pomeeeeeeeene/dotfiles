use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn env_or(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

pub fn home_dir() -> PathBuf {
    PathBuf::from(env::var("HOME").unwrap_or_else(|_| ".".to_string()))
}

pub fn state_root() -> PathBuf {
    PathBuf::from(env::var("TMPDIR").unwrap_or_else(|_| "/tmp".to_string()))
}

pub fn user_name() -> String {
    env::var("USER").unwrap_or_else(|_| "unknown".to_string())
}

pub fn sanitize_scope(input: &str) -> String {
    input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

pub fn command_output<S, I, A>(program: S, args: I) -> Option<String>
where
    S: AsRef<OsStr>,
    I: IntoIterator<Item = A>,
    A: AsRef<OsStr>,
{
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

pub fn command_status<S, I, A>(program: S, args: I) -> bool
where
    S: AsRef<OsStr>,
    I: IntoIterator<Item = A>,
    A: AsRef<OsStr>,
{
    Command::new(program)
        .args(args)
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn resolve_symlink_once(path: &Path) -> PathBuf {
    match fs::read_link(path) {
        Ok(target) if target.is_absolute() => target,
        Ok(target) => path.parent().unwrap_or_else(|| Path::new(".")).join(target),
        Err(_) => path.to_path_buf(),
    }
}

pub fn dotfiles_root_from_current_exe() -> PathBuf {
    let exe = env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let resolved = resolve_symlink_once(&exe);

    // During development/build:
    //   <repo>/tools/dotfiles-cli/target/{debug,release}/<bin>
    // Installed wrapper executes that same binary.
    let mut current = resolved.as_path();
    while let Some(parent) = current.parent() {
        if parent.join("tools/dotfiles-cli/Cargo.toml").exists() {
            return parent.to_path_buf();
        }
        current = parent;
    }

    home_dir().join("dotfiles")
}

pub fn unique_suffix() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("{}-{}", std::process::id(), now)
}

pub fn ensure_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)
}

pub fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path)
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }

    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

pub fn kak_quote_single(input: &str) -> String {
    input.replace('\'', "''")
}

pub fn toml_escape(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}
