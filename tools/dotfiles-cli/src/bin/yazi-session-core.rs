use dotfiles_cli::{command_output, ensure_dir, home_dir, sanitize_scope, unique_suffix};
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use toml::Value;

fn main() {
    let tmux_session = detect_tmux_session();
    let safe_tmux_session = sanitize_scope(&tmux_session);
    let kak_session = format!("kak-{safe_tmux_session}");

    let xdg_runtime_dir = setup_xdg_runtime_dir();
    env::set_var("KAK_SESSION", &kak_session);
    env::set_var("KAK_SYNC_SCOPE", &safe_tmux_session);

    let config_root = env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home_dir().join(".config"));
    let session_config_prefix =
        env::var("YAZI_SESSION_CONFIG_PREFIX").unwrap_or_else(|_| "yazi-".to_string());
    let session_config_home =
        config_root.join(format!("{session_config_prefix}{safe_tmux_session}"));
    let default_config_home = env::var("YAZI_DEFAULT_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| config_root.join("yazi"));

    let selected_config_home = env::var("YAZI_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            if session_config_home.is_dir() {
                session_config_home.clone()
            } else {
                default_config_home.clone()
            }
        });
    let persist_config_home = selected_config_home.clone();

    let mut yazi_config_home = compose_session_config(
        &selected_config_home,
        &session_config_home,
        &default_config_home,
    )
    .unwrap_or_else(|| selected_config_home.clone());
    env::set_var("YAZI_CONFIG_HOME", &yazi_config_home);

    let dir_config = apply_repo_yazi_config(
        &tmux_session,
        &safe_tmux_session,
        &selected_config_home,
        &mut yazi_config_home,
    );

    if tmux_session != "standalone" {
        tmux_setenv(
            &tmux_session,
            "XDG_RUNTIME_DIR",
            &xdg_runtime_dir.to_string_lossy(),
        );
        tmux_setenv(
            &tmux_session,
            "CLI_NOTES_XDG_RUNTIME_DIR",
            &xdg_runtime_dir.to_string_lossy(),
        );
        tmux_setenv(
            &tmux_session,
            "YAZI_CONFIG_HOME",
            &persist_config_home.to_string_lossy(),
        );
    }

    if is_truthy(&env::var("YAZI_SESSION_DEBUG").unwrap_or_default()) {
        eprintln!("tmux_session={tmux_session}");
        eprintln!("selected_config_home={}", selected_config_home.display());
        eprintln!("persist_config_home={}", persist_config_home.display());
        eprintln!("XDG_RUNTIME_DIR={}", xdg_runtime_dir.display());
        eprintln!("YAZI_CONFIG_HOME={}", yazi_config_home.display());
        eprintln!(
            "YAZI_DIR_CONFIG={} (enabled={})",
            dir_config.mode, dir_config.enabled as u8
        );
        if dir_config.enabled {
            eprintln!("repo_root={}", dir_config.repo_root.unwrap_or_default());
            eprintln!(
                "repo_yazi_toml={}",
                dir_config.repo_yazi_toml.unwrap_or_default()
            );
        }
    }

    if is_truthy(&env::var("YAZI_SESSION_DRY_RUN").unwrap_or_default()) {
        return;
    }

    let args: Vec<String> = env::args().skip(1).collect();
    let err = Command::new("yazi").args(args).exec();
    eprintln!("failed to exec yazi: {err}");
    std::process::exit(1);
}

fn detect_tmux_session() -> String {
    if let Ok(pane) = env::var("TMUX_PANE") {
        if let Some(session) = command_output("tmux", ["display-message", "-p", "-t", &pane, "#S"])
        {
            return session;
        }
    }
    command_output("tmux", ["display-message", "-p", "#S"])
        .unwrap_or_else(|| "standalone".to_string())
}

fn setup_xdg_runtime_dir() -> PathBuf {
    let dir = env::var("CLI_NOTES_XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(format!("/tmp/xdg-runtime-{}", dotfiles_cli::user_name()))
        });
    let _ = ensure_dir(&dir);
    let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
    env::set_var("XDG_RUNTIME_DIR", &dir);
    dir
}

fn compose_session_config(
    selected: &Path,
    session_home: &Path,
    default_home: &Path,
) -> Option<PathBuf> {
    if selected != session_home || !session_home.is_dir() {
        return None;
    }

    let merged = env::temp_dir().join(format!(
        "yazi-config-{}.{}",
        session_home
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("session"),
        unique_suffix()
    ));
    ensure_dir(&merged).ok()?;
    if default_home.is_dir() {
        let _ = copy_dir_contents(default_home, &merged);
    }
    let _ = copy_dir_contents(session_home, &merged);

    let default_toml = default_home.join("yazi.toml");
    let session_toml = session_home.join("yazi.toml");
    if default_toml.is_file() && session_toml.is_file() {
        let merged_toml = merged.join("yazi.toml");
        if merge_toml_files(&default_toml, &session_toml, &merged_toml).is_err() {
            let _ = fs::copy(&session_toml, &merged_toml);
        }
    }

    Some(merged)
}

