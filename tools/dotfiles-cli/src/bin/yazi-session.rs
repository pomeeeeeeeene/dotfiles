use dotfiles_cli::{home_dir, is_executable};
use std::env;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if let Ok(core) = env::var("YAZI_SESSION_CORE") {
        let core = PathBuf::from(core);
        if !is_executable(&core) {
            eprintln!(
                "YAZI_SESSION_CORE is set but not executable: {}",
                core.display()
            );
            std::process::exit(1);
        }
        exec_core(&core, &args);
    }

    let local_core = env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|parent| parent.join("yazi-session-core")));
    if let Some(core) = local_core.as_ref().filter(|path| is_executable(path)) {
        exec_core(core, &args);
    }

    let global_core = env::var("XDG_BIN_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home_dir().join(".local/bin"))
        .join("yazi-session-core");
    if is_executable(&global_core) {
        exec_core(&global_core, &args);
    }

    eprintln!("Missing yazi-session-core.");
    eprintln!(
        "Checked: {} and {}",
        global_core.display(),
        local_core
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "<unknown>".to_string())
    );
    std::process::exit(1);
}

fn exec_core(core: &PathBuf, args: &[String]) -> ! {
    let err = Command::new(core).args(args).exec();
    eprintln!("failed to exec {}: {err}", core.display());
    std::process::exit(1);
}
