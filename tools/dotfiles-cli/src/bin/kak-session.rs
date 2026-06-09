use dotfiles_cli::{
    command_output, dotfiles_root_from_current_exe, ensure_dir, sanitize_scope, user_name,
};
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let runtime_dir = env::var("CLI_NOTES_XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(format!("/tmp/xdg-runtime-{}", user_name())));
    let _ = ensure_dir(&runtime_dir);
    let _ = fs::set_permissions(&runtime_dir, fs::Permissions::from_mode(0o700));
    env::set_var("XDG_RUNTIME_DIR", &runtime_dir);

    let tmux_session = command_output("tmux", ["display-message", "-p", "#S"])
        .unwrap_or_else(|| "standalone".to_string());
    let safe_tmux_session = sanitize_scope(&tmux_session);
    let session_name = format!("kak-{safe_tmux_session}");
    env::set_var("KAK_SESSION", &session_name);
    env::set_var("KAK_SYNC_SCOPE", &safe_tmux_session);

    let glow_sync = dotfiles_root_from_current_exe().join("kak/glow-sync.kak");
    let expression = if glow_sync.is_file() {
        format!("rename-client main; source '{}'", glow_sync.display())
    } else {
        "rename-client main".to_string()
    };

    let args: Vec<String> = env::args().skip(1).collect();
    let err = Command::new("kak")
        .arg("-s")
        .arg(&session_name)
        .arg("-e")
        .arg(expression)
        .args(args)
        .exec();
    eprintln!("failed to exec kak: {err}");
    std::process::exit(1);
}