struct DirConfigState {
    mode: String,
    enabled: bool,
    repo_root: Option<String>,
    repo_yazi_toml: Option<String>,
}

fn apply_repo_yazi_config(
    tmux_session: &str,
    safe_tmux_session: &str,
    selected_config_home: &Path,
    yazi_config_home: &mut PathBuf,
) -> DirConfigState {
    let mode = env::var("YAZI_DIR_CONFIG").unwrap_or_else(|_| "auto".to_string());
    let repo_root = command_output("git", ["rev-parse", "--show-toplevel"]);
    let repo_yazi_toml = repo_root
        .as_ref()
        .map(|root| Path::new(root).join("yazi.toml"));

    let enabled = if env::var("TMUX_PANE").is_err() || tmux_session == "standalone" {
        false
    } else if is_truthy(&mode) {
        true
    } else if is_falsy(&mode) {
        false
    } else if mode == "auto" {
        repo_yazi_toml.as_ref().is_some_and(|path| path.is_file())
    } else {
        eprintln!("Invalid YAZI_DIR_CONFIG value: {mode} (expected auto/true/false)");
        false
    };

    if enabled {
        if let Some(repo_toml) = repo_yazi_toml.as_ref().filter(|path| path.is_file()) {
            if yazi_config_home == selected_config_home {
                let merged = env::temp_dir().join(format!(
                    "yazi-repo-config-{safe_tmux_session}.{}",
                    unique_suffix()
                ));
                let _ = ensure_dir(&merged);
                let _ = copy_dir_contents(selected_config_home, &merged);
                *yazi_config_home = merged;
                env::set_var("YAZI_CONFIG_HOME", &yazi_config_home);
            }

            let current_toml = yazi_config_home.join("yazi.toml");
            let tmp = env::temp_dir().join(format!("yazi-toml-merge.{}", unique_suffix()));
            let _ = if current_toml.is_file() {
                fs::copy(&current_toml, &tmp)
            } else {
                fs::write(&tmp, "").map(|_| 0)
            };
            if merge_toml_files(&tmp, repo_toml, &current_toml).is_err() {
                let _ = fs::copy(repo_toml, &current_toml);
            }
            let _ = fs::remove_file(tmp);
        }
    }

    DirConfigState {
        mode,
        enabled,
        repo_root,
        repo_yazi_toml: repo_yazi_toml.map(|path| path.to_string_lossy().to_string()),
    }
}

fn copy_dir_contents(src: &Path, dst: &Path) -> std::io::Result<()> {
    ensure_dir(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir_contents(&from, &to)?;
        } else if file_type.is_file() || file_type.is_symlink() {
            if let Some(parent) = to.parent() {
                ensure_dir(parent)?;
            }
            let _ = fs::copy(&from, &to);
        }
    }
    Ok(())
}

fn merge_toml_files(base_path: &Path, override_path: &Path, out_path: &Path) -> Result<(), String> {
    let base = read_toml_or_empty(base_path)?;
    let override_value = read_toml_or_empty(override_path)?;
    let merged = deep_merge(base, override_value);
    let text = toml::to_string_pretty(&merged).map_err(|err| err.to_string())?;
    fs::write(out_path, text).map_err(|err| err.to_string())
}

fn read_toml_or_empty(path: &Path) -> Result<Value, String> {
    if !path.is_file() {
        return Ok(Value::Table(Default::default()));
    }
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    if text.trim().is_empty() {
        return Ok(Value::Table(Default::default()));
    }
    text.parse::<Value>().map_err(|err| err.to_string())
}

fn deep_merge(base: Value, override_value: Value) -> Value {
    match (base, override_value) {
        (Value::Table(mut base), Value::Table(override_table)) => {
            for (key, value) in override_table {
                let merged = base
                    .remove(&key)
                    .map(|base_value| deep_merge(base_value, value.clone()))
                    .unwrap_or(value);
                base.insert(key, merged);
            }
            Value::Table(base)
        }
        (_, override_value) => override_value,
    }
}

fn is_truthy(value: &str) -> bool {
    matches!(value, "1" | "true" | "yes" | "on")
}

fn is_falsy(value: &str) -> bool {
    matches!(value, "0" | "false" | "no" | "off")
}

fn tmux_setenv(session: &str, name: &str, value: &str) {
    let _ = Command::new("tmux")
        .args(["set-environment", "-t", session, name, value])
        .status();
}
